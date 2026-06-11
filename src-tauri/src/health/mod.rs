/// 健康诊断模块
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub db_size_mb: f64,
    pub api_latency_ms: u64,
    pub consecutive_errors: u32,
    pub last_success_at: Option<i64>,
    pub active_sources: Vec<String>,
    pub cache_hit_rate: f64,
}

impl HealthMetrics {
    pub fn new() -> Self {
        Self {
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            db_size_mb: 0.0,
            api_latency_ms: 0,
            consecutive_errors: 0,
            last_success_at: None,
            active_sources: vec!["东方财富".to_string()],
            cache_hit_rate: 0.0,
        }
    }
}
