use crate::config::validate_secid_or_error;
use crate::datasource::DataSourceManager;
use crate::health::HealthCollector;
use crate::market::MarketDataSource;
use once_cell::sync::Lazy;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Manager;

mod adjust;
mod alert;
mod analysis;
mod anomaly;
mod cache;
mod config;
mod datasource;
mod db;
mod health;
mod indicator;
mod limit;
mod market;
mod sidecar;
mod sync;
mod system;

/// 全局共享 HTTP 客户端（连接池复用，避免每次请求新建 TCP 连接）
pub(crate) static SHARED_HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config::constants::HTTP_TIMEOUT_SECS))
        .gzip(true)
        .build()
        .expect("Failed to build shared HTTP client")
});

/// 应用级共享状态（DbPool + DataSourceManager + HealthCollector）
/// 通过 Tauri State 注入，所有 Command 共享同一个 DataSourceManager 实例
pub struct AppState {
    pub db: db::DbPool,
    pub source: Arc<DataSourceManager>,
    pub health: Arc<Mutex<HealthCollector>>,
}

// ==================== Tauri 命令 ====================

/// 获取自选股分组
#[tauri::command]
fn get_watchlist_groups(state: tauri::State<'_, AppState>) -> Result<Vec<db::watchlist::WatchlistGroup>, String> {
    state.db.get_watchlist_groups().map_err(|e| e.to_string())
}

/// 创建自选股分组
#[tauri::command]
fn create_watchlist_group(name: &str, state: tauri::State<'_, AppState>) -> Result<i64, String> {
    state.db.create_watchlist_group(name).map_err(|e| e.to_string())
}

/// 删除自选股分组
#[tauri::command]
fn delete_watchlist_group(id: i64, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.db.delete_watchlist_group(id).map_err(|e| e.to_string())
}

/// 重命名分组
#[tauri::command]
fn rename_watchlist_group(id: i64, name: &str, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.db.rename_watchlist_group(id, name).map_err(|e| e.to_string())
}

/// 获取分组下的自选股
#[tauri::command]
fn get_watchlist_stocks(group_id: i64, state: tauri::State<'_, AppState>) -> Result<Vec<db::watchlist::WatchlistStock>, String> {
    state.db.get_watchlist_stocks(group_id).map_err(|e| e.to_string())
}

/// 添加自选股
#[tauri::command]
fn add_watchlist_stock(group_id: i64, secid: &str, name: Option<&str>, state: tauri::State<'_, AppState>) -> Result<i64, String> {
    validate_secid_or_error(secid)?;
    state.db.add_watchlist_stock(group_id, secid, name).map_err(|e| e.to_string())
}

/// 删除自选股
#[tauri::command]
fn remove_watchlist_stock(id: i64, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.db.remove_watchlist_stock(id).map_err(|e| e.to_string())
}

/// 置顶/取消置顶自选股
#[tauri::command]
fn toggle_pin_watchlist_stock(id: i64, pinned: bool, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.db.toggle_pin_watchlist_stock(id, pinned).map_err(|e| e.to_string())
}

/// 获取持仓列表
#[tauri::command]
fn get_positions(group_id: i64, state: tauri::State<'_, AppState>) -> Result<Vec<db::position::Position>, String> {
    state.db.get_positions(group_id).map_err(|e| e.to_string())
}

/// 添加持仓
#[tauri::command]
fn add_position(
    group_id: i64,
    secid: &str,
    name: Option<&str>,
    cost_price: &str,
    quantity: &str,
    state: tauri::State<'_, AppState>,
) -> Result<i64, String> {
    validate_secid_or_error(secid)?;
    state.db.add_position(group_id, secid, name, cost_price, quantity).map_err(|e| e.to_string())
}

/// 更新持仓
#[tauri::command]
fn update_position(id: i64, cost_price: &str, quantity: &str, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.db.update_position(id, cost_price, quantity).map_err(|e| e.to_string())
}

/// 删除持仓
#[tauri::command]
fn delete_position(id: i64, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.db.delete_position(id).map_err(|e| e.to_string())
}

/// 获取持仓汇总（通过共享 DataSourceManager 获取行情，享有容灾保护）
#[tauri::command]
async fn get_portfolio_summary(
    group_id: i64,
    benchmark_change: Option<f64>,
    state: tauri::State<'_, AppState>,
) -> Result<db::position::PortfolioSummary, String> {
    use rust_decimal::prelude::*;
    use rust_decimal::Decimal;

    let positions = state.db.get_positions(group_id).map_err(|e| e.to_string())?;
    let secids: Vec<String> = positions.iter().map(|p| p.secid.clone()).collect();
    let quotes = if secids.is_empty() {
        vec![]
    } else {
        state.source.fetch_quotes(&secids).await.unwrap_or_default()
    };

    let position_quotes: Vec<db::position::PositionQuote> = positions
        .iter()
        .filter_map(|p| {
            let quote = quotes.iter().find(|q| q.secid == p.secid)?;
            Some(db::position::PositionQuote {
                secid: p.secid.clone(),
                cost_price: Decimal::from_str(&p.cost_price).unwrap_or(Decimal::ZERO),
                quantity: Decimal::from_str(&p.quantity).unwrap_or(Decimal::ZERO),
                current_price: Decimal::from_f64(quote.price).unwrap_or(Decimal::ZERO),
                prev_close: Decimal::from_f64(quote.pre_close).unwrap_or(Decimal::ZERO),
            })
        })
        .collect();

    let bc = benchmark_change.and_then(|v| Decimal::from_f64(v));
    Ok(db::position::summarize(&position_quotes, bc))
}

/// 获取配置（返回 HashMap 与前端 Record<string, string> 对齐）
#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let pairs = state.db.get_all_settings().map_err(|e| e.to_string())?;
    Ok(pairs.into_iter().collect())
}

/// 更新配置
#[tauri::command]
fn update_setting(key: &str, value: &str, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.db.upsert_setting(key, value).map_err(|e| e.to_string())
}

/// 搜索股票（通过 DataSourceManager 容灾链路）
#[tauri::command]
async fn search_stock(keyword: &str, _state: tauri::State<'_, AppState>) -> Result<Vec<market::SearchResult>, String> {
    // 搜索优先走东财源（DataSourceManager 中第一个支持 search 的源）
    // 直接尝试各源的 search 方法
    let source = market::sources::eastmoney::EastMoneySource::new();
    source.search(keyword).await
}

/// 批量获取行情（通过 DataSourceManager 容灾链路）
#[tauri::command]
async fn fetch_quotes(secids: Vec<String>, state: tauri::State<'_, AppState>) -> Result<Vec<market::StockQuote>, String> {
    for secid in &secids {
        validate_secid_or_error(secid)?;
    }
    state.source.fetch_quotes(&secids).await
}

