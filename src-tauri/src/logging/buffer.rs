use parking_lot::RwLock;
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::Context;

const MAX_LOG_ENTRIES: usize = 1000;

/// 日志条目
#[derive(Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: String,
    pub target: String,
    pub message: String,
}

/// 环形缓冲区存储日志
pub struct LogBuffer {
    entries: RwLock<VecDeque<LogEntry>>,
}

impl LogBuffer {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            entries: RwLock::new(VecDeque::with_capacity(MAX_LOG_ENTRIES)),
        })
    }

    pub fn push(&self, entry: LogEntry) {
        let mut entries = self.entries.write();
        if entries.len() >= MAX_LOG_ENTRIES {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    pub fn get_recent(&self, count: usize) -> Vec<LogEntry> {
        let entries = self.entries.read();
        entries.iter().rev().take(count).rev().cloned().collect()
    }

    pub fn clear(&self) {
        let mut entries = self.entries.write();
        entries.clear();
    }
}

/// 日志缓冲层，用于捕获 tracing 日志
pub struct LogBufferLayer {
    buffer: Arc<LogBuffer>,
}

impl LogBufferLayer {
    pub fn new(buffer: Arc<LogBuffer>) -> Self {
        Self { buffer }
    }

    pub fn buffer(&self) -> Arc<LogBuffer> {
        self.buffer.clone()
    }
}

impl<S: Subscriber> tracing_subscriber::Layer<S> for LogBufferLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = match *metadata.level() {
            Level::TRACE => "TRACE",
            Level::DEBUG => "DEBUG",
            Level::INFO => "INFO",
            Level::WARN => "WARN",
            Level::ERROR => "ERROR",
        };

        // 提取消息
        let mut message = String::new();
        event.record(&mut MessageRecorder(&mut message));

        let entry = LogEntry {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0),
            level: level.to_string(),
            target: metadata.target().to_string(),
            message,
        };

        self.buffer.push(entry);
    }
}

/// 辅助结构，用于提取日志消息
struct MessageRecorder<'a>(&'a mut String);

impl tracing::field::Visit for MessageRecorder<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0.push_str(&format!("{:?}", value));
        } else {
            self.0.push_str(&format!(" {}={:?}", field.name(), value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0.push_str(value);
        } else {
            self.0.push_str(&format!(" {}={}", field.name(), value));
        }
    }
}