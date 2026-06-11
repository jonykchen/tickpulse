pub mod buffer;

use std::path::Path;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
use crate::logging::buffer::LogBufferLayer;

/// 初始化日志系统
/// - 控制台输出（带颜色）
/// - 文件输出（按天滚动，保留7天）
/// - 内存缓冲区（用于前端实时查看）
pub fn init_logging(app_dir: &Path, log_layer: LogBufferLayer) {
    let log_dir = app_dir.join("logs");
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory: {}", e);
    }

    // 文件输出（按天滚动）
    let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .max_log_files(7)
        .filename_prefix("tickpulse")
        .filename_suffix("log")
        .build(&log_dir);

    match file_appender {
        Ok(file) => {
            let result = tracing_subscriber::registry()
                .with(
                    EnvFilter::from_default_env()
                        .add_directive("info".parse().unwrap_or_default())
                )
                .with(fmt::layer().with_writer(std::io::stdout).with_ansi(true))
                .with(fmt::layer().with_writer(file).with_ansi(false))
                .with(log_layer)
                .try_init();

            if result.is_err() {
                // 回退到简单初始化
                let _ = tracing_subscriber::fmt::try_init();
            }
        }
        Err(_) => {
            // 文件创建失败，仅使用控制台和缓冲区
            let _ = tracing_subscriber::registry()
                .with(EnvFilter::from_default_env().add_directive("info".parse().unwrap_or_default()))
                .with(fmt::layer().with_writer(std::io::stdout).with_ansi(true))
                .with(log_layer)
                .try_init();
        }
    }
}