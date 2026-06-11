//! 全局快捷键
//! CmdOrCtrl+Shift+S: 切换主窗口显示/隐藏
//! CmdOrCtrl+Shift+D: 打开悬浮窗

use tauri::{App, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// 注册全局快捷键
pub fn register_hotkeys(app: &App) -> Result<(), tauri::plugin::PluginError> {
    let shortcut = app.global_shortcut();

    // CmdOrCtrl+Shift+S: 切换主窗口
    let toggle_shortcut = Shortcut::new(
        Some(Modifiers::SUPER | Modifiers::SHIFT),
        Code::KeyS,
    );
    shortcut.on_shortcut(toggle_shortcut, move |app, _event, kind| {
        if kind != ShortcutState::Pressed {
            return;
        }
        if let Some(window) = app.get_webview_window("main") {
            if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
            } else {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    })?;

    // CmdOrCtrl+Shift+D: 打开悬浮窗
    let float_shortcut = Shortcut::new(
        Some(Modifiers::SUPER | Modifiers::SHIFT),
        Code::KeyD,
    );
    shortcut.on_shortcut(float_shortcut, move |app, _event, kind| {
        if kind != ShortcutState::Pressed {
            return;
        }
        super::window::open_suspend_window(app);
    })?;

    Ok(())
}
