//! 开机自启
//! 使用 tauri-plugin-autostart 管理

use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

/// 启用开机自启
pub fn enable_autostart(app: &tauri::AppHandle) -> Result<(), tauri_plugin_autostart::Error> {
    app.autolaunch().enable()
}

/// 禁用开机自启
pub fn disable_autostart(app: &tauri::AppHandle) -> Result<(), tauri_plugin_autostart::Error> {
    app.autolaunch().disable()
}

/// 查询开机自启状态
pub fn is_autostart_enabled(app: &tauri::AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}
