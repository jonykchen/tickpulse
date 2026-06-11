use crate::alert::{AlertManager, AlertRule};
use crate::datasource::DataSourceManager;
use crate::db::DbPool;
use crate::health::HealthCollector;
use crate::market::exchange::Exchange;
use crate::market::northbound_cache::NorthboundFlow;
use crate::market::types::MarketPhase;
use crate::market::{MarketSummary, SchedulerStatus, StockQuote, VolumeRatioNote};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// 行情调度器
/// 14阶段精细化调度，含增量推送、数据自愈、多数据源容灾
/// + 预警检查 + 北向资金缓存 + 托盘更新 + 健康采集
pub struct MarketScheduler {
    app_handle: tauri::AppHandle,
    cancel_token: CancellationToken,
    source: Arc<DataSourceManager>,
    db: Arc<DbPool>,
    prev_quotes: Arc<DashMap<String, StockQuote>>,
    current_phase: Arc<RwLock<MarketPhase>>,
    consecutive_errors: std::sync::atomic::AtomicU32,
    health: Arc<std::sync::Mutex<HealthCollector>>,
    /// 上次北向资金抓取时间（秒级时间戳）
    last_northbound_fetch: Arc<std::sync::Mutex<i64>>,
    /// 上次预警重置日期（YYYY-MM-DD）
    last_alert_reset_date: Arc<std::sync::Mutex<String>>,
    /// 当日开盘时间（秒级时间戳），用于量比衰减判断
    market_open_time: Arc<std::sync::Mutex<Option<i64>>>,
}

