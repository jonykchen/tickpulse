use crate::db::DbPool;
use crate::market::exchange::Exchange;
use crate::market::sources::eastmoney::EastMoneySource;
use crate::market::types::MarketPhase;
use crate::market::{MarketDataSource, MarketSummary, SchedulerStatus, StockQuote};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// 行情调度器
/// 14阶段精细化调度，含增量推送、数据自愈
pub struct MarketScheduler {
    app_handle: tauri::AppHandle,
    cancel_token: CancellationToken,
    source: Arc<EastMoneySource>,
    db: Arc<DbPool>,
    prev_quotes: Arc<DashMap<String, StockQuote>>,
    current_phase: Arc<RwLock<MarketPhase>>,
    consecutive_errors: std::sync::atomic::AtomicU32,
}

impl MarketScheduler {
    pub fn new(
        app_handle: tauri::AppHandle,
        db: Arc<DbPool>,
        cancel_token: CancellationToken,
    ) -> Self {
        let source = Arc::new(EastMoneySource::new());
        Self {
            app_handle,
            cancel_token,
            source,
            db,
            prev_quotes: Arc::new(DashMap::new()),
            current_phase: Arc::new(RwLock::new(MarketPhase::PreMarket)),
            consecutive_errors: std::sync::atomic::AtomicU32::new(0),
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

        if phase.has_realtime_data() {
            match self.refresh_and_emit().await {
                Ok(()) => {
                    self.consecutive_errors
                        .store(0, std::sync::atomic::Ordering::Relaxed);
                }
                Err(e) => {
                    let errors = self
                        .consecutive_errors
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                        + 1;
                    tracing::error!("行情刷新连续失败 #{}: {}", errors, e);

                    if errors >= 10 {
                        tracing::warn!("连续失败10次，建议重建HTTP客户端");
                        self.consecutive_errors
                            .store(0, std::sync::atomic::Ordering::Relaxed);
                    }

                    // 指数退避：5s → 10s → 15s → ... → 30s
                    let backoff = Duration::from_secs(5 * errors.min(6) as u64);
                    tokio::time::sleep(backoff).await;
                    return;
                }
            }
        }

        tokio::select! {
            _ = tokio::time::sleep(interval) => {},
            _ = self.cancel_token.cancelled() => {
                tracing::info!("行情调度器已停止");
            }
        }
    }

    /// 刷新行情并增量推送
    async fn refresh_and_emit(&self) -> Result<(), String> {
        let secids = self
            .db
            .get_all_watchlist_secids()
            .map_err(|e| e.to_string())?;

        if secids.is_empty() {
            return Ok(());
        }

        let quotes = self.source.fetch_quotes(&secids).await?;

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

    /// 停止调度器
    pub fn stop(&self) {
        self.cancel_token.cancel();
    }
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
