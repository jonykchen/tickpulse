//! 窗口管理（主窗口 + 悬浮窗）

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// 打开悬浮窗
/// 220×300 无边框置顶透明窗口
pub fn open_suspend_window(app: &tauri::AppHandle) {
    // 如果已存在则聚焦
    if let Some(window) = app.get_webview_window("suspend") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    let _window = WebviewWindowBuilder::new(
        app,
        "suspend",
        WebviewUrl::App("index.html#/suspend".into()),
    )
    .title("TickPulse - 悬浮窗")
    .inner_size(220.0, 300.0)
    .decorations(false)
    .always_on_top(true)
    .transparent(true)
    .skip_taskbar(true)
    .resizable(false)
    .build()
    .expect("Failed to create suspend window");
}

/// 关闭悬浮窗
pub fn close_suspend_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("suspend") {
        let _ = window.close();
    }
}