impl MarketScheduler {
    /// 接受外部注入的 DataSourceManager + HealthCollector（与 lib.rs AppState 共享）
    pub fn new(
        app_handle: tauri::AppHandle,
        db: Arc<DbPool>,
        source: Arc<DataSourceManager>,
        health: Arc<std::sync::Mutex<HealthCollector>>,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            app_handle,
            cancel_token,
            source,
            db,
            prev_quotes: Arc::new(DashMap::new()),
            current_phase: Arc::new(RwLock::new(MarketPhase::PreMarket)),
            consecutive_errors: std::sync::atomic::AtomicU32::new(0),
            health,
            last_northbound_fetch: Arc::new(std::sync::Mutex::new(0)),
            last_alert_reset_date: Arc::new(std::sync::Mutex::new(String::new())),
            market_open_time: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// 获取当日开盘时间（09:30）的时间戳
    fn get_today_open_time(&self) -> i64 {
        let today = chrono::Local::now();
        let open_time = today.date_naive().and_hms_opt(9, 30, 0).unwrap();
        open_time.and_utc().timestamp()
    }

    /// 更新当日开盘时间（时设置）
    fn update_market_open_time(&self) {
        let mut open_time = self.market_open_time.lock().unwrap();
        if open_time.is_none() {
            *open_time = Some(self.get_today_open_time());
            tracing::debug!("当日开盘时间已设置: {}", open_time.unwrap());
        }
    }

    /// 启动调度器主循环
    pub async fn run(&self, is_trading_day: bool) {
        let phase = MarketPhase::current(is_trading_day, Exchange::SZSE);
        *self.current_phase.write().await = phase;

        // 推送调度器状态
        let _ = self.app_handle.emit(
            "scheduler-status",
            SchedulerStatus {
                phase: phase.display_name().to_string(),
                interval_secs: phase.refresh_interval().as_secs(),
                is_trading_day,
            },
        );

        let interval = phase.refresh_interval();

        // 每日首次运行时自动重置预警（开盘前）
        self.auto_reset_daily_alerts().await;

        // 每日首次运行时设置开盘时间（用于量比衰减判断）
        self.update_market_open_time();

        if phase.has_realtime_data() {
            let start = std::time::Instant::now();

            match self.refresh_and_emit().await {
                Ok(()) => {
                    self.consecutive_errors
                        .store(0, std::sync::atomic::Ordering::Relaxed);

                    // 记录健康指标
                    let elapsed_ms = start.elapsed().as_millis() as u64;
                    if let Ok(mut h) = self.health.lock() {
                        h.record_refresh(elapsed_ms);
                    }
                }
                Err(e) => {
                    let errors = self
                        .consecutive_errors
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                        + 1;
                    tracing::error!("行情刷新连续失败 #{}: {}", errors, e);

                    if errors >= 10 {
                        tracing::warn!("连续失败10次，建议检查网络连接");
                        self.consecutive_errors
                            .store(0, std::sync::atomic::Ordering::Relaxed);
                    }

                    // 指数退避：5s → 10s → 15s → ... → 30s
                    let backoff = Duration::from_secs(5 * errors.min(6) as u64);
                    tokio::time::sleep(backoff).await;
                    return;
                }
            }

            // 交易时段内执行：预警检查 + 北向资金缓存
            let db = self.db.clone();
            let app_handle = self.app_handle.clone();
            let prev_quotes = self.prev_quotes.clone();
            let last_northbound = self.last_northbound_fetch.clone();

            // 预警检查（每次刷新后，使用 HashMap 优化）
            tokio::spawn(async move {
                check_alerts_and_notify(&db, &prev_quotes, &app_handle).await;
            });

            // 北向资金缓存（每5分钟抓取一次，复用共享 HTTP Client）
            let now_ts = chrono::Utc::now().timestamp();
            let should_fetch_northbound = {
                let last = last_northbound.lock().unwrap();
                now_ts - *last >= 300 // 5分钟
            };
            if should_fetch_northbound {
                let db2 = self.db.clone();
                let last_nb = self.last_northbound_fetch.clone();
                tokio::spawn(async move {
                    fetch_and_cache_northbound(&db2).await;
                    let mut last = last_nb.lock().unwrap();
                    *last = chrono::Utc::now().timestamp();
                });
            }

            // 更新托盘 tooltip
            update_tray_tooltip(&self.prev_quotes, &self.db, &self.app_handle);
        }

        tokio::select! {
            _ = tokio::time::sleep(interval) => {},
            _ = self.cancel_token.cancelled() => {
                tracing::info!("行情调度器已停止");
            }
        }
    }

    /// 每日首次运行自动重置预警（避免重复触发）
    async fn auto_reset_daily_alerts(&self) {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let should_reset = {
            let last_date = self.last_alert_reset_date.lock().unwrap();
            *last_date != today
        };

        if should_reset {
            let manager = AlertManager::new(self.db.clone());
            match manager.reset_daily() {
                Ok(()) => {
                    let mut last_date = self.last_alert_reset_date.lock().unwrap();
                    *last_date = today.clone();
                    tracing::info!("每日预警自动重置完成 ({})", today);
                }
                Err(e) => {
                    tracing::warn!("每日预警重置失败: {}", e);
                }
            }
        }
    }

    /// 刷新行情并增量推送（使用并行分批请求）
    async fn refresh_and_emit(&self) -> Result<(), String> {
        let secids = self
            .db
            .get_all_watchlist_secids()
            .map_err(|e| e.to_string())?;

        if secids.is_empty() {
            return Ok(());
        }

        // 使用并行分批请求（200只→4批×50并行），提升大批量刷新性能
        let mut quotes = self.source.fetch_quotes_parallel(&secids).await?;

        // 量比衰减逻辑：开盘30分钟内（09:30-10:00）量比偏高，标记 Early
        let now_ts = chrono::Utc::now().timestamp();
        let elapsed_minutes = {
            let market_open = self.market_open_time.lock().unwrap();
            market_open
                .map(|open_ts| (now_ts - open_ts) / 60)
                .unwrap_or(999)
        };

        for quote in &mut quotes {
            let volume_ratio_note = if elapsed_minutes < 30 && quote.volume_ratio > 0.0 {
                Some(VolumeRatioNote::Early)
            } else {
                None
            };
            quote.volume_ratio_note = volume_ratio_note;
        }

        // 增量过滤
        let changed: Vec<StockQuote> = quotes
            .iter()
            .filter(|q| {
                self.prev_quotes
                    .get(&q.secid)
                    .map_or(true, |p| p.price != q.price || p.change_percent != q.change_percent)
            })
            .cloned()
            .collect();

        // 静默期(PreOpen)无变化则跳过推送
        let current_phase = *self.current_phase.read().await;
        if current_phase == MarketPhase::PreOpen && changed.is_empty() {
            return Ok(());
        }

        // 更新快照
        for q in &quotes {
            self.prev_quotes.insert(q.secid.clone(), q.clone());
        }

        // 推送增量数据
        if !changed.is_empty() {
            let _ = self.app_handle.emit("stock-update", &changed);
        }

        // 推送涨跌家数摘要
        let summary = MarketSummary::from_quotes(&quotes);
        let _ = self.app_handle.emit("market-summary", &summary);

        Ok(())
    }

    /// 获取当前阶段
    pub async fn current_phase(&self) -> MarketPhase {
        *self.current_phase.read().await
    }

    /// 获取健康诊断数据
    pub fn health_metrics(&self) -> crate::health::HealthMetrics {
        let collector = self.health.lock().unwrap();
        let circuit_status = self.source.circuit_breaker_status()
            .into_iter()
            .map(|(name, state)| crate::health::CircuitBreakerStatus {
                source_name: name,
                state: format!("{:?}", state),
                consecutive_failures: 0,
            })
            .collect();
        collector.collect(0, circuit_status)
    }

    /// 停止调度器
    pub fn stop(&self) {
        self.cancel_token.cancel();
    }
}

// ==================== 预警检查（HashMap 优化） ====================

/// 刷新后检查预警并触发事件
/// 优化：使用 HashMap 避免重复查 DB，一次遍历完成匹配
async fn check_alerts_and_notify(
    db: &Arc<DbPool>,
    prev_quotes: &Arc<DashMap<String, StockQuote>>,
    app_handle: &tauri::AppHandle,
) {
    // 获取所有启用的预警规则
    let rules = match db.get_alert_rules() {
        Ok(r) => r.into_iter().filter(|r: &AlertRule| r.enabled && !r.triggered).collect::<Vec<_>>(),
        Err(e) => {
            tracing::warn!("获取预警规则失败: {}", e);
            return;
        }
    };

    if rules.is_empty() {
        return;
    }

    // 构建行情 HashMap（O(1) 查找，避免对每条规则遍历 DashMap）
    let quote_map: HashMap<String, StockQuote> = rules.iter()
        .filter_map(|r| prev_quotes.get(&r.secid).map(|q| (r.secid.clone(), q.value().clone())))
        .collect();

    // 收集相关行情
    let quotes: Vec<StockQuote> = quote_map.values().cloned().collect();

    // 执行预警检查
    let manager = AlertManager::new(db.clone());
    let triggered = manager.check_and_notify(&quotes, &rules);

    if triggered.is_empty() {
        return;
    }

    // 构建 secid+rule_type → rule_id 映射（一次查 DB，避免 N 次循环）
    let all_rules = match db.get_alert_rules() {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("查询预警规则失败: {}", e);
            return;
        }
    };
    let rule_map: HashMap<(String, String), String> = all_rules.iter()
        .map(|r| ((r.secid.clone(), format!("{:?}", r.rule_type)), r.id.clone()))
        .collect();

