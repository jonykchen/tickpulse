use tauri::Manager;

mod config;
mod db;
mod market;

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
            // 初始化数据库
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("stock-monitor.db");
            let pool = db::DbPool::open(&db_path)?;
            db::migrations::run(&pool)?;
            app.manage(pool);

            tracing::info!("TickPulse启动成功，数据库路径: {:?}", db_path);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
