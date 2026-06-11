use crate::market::MarketDataSource;
use tauri::Manager;

mod adjust;
mod alert;
mod anomaly;
mod cache;
mod config;
mod datasource;
mod db;
mod health;
mod indicator;
mod limit;
mod market;
mod system;

// ==================== Tauri 命令 ====================

/// 获取自选股分组
#[tauri::command]
fn get_watchlist_groups(state: tauri::State<'_, db::DbPool>) -> Result<Vec<db::watchlist::WatchlistGroup>, String> {
    state.get_watchlist_groups().map_err(|e| e.to_string())
}

/// 创建自选股分组
#[tauri::command]
fn create_watchlist_group(name: &str, state: tauri::State<'_, db::DbPool>) -> Result<i64, String> {
    state.create_watchlist_group(name).map_err(|e| e.to_string())
}

/// 删除自选股分组
#[tauri::command]
fn delete_watchlist_group(id: i64, state: tauri::State<'_, db::DbPool>) -> Result<(), String> {
    state.delete_watchlist_group(id).map_err(|e| e.to_string())
}

/// 重命名分组
#[tauri::command]
fn rename_watchlist_group(id: i64, name: &str, state: tauri::State<'_, db::DbPool>) -> Result<(), String> {
    state.rename_watchlist_group(id, name).map_err(|e| e.to_string())
}

/// 获取分组下的自选股
#[tauri::command]
fn get_watchlist_stocks(group_id: i64, state: tauri::State<'_, db::DbPool>) -> Result<Vec<db::watchlist::WatchlistStock>, String> {
    state.get_watchlist_stocks(group_id).map_err(|e| e.to_string())
}

/// 添加自选股
#[tauri::command]
fn add_watchlist_stock(group_id: i64, secid: &str, name: Option<&str>, state: tauri::State<'_, db::DbPool>) -> Result<i64, String> {
    state.add_watchlist_stock(group_id, secid, name).map_err(|e| e.to_string())
}

/// 删除自选股
#[tauri::command]
fn remove_watchlist_stock(id: i64, state: tauri::State<'_, db::DbPool>) -> Result<(), String> {
    state.remove_watchlist_stock(id).map_err(|e| e.to_string())
}

/// 置顶/取消置顶自选股
#[tauri::command]
fn toggle_pin_watchlist_stock(id: i64, pinned: bool, state: tauri::State<'_, db::DbPool>) -> Result<(), String> {
    state.toggle_pin_watchlist_stock(id, pinned).map_err(|e| e.to_string())
}

/// 获取持仓列表
#[tauri::command]
fn get_positions(group_id: i64, state: tauri::State<'_, db::DbPool>) -> Result<Vec<db::position::Position>, String> {
    state.get_positions(group_id).map_err(|e| e.to_string())
}

/// 添加持仓
#[tauri::command]
fn add_position(
    group_id: i64,
    secid: &str,
    name: Option<&str>,
    cost_price: &str,
    quantity: &str,
    state: tauri::State<'_, db::DbPool>,
) -> Result<i64, String> {
    state.add_position(group_id, secid, name, cost_price, quantity).map_err(|e| e.to_string())
}

/// 更新持仓
#[tauri::command]
fn update_position(id: i64, cost_price: &str, quantity: &str, state: tauri::State<'_, db::DbPool>) -> Result<(), String> {
    state.update_position(id, cost_price, quantity).map_err(|e| e.to_string())
}

/// 删除持仓
#[tauri::command]
fn delete_position(id: i64, state: tauri::State<'_, db::DbPool>) -> Result<(), String> {
    state.delete_position(id).map_err(|e| e.to_string())
}

/// 获取配置
#[tauri::command]
fn get_settings(state: tauri::State<'_, db::DbPool>) -> Result<Vec<(String, String)>, String> {
    state.get_all_settings().map_err(|e| e.to_string())
}

/// 更新配置
#[tauri::command]
fn update_setting(key: &str, value: &str, state: tauri::State<'_, db::DbPool>) -> Result<(), String> {
    state.upsert_setting(key, value).map_err(|e| e.to_string())
}

/// 搜索股票
#[tauri::command]
async fn search_stock(keyword: &str) -> Result<Vec<market::SearchResult>, String> {
    let source = market::sources::eastmoney::EastMoneySource::new();
    source.search(keyword).await
}

/// 批量获取行情
#[tauri::command]
async fn fetch_quotes(secids: Vec<String>) -> Result<Vec<market::StockQuote>, String> {
    let source = market::sources::eastmoney::EastMoneySource::new();
    source.fetch_quotes(&secids).await
}

/// 获取K线数据
#[tauri::command]
async fn fetch_kline(
    secid: &str,
    period: &str,
    limit: u32,
    adjust: &str,
) -> Result<Vec<market::KlineBar>, String> {
    let source = market::sources::eastmoney::EastMoneySource::new();
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
    source.fetch_kline(secid, kline_period, limit, adjust_type).await
}

/// 获取分时走势
#[tauri::command]
async fn fetch_timeline(secid: &str) -> Result<market::TimelineData, String> {
    let source = market::sources::eastmoney::EastMoneySource::new();
    source.fetch_timeline(secid).await
}

/// 获取除权除息
#[tauri::command]
async fn fetch_exrights(secid: &str) -> Result<Vec<market::ExRightInfo>, String> {
    let source = market::sources::eastmoney::EastMoneySource::new();
    source.fetch_exrights(secid).await
}

/// 获取健康诊断
#[tauri::command]
fn get_health_metrics() -> health::HealthMetrics {
    health::HealthMetrics::new()
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
            // 初始化日志
            tracing_subscriber::fmt::init();

            // 初始化数据库
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join(config::constants::DB_NAME);
            let pool = db::DbPool::open(&db_path)?;
            db::migrations::run(&pool)?;
            app.manage(pool);

            tracing::info!("TickPulse启动成功，数据库路径: {:?}", db_path);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_watchlist_groups,
            create_watchlist_group,
            delete_watchlist_group,
            rename_watchlist_group,
            get_watchlist_stocks,
            add_watchlist_stock,
            remove_watchlist_stock,
            toggle_pin_watchlist_stock,
            get_positions,
            add_position,
            update_position,
            delete_position,
            get_settings,
            update_setting,
            search_stock,
            fetch_quotes,
            fetch_kline,
            fetch_timeline,
            fetch_exrights,
            get_health_metrics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