    // 标记已触发的规则
    for alert in &triggered {
        let key = (alert.secid.clone(), format!("{:?}", alert.rule_type));
        if let Some(rule_id) = rule_map.get(&key) {
            let _ = db.mark_alert_triggered(rule_id);
        }
    }

    // 发送预警触发事件
    let _ = app_handle.emit("alert-triggered", &triggered);

    // 发送系统通知（使用 tauri_plugin_notification v2 API）
    for alert in &triggered {
        use tauri_plugin_notification::NotificationExt;
        let _ = app_handle.notification()
            .builder()
            .title("TickPulse预警")
            .body(&alert.message)
            .show();
    }
}

// ==================== 北向资金缓存（复用共享 HTTP Client） ====================

/// 抓取北向资金数据并写入缓存
async fn fetch_and_cache_northbound(db: &Arc<DbPool>) {
    let url = "https://push2his.eastmoney.com/api/qt/kamt.kline/get?fields1=f1,f2,f3,f4&fields2=f51,f52,f53,f54,f55,f56&klt=101&lmt=10&ut=b73502e0ed8b4e5f9c5ce1ec0c7c7d86";

    // 复用 lib.rs 全局共享 HTTP 客户端
    let resp = match crate::SHARED_HTTP_CLIENT.get(url)
        .header("Referer", "https://data.eastmoney.com/")
        .send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("北向资金请求失败: {}", e);
            return;
        }
    };

    let body: serde_json::Value = match resp.json().await {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("北向资金解析失败: {}", e);
            return;
        }
    };

    // 解析北向资金K线数据
    // 格式: klines数组，每条 "日期,沪股通净买入,深股通净买入,北向合计净买入,沪余额,深余额"
    if let Some(klines) = body.get("data").and_then(|d| d.get("s2n")).and_then(|v| v.as_array()) {
        for kline in klines {
            if let Some(line) = kline.as_str() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 4 {
                    let flow = NorthboundFlow {
                        trade_date: parts[0].to_string(),
                        sh_net_inflow: parts[1].parse().unwrap_or(0.0),
                        sz_net_inflow: parts[2].parse().unwrap_or(0.0),
                        total_net_inflow: parts[3].parse().unwrap_or(0.0),
                    };
                    if let Err(e) = db.save_northbound(&flow) {
                        tracing::warn!("北向资金缓存写入失败: {}", e);
                    }
                }
            }
        }
        tracing::debug!("北向资金缓存更新完成");
    }
}