/// 获取K线数据（通过 DataSourceManager 容灾链路）
#[tauri::command]
async fn fetch_kline(
    secid: &str,
    period: &str,
    limit: u32,
    adjust: &str,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<market::KlineBar>, String> {
    validate_secid_or_error(secid)?;
    let kline_period = match period {
        "1m" => market::KlinePeriod::Min1,
        "5m" => market::KlinePeriod::Min5,
        "15m" => market::KlinePeriod::Min15,
        "30m" => market::KlinePeriod::Min30,
        "60m" => market::KlinePeriod::Min60,
        "week" => market::KlinePeriod::Weekly,
        "month" => market::KlinePeriod::Monthly,
        _ => market::KlinePeriod::Daily,
    };
    let adjust_type = match adjust {
        "none" => market::AdjustType::None,
        "backward" => market::AdjustType::Backward,
        _ => market::AdjustType::Forward,
    };
    state.source.fetch_kline(secid, kline_period, limit, adjust_type).await
}

/// 获取分时走势（通过 DataSourceManager 容灾链路）
#[tauri::command]
async fn fetch_timeline(secid: &str, state: tauri::State<'_, AppState>) -> Result<market::TimelineData, String> {
    validate_secid_or_error(secid)?;
    state.source.fetch_timeline(secid).await
}

/// 获取除权除息（通过 DataSourceManager 容灾链路）
#[tauri::command]
async fn fetch_exrights(secid: &str, state: tauri::State<'_, AppState>) -> Result<Vec<market::ExRightInfo>, String> {
    validate_secid_or_error(secid)?;
    state.source.fetch_exrights(secid).await
}

/// 获取健康诊断（共享 scheduler 的 HealthCollector）
#[tauri::command]
fn get_health_metrics(state: tauri::State<'_, AppState>) -> Result<health::HealthMetrics, String> {
    let collector = state.health.lock().unwrap();
    let circuit_status = state.source.circuit_breaker_status()
        .into_iter()
        .map(|(name, cs)| health::CircuitBreakerStatus {
            source_name: name,
            state: format!("{:?}", cs),
            consecutive_failures: 0,
        })
        .collect();

    // 获取数据库文件大小
    let db_size_bytes = state.db.db_file_size().unwrap_or(0);

    Ok(collector.collect(db_size_bytes, circuit_status))
}

// ==================== 预警规则 Commands ====================

/// 获取预警规则列表
#[tauri::command]
fn get_alert_rules(state: tauri::State<'_, AppState>) -> Result<Vec<alert::AlertRule>, String> {
    state.db.get_alert_rules().map_err(|e| e.to_string())
}

/// 添加预警规则
#[tauri::command]
fn add_alert_rule(
    secid: &str,
    stock_name: &str,
    rule_type: &str,
    threshold: f64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    validate_secid_or_error(secid)?;
    let rt = alert::AlertRuleType::from_str_ex(rule_type);
    let id = uuid::Uuid::new_v4().to_string();
    state.db.add_alert_rule(&id, secid, stock_name, &rt, threshold).map_err(|e| e.to_string())
}

/// 删除预警规则
#[tauri::command]
fn remove_alert_rule(rule_id: &str, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.db.remove_alert_rule(rule_id).map_err(|e| e.to_string())
}

/// 切换预警规则启用/禁用
#[tauri::command]
fn toggle_alert_rule(rule_id: &str, enabled: bool, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.db.toggle_alert_rule(rule_id, enabled).map_err(|e| e.to_string())
}

// ==================== 大宗交易 Command ====================

/// 获取大宗交易数据（使用共享 HTTP 客户端）
#[tauri::command]
async fn fetch_block_trades(date: Option<String>) -> Result<Vec<market::BlockTrade>, String> {
    let trade_date = date.unwrap_or_else(|| {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    });

    let url = format!(
        "https://datacenter-web.eastmoney.com/api/data/v1/get?reportName=RPT_DATASPECIAL_BIGDEAL&columns=ALL&filter=(TRADE_DATE='{}')&pageNumber=1&pageSize=50&sortTypes=-1&sortColumns=DEAL_AMT",
        trade_date
    );

    let resp = SHARED_HTTP_CLIENT.get(&url)
        .header("Referer", "https://data.eastmoney.com/")
        .send().await
        .map_err(|e| format!("大宗交易请求失败: {}", e))?;

    let body: serde_json::Value = resp.json().await
        .map_err(|e| format!("大宗交易解析失败: {}", e))?;

    let mut trades = Vec::new();
    if let Some(items) = body.get("result").and_then(|r| r.get("data")).and_then(|d| d.as_array()) {
        for item in items {
            let market_code: i64 = item.get("MARKET_CODE").and_then(|v| v.as_i64()).unwrap_or(1);
            let code = item.get("SECURITY_CODE").and_then(|v| v.as_str()).unwrap_or("");
            let secid = format!("{}.{}", market_code, code);

            trades.push(market::BlockTrade {
                secid,
                code: code.to_string(),
                name: item.get("SECURITY_NAME_ABBR").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                trade_date: item.get("TRADE_DATE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                price: item.get("DEAL_PRICE").and_then(|v| v.as_f64()).unwrap_or(0.0),
                volume: item.get("DEAL_VOL").and_then(|v| v.as_f64()).unwrap_or(0.0),
                amount: item.get("DEAL_AMT").and_then(|v| v.as_f64()).unwrap_or(0.0),
                buyer: item.get("BUYER_NAME").and_then(|v| v.as_str()).map(|s| s.to_string()),
                seller: item.get("SELLER_NAME").and_then(|v| v.as_str()).map(|s| s.to_string()),
                premium_rate: item.get("PREMIUM_RATE").and_then(|v| v.as_f64()),
            });
        }
    }

    Ok(trades)
}

// ==================== AI 分析引擎 Commands ====================

/// 执行股票分析
#[tauri::command]
async fn analyze_stock(
    secid: String,
    stock_name: String,
    state: tauri::State<'_, AppState>,
) -> Result<analysis::engine::AnalysisResult, String> {
    validate_secid_or_error(&secid)?;
    let engine = analysis::engine::AnalysisEngine::new(Arc::new(state.db.clone()));
    let config = engine.get_llm_config().unwrap_or(analysis::engine::LlmConfig {
        provider: analysis::engine::CloudProvider::Anthropic,
        model: "claude-sonnet-4-6".to_string(),
        api_key: None,
        base_url: None,
        mode: analysis::engine::LlmMode::Cloud,
    });
    engine.run(&secid, &stock_name, &config).await
}

/// 获取分析历史
#[tauri::command]
fn get_analysis_history(
    secid: Option<String>,
    limit: Option<u32>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<analysis::engine::AnalysisResult>, String> {
    let engine = analysis::engine::AnalysisEngine::new(Arc::new(state.db.clone()));
    engine.get_history(secid.as_deref(), limit.unwrap_or(20))
}

/// 获取 LLM 配置
#[tauri::command]
fn get_llm_config(state: tauri::State<'_, AppState>) -> Result<analysis::engine::LlmConfig, String> {
    let engine = analysis::engine::AnalysisEngine::new(Arc::new(state.db.clone()));
    engine.get_llm_config()
}

/// 保存 LLM 配置
#[tauri::command]
fn save_llm_config(
    provider: String,
    model: String,
    api_key: Option<String>,
    base_url: Option<String>,
    mode: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let engine = analysis::engine::AnalysisEngine::new(Arc::new(state.db.clone()));
    let config = analysis::engine::LlmConfig {
        provider: analysis::engine::CloudProvider::from_str(&provider),
        model,
        api_key,
        base_url,
        mode: if mode == "local" { analysis::engine::LlmMode::Local } else { analysis::engine::LlmMode::Cloud },
    };
    engine.save_llm_config(&config)
}

/// 获取分析预设列表
#[tauri::command]
fn get_analysis_presets() -> Vec<serde_json::Value> {
    let presets = analysis::profiles::default_presets();
    presets.iter().map(|p| serde_json::to_value(p).unwrap_or_default()).collect()
}

/// 计算 PEG
#[tauri::command]
fn calc_peg(pe: f64, cagr: f64) -> Result<serde_json::Value, String> {
    use rust_decimal::prelude::*;
    use rust_decimal::Decimal;
    let pe_d = Decimal::from_f64(pe).ok_or("无效的PE值")?;
    let cagr_d = Decimal::from_f64(cagr).ok_or("无效的CAGR值")?;
    match analysis::peg::PegCalculator::calc_peg(pe_d, cagr_d) {
        Some(peg) => {
            let rating = analysis::peg::PegRating::from_peg(peg);
            Ok(serde_json::json!({
                "peg": peg.to_f64().unwrap_or(0.0),
                "rating": format!("{:?}", rating),
                "ratingDisplay": rating.display(),
                "ratingScore": rating.score(),
            }))
        }
        None => Err("CAGR 必须 > 0 才能计算 PEG".to_string()),
    }
}

/// 计算复合年增长率 CAGR
#[tauri::command]
fn calc_cagr(begin_value: f64, end_value: f64, years: f64) -> Result<f64, String> {
    use rust_decimal::prelude::*;
    use rust_decimal::Decimal;
    let begin = Decimal::from_f64(begin_value).ok_or("无效的起始值")?;
    let end = Decimal::from_f64(end_value).ok_or("无效的终止值")?;
    let yrs = Decimal::from_f64(years).ok_or("无效的年数")?;
    match analysis::peg::PegCalculator::calc_cagr(begin, end, yrs) {
        Some(cagr) => Ok(cagr.to_f64().unwrap_or(0.0)),
        None => Err("参数必须 > 0".to_string()),
    }
}

/// 删除分析结果
#[tauri::command]
fn delete_analysis_result(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM analysis_results WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| format!("删除分析结果失败: {}", e))?;
    Ok(())
}

/// 获取分析进度（预留事件推送）
#[tauri::command]
fn get_analysis_progress() -> serde_json::Value {
    serde_json::json!({ "percent": 0.0 })
}

/// 重置每日预警（开盘前调用）
#[tauri::command]
fn reset_daily_alerts(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let manager = alert::AlertManager::new(Arc::new(state.db.clone()));
    manager.reset_daily()
}

/// 查询自启状态
#[tauri::command]
async fn is_autostart_enabled(app: tauri::AppHandle) -> bool {
    system::autostart::is_autostart_enabled(&app)
}

/// 设置自启状态
#[tauri::command]
async fn set_autostart(enabled: bool, app: tauri::AppHandle) -> Result<(), String> {
    if enabled {
        system::autostart::enable_autostart(&app).map_err(|e| format!("启用自启失败: {:?}", e))
    } else {
        system::autostart::disable_autostart(&app).map_err(|e| format!("禁用自启失败: {:?}", e))
    }
}

/// 打开悬浮窗
#[tauri::command]
async fn open_suspend_window(app: tauri::AppHandle) {
    system::window::open_suspend_window(&app);
}

/// 关闭悬浮窗
#[tauri::command]
async fn close_suspend_window(app: tauri::AppHandle) {
    system::window::close_suspend_window(&app);
}

// ==================== 北向资金 Command ====================

/// 获取北向资金缓存数据
#[tauri::command]
fn get_northbound_cache(
    days: u32,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<market::northbound_cache::NorthboundFlow>, String> {
    state.db.get_northbound_recent(days).map_err(|e| e.to_string())
}

// ==================== PEG看板 Commands ====================

/// PEG看板单项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PegBoardItem {
    pub secid: String,
    pub code: String,
    pub name: String,
    pub pe_ttm: f64,
    pub pb: f64,
    pub peg: Option<f64>,
    pub peg_rating: String,
    pub cagr: Option<f64>,
    pub price: f64,
    pub change_percent: f64,
}

/// 行业对比结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryComparisonResult {
    pub secid: String,
    pub stock_name: String,
    pub stock_pe: f64,
    pub industry_name: String,
    pub industry_avg_pe: f64,
    pub industry_median_pe: f64,
    pub industry_pe_percentile: f64,
    pub peers: Vec<PeerStock>,
}

/// 同行业股票
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerStock {
    pub secid: String,
    pub name: String,
    pub pe_ttm: f64,
    pub pb: f64,
    pub market_cap: f64,
}

/// 批量获取PEG看板数据
#[tauri::command]
async fn fetch_peg_board(
    secids: Option<Vec<String>>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<PegBoardItem>, String> {
    let target_secids = secids.unwrap_or_else(|| {
        // 默认从自选股获取
        let groups = state.db.get_watchlist_groups().unwrap_or_default();
        groups.iter().flat_map(|g| {
            state.db.get_watchlist_stocks(g.id).unwrap_or_default()
        }).map(|s| s.secid).collect()
    });

    if target_secids.is_empty() {
        return Ok(vec![]);
    }

    // 获取实时行情
    let quotes = state.source.fetch_quotes(&target_secids).await.unwrap_or_default();

    let items: Vec<PegBoardItem> = quotes.iter().filter_map(|q| {
        // 计算PEG（假设使用10%作为默认CAGR，实际应从财务数据获取）
        let peg = if q.pe_ttm > 0.0 {
            let cagr = 10.0; // 默认增长率10%，实际应从数据库或API获取
            let pe_dec = Decimal::from_f64(q.pe_ttm)?;
            let cagr_dec = Decimal::from_f64(cagr)?;
            analysis::peg::PegCalculator::calc_peg(pe_dec, cagr_dec)
                .and_then(|p| p.to_f64())
        } else {
            None
        };

        let peg_rating = peg.map(|p| {
            let p_dec = Decimal::from_f64(p).unwrap_or(Decimal::ZERO);
            analysis::peg::PegRating::from_peg(p_dec).display().to_string()
        }).unwrap_or_else(|| "N/A".to_string());

        Some(PegBoardItem {
            secid: q.secid.clone(),
            code: q.code.clone(),
            name: q.name.clone(),
            pe_ttm: q.pe_ttm,
            pb: q.pb,
            peg,
            peg_rating,
            cagr: Some(10.0), // 默认值
            price: q.price,
            change_percent: q.change_percent,
        })
    }).collect();

    Ok(items)
}

/// 获取行业PE对比数据
#[tauri::command]
async fn fetch_industry_comparison(
    secid: String,
    state: tauri::State<'_, AppState>,
) -> Result<IndustryComparisonResult, String> {
    validate_secid_or_error(&secid)?;

    // 获取目标股票行情
    let quotes = state.source.fetch_quotes(&[secid.clone()]).await?;
    let stock_quote = quotes.first().ok_or("未找到股票数据")?;

    // 构建行业对比数据（实际应从API获取同行业股票）
    // 这里使用模拟数据，实际应接入行业分类API
    let industry_name = "未分类".to_string(); // TODO: 接入行业分类API
    let industry_avg_pe = stock_quote.pe_ttm;
    let industry_median_pe = stock_quote.pe_ttm;
    let industry_pe_percentile = 50.0;

    Ok(IndustryComparisonResult {
        secid: secid.clone(),
        stock_name: stock_quote.name.clone(),
        stock_pe: stock_quote.pe_ttm,
        industry_name,
        industry_avg_pe,
        industry_median_pe,
        industry_pe_percentile,
        peers: vec![], // TODO: 接入同行业股票数据
    })
}

/// 导出分析报告为PDF
#[tauri::command]
async fn export_analysis_pdf(
    analysis_id: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    // 从数据库获取分析结果
    let conn = state.db.conn().map_err(|e| e.to_string())?;
    let result: Option<(String, String, String)> = conn.query_row(
        "SELECT stock_name, overall_rating, dimensions_json FROM analysis_results WHERE id = ?1",
        rusqlite::params![analysis_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).ok();

    let (stock_name, overall_rating, dimensions_json) = result.ok_or("分析结果不存在")?;

    // 生成Markdown报告
    let markdown_report = analysis::report::generate_report_from_json(&serde_json::json!({
        "stock_name": stock_name,
        "overall_rating": overall_rating,
        "overall_score": 0.0,
        "bull_argument": "",
        "bear_argument": "",
        "verdict": "",
        "quality_grade": "",
        "dimensions": serde_json::from_str::<serde_json::Value>(&dimensions_json).unwrap_or_default()
    }).to_string());

    // 保存为文件（实际应转换为PDF）
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    let file_name = format!("analysis_{}_{}.md", stock_name, chrono::Local::now().format("%Y%m%d%H%M%S"));
    let file_path = app_dir.join(&file_name);

    std::fs::write(&file_path, markdown_report).map_err(|e| e.to_string())?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Scheduler 控制句柄（用于优雅退出）
pub struct SchedulerHandle {
    pub cancel_token: tokio_util::sync::CancellationToken,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // 初始化日志（try_init 避免重复初始化 panic）
            let _ = tracing_subscriber::fmt::try_init();

            // 初始化数据库
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join(config::constants::DB_NAME);
            let pool = db::DbPool::open(&db_path)?;
            db::migrations::run(&pool)?;

            // 构建多数据源管理器（按优先级排序）
            let sources: Vec<Arc<dyn MarketDataSource>> = vec![
                Arc::new(market::sources::eastmoney::EastMoneySource::new()),
                Arc::new(market::sources::tencent::TencentSource::new()),
                Arc::new(market::sources::ths::ThsSource::new()),
                Arc::new(market::sources::sina::SinaSource::new()),
                Arc::new(market::sources::fund::FundSource::new()),
                Arc::new(market::sources::dzh::DzhSource::new()),
            ];
            let source = Arc::new(DataSourceManager::new(sources));

            // 构建共享 HealthCollector（scheduler + AppState 共用）
            let health = Arc::new(Mutex::new(HealthCollector::new()));
            let db_arc = Arc::new(pool.clone());

            // 注册应用级共享状态
            let app_state = AppState {
                db: pool,
                source: source.clone(),
                health: health.clone(),
            };
            app.manage(app_state);

            // 启动行情调度器（共享 DataSourceManager + HealthCollector）
            let cancel_token = tokio_util::sync::CancellationToken::new();
            let scheduler = market::scheduler::MarketScheduler::new(
                app.handle().clone(),
                db_arc,
                source,
                health,
                cancel_token.clone(),
            );
            // 保存 cancel_token 供退出时使用
            app.manage(SchedulerHandle { cancel_token });

            // 后台启动调度器
            tauri::async_runtime::spawn(async move {
                let is_trading_day = true; // TODO: 接入交易日历
                loop {
                    scheduler.run(is_trading_day).await;
                }
            });

            // 初始化系统托盘
            if let Err(e) = system::tray::init_tray(app) {
                tracing::warn!("托盘初始化失败: {}", e);
            }

            // 注册全局快捷键
            if let Err(e) = system::hotkey::register_hotkeys(app) {
                tracing::warn!("快捷键注册失败: {:?}", e);
            }

            tracing::info!("TickPulse启动成功，数据库路径: {:?}", db_path);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 自选股
            get_watchlist_groups,
            create_watchlist_group,
            delete_watchlist_group,
            rename_watchlist_group,
            get_watchlist_stocks,
            add_watchlist_stock,
            remove_watchlist_stock,
            toggle_pin_watchlist_stock,
            // 持仓
            get_positions,
            add_position,
            update_position,
            delete_position,
            get_portfolio_summary,
            // 配置
            get_settings,
            update_setting,
            // 行情
            search_stock,
            fetch_quotes,
            fetch_kline,
            fetch_timeline,
            fetch_exrights,
            get_health_metrics,
            // 预警
            get_alert_rules,
            add_alert_rule,
            remove_alert_rule,
            toggle_alert_rule,
            reset_daily_alerts,
            // 大宗交易
            fetch_block_trades,
            // AI 分析引擎
            analyze_stock,
            get_analysis_history,
            get_llm_config,
            save_llm_config,
            get_analysis_presets,
            calc_peg,
            calc_cagr,
            delete_analysis_result,
            get_analysis_progress,
            // PEG看板
            fetch_peg_board,
            fetch_industry_comparison,
            export_analysis_pdf,
            // 系统功能
            is_autostart_enabled,
            set_autostart,
            open_suspend_window,
            close_suspend_window,
            // 北向资金
            get_northbound_cache,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
