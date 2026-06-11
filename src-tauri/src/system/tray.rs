//! 系统托盘管理
//! 跨平台适配，macOS tooltip 有长度限制（80字符截断）

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, Manager,
};

/// 托盘 Tooltip 数据摘要
#[derive(Debug, Clone, Default)]
pub struct TrayTooltipData {
    /// 上证指数价格
    pub sh_price: Option<f64>,
    /// 上证指数涨跌幅(%)
    pub sh_change_percent: Option<f64>,
    /// 上涨家数
    pub up_count: i32,
    /// 下跌家数
    pub down_count: i32,
    /// 北向资金净流入(亿)
    pub northbound_inflow: Option<f64>,
    /// 持仓浮动盈亏
    pub position_pnl: Option<String>,
}

impl TrayTooltipData {
    /// 格式化 Tooltip 文本
    /// 格式：
    /// ```
    /// 上证 {price} {change}%
    /// 涨 {up} 跌 {down}
    /// 北向 {northbound}亿
    /// 持仓 {pnl}
    /// ```
    pub fn format_tooltip(&self) -> String {
        let mut lines = Vec::new();

        // 第一行：上证指数
        if let (Some(price), Some(change)) = (self.sh_price, self.sh_change_percent) {
            let change_sign = if change >= 0.0 { "+" } else { "" };
            lines.push(format!("上证 {:.2} {}{:.2}%", price, change_sign, change));
        } else {
            lines.push("上证 --".to_string());
        }

        // 第二行：涨跌家数
        lines.push(format!("涨 {} 跌 {}", self.up_count, self.down_count));

        // 第三行：北向资金
        if let Some(nb) = self.northbound_inflow {
            let nb_sign = if nb >= 0.0 { "+" } else { "" };
            lines.push(format!("北向 {}{:.2}亿", nb_sign, nb));
        } else {
            lines.push("北向 --".to_string());
        }

        // 第四行：持仓盈亏
        if let Some(ref pnl) = self.position_pnl {
            lines.push(format!("持仓 {}", pnl));
        } else {
            lines.push("持仓 --".to_string());
        }

        lines.join("\n")
    }
}

/// 初始化系统托盘
pub fn init_tray(app: &App) -> Result<(), tauri::Error> {
    let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let float_item = MenuItem::with_id(app, "float", "悬浮窗", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &float_item, &quit_item])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("TickPulse")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "float" => {
                super::window::open_suspend_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

/// 更新托盘 Tooltip（简单文本）
/// macOS 有 80 字符限制，自动截断
pub fn update_tray_text(app: &tauri::AppHandle, text: &str) {
    let tooltip = if cfg!(target_os = "macos") {
        // macOS tooltip 限制约 80 字符
        let chars: Vec<char> = text.chars().take(80).collect();
        chars.into_iter().collect::<String>()
    } else {
        text.to_string()
    };

    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(&tooltip));
    }
}

/// 更新托盘 Tooltip（使用结构化数据）
/// 自动处理 macOS 长度限制
pub fn update_tray_with_data(app: &tauri::AppHandle, data: &TrayTooltipData) {
    let text = data.format_tooltip();

    // macOS 限制处理：截断每行或整体
    let tooltip = if cfg!(target_os = "macos") {
        // macOS tooltip 限制约 80 字符，按行截断
        let lines: Vec<&str> = text.lines().take(4).collect();
        let truncated: String = lines.join("\n");
        let chars: Vec<char> = truncated.chars().take(80).collect();
        chars.into_iter().collect()
    } else {
        text
    };

    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(&tooltip));
    }
}