// ==================== 托盘更新 ====================

/// 更新托盘 Tooltip 显示涨跌概览、北向资金、持仓盈亏
fn update_tray_tooltip(
    prev_quotes: &Arc<DashMap<String, StockQuote>>,
    db: &Arc<DbPool>,
    app_handle: &tauri::AppHandle,
) {
    use crate::system::tray::TrayTooltipData;

    let mut data = TrayTooltipData::default();

    // 1. 计算涨跌家数
    if !prev_quotes.is_empty() {
        for entry in prev_quotes.iter() {
            let q = entry.value();
            if q.change_percent > 0.0 {
                data.up_count += 1;
            } else if q.change_percent < 0.0 {
                data.down_count += 1;
            }
        }
    }

    // 2. 提取上证指数行情（secid: 1.000001）
    if let Some(index_quote) = prev_quotes.get("1.000001") {
        data.sh_price = Some(index_quote.price);
        data.sh_change_percent = Some(index_quote.change_percent);
    }

    // 3. 获取最新北向资金（从缓存）
    if let Ok(flows) = db.get_northbound_recent(1) {
        if let Some(latest) = flows.first() {
            data.northbound_inflow = Some(latest.total_net_inflow);
        }
    }

    // 4. 计算持仓盈亏（如有）
    if let Ok(positions) = db.get_all_positions() {
        if !positions.is_empty() {
            let mut total_pnl = rust_decimal::Decimal::ZERO;
            let mut has_valid = false;

            for pos in &positions {
                // 从缓存获取当前行情
                if let Some(quote) = prev_quotes.get(&pos.secid) {
                    if let Ok(qty) = pos.quantity.parse::<rust_decimal::Decimal>() {
                        if let Ok(cost) = pos.cost_price.parse::<rust_decimal::Decimal>() {
                            let pnl = qty * (rust_decimal::Decimal::from_f64_retain(quote.price).unwrap_or_default() - cost);
                            total_pnl += pnl;
                            has_valid = true;
                        }
                    }
                }
            }

            if has_valid {
                let pnl_str = if total_pnl >= rust_decimal::Decimal::ZERO {
                    format!("+{:.2}", total_pnl)
                } else {
                    format!("{:.2}", total_pnl)
                };
                data.position_pnl = Some(pnl_str);
            }
        }
    }

    // 更新托盘
    crate::system::tray::update_tray_with_data(app_handle, &data);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_phase_current() {
        // 非交易日
        let phase = MarketPhase::current(false, Exchange::SZSE);
        assert_eq!(phase, MarketPhase::Holiday);
    }
}
