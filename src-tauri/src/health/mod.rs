//! 健康诊断模块
//! 使用 VecDeque 环形缓冲 + sysinfo 获取真实 RSS + 追踪 emit 延迟

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::Instant;

/// 健康诊断指标（对齐 S17 文档）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// 上次刷新耗时(ms)
    pub last_refresh_ms: u64,
    /// 平均刷新耗时(ms)（最近 100 次滑动窗口）
    pub avg_refresh_ms: u64,
    /// 事件推送延迟(ms)
    pub emit_latency_ms: u64,
    /// 断路器状态（各数据源）
    pub circuit_breaker_status: Vec<CircuitBreakerStatus>,
    /// 进程内存 RSS(MB)
    pub memory_rss_mb: f64,
    /// 数据库大小(MB)
    pub db_size_mb: f64,
    /// 应用运行时长(秒)
    pub uptime_secs: u64,
}

/// 单个数据源断路器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStatus {
    pub source_name: String,
    pub state: String,
    pub consecutive_failures: u32,
}

impl HealthMetrics {
    pub fn new() -> Self {
        Self {
            last_refresh_ms: 0,
            avg_refresh_ms: 0,
            emit_latency_ms: 0,
            circuit_breaker_status: vec![CircuitBreakerStatus {
                source_name: "东方财富".to_string(),
                state: "Closed".to_string(),
                consecutive_failures: 0,
            }],
            memory_rss_mb: 0.0,
            db_size_mb: 0.0,
            uptime_secs: 0,
        }
    }
}

/// 滑动窗口大小
const MAX_REFRESH_SAMPLES: usize = 100;
const MAX_EMIT_SAMPLES: usize = 100;

/// 健康诊断收集器
/// - VecDeque 环形缓冲，O(1) push/pop
/// - sysinfo 获取真实 RSS
/// - 追踪 emit 延迟（滑动窗口平均）
pub struct HealthCollector {
    start_time: Instant,
    refresh_times: VecDeque<u64>,
    last_refresh_ms: u64,
    emit_times: VecDeque<u64>,
    cached_circuit_breakers: Vec<CircuitBreakerStatus>,
    pid: sysinfo::Pid,
}

impl HealthCollector {
    pub fn new() -> Self {
        let pid = sysinfo::Pid::from(std::process::id() as usize);
        Self {
            start_time: Instant::now(),
            refresh_times: VecDeque::with_capacity(MAX_REFRESH_SAMPLES),
            last_refresh_ms: 0,
            emit_times: VecDeque::with_capacity(MAX_EMIT_SAMPLES),
            cached_circuit_breakers: vec![],
            pid,
        }
    }

    /// 记录一次刷新耗时
    pub fn record_refresh(&mut self, elapsed_ms: u64) {
        self.last_refresh_ms = elapsed_ms;
        if self.refresh_times.len() >= MAX_REFRESH_SAMPLES {
            self.refresh_times.pop_front();
        }
        self.refresh_times.push_back(elapsed_ms);
    }

    /// 记录事件推送延迟（滑动窗口）
    pub fn record_emit(&mut self, elapsed_ms: u64) {
        if self.emit_times.len() >= MAX_EMIT_SAMPLES {
            self.emit_times.pop_front();
        }
        self.emit_times.push_back(elapsed_ms);
    }

    /// 更新断路器状态缓存
    pub fn update_circuit_breakers(&mut self, status: Vec<CircuitBreakerStatus>) {
        self.cached_circuit_breakers = status;
    }

    /// 获取平均刷新耗时（滑动窗口）
    pub fn avg_refresh_ms(&self) -> u64 {
        if self.refresh_times.is_empty() {
            return 0;
        }
        self.refresh_times.iter().sum::<u64>() / self.refresh_times.len() as u64
    }

    /// 获取平均 emit 延迟（滑动窗口）
    pub fn avg_emit_ms(&self) -> u64 {
        if self.emit_times.is_empty() {
            return 0;
        }
        self.emit_times.iter().sum::<u64>() / self.emit_times.len() as u64
    }

    /// 获取当前进程 RSS（Resident Set Size），单位 MB
    fn get_process_rss_mb(&self) -> f64 {
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[self.pid]));
        if let Some(process) = sys.process(self.pid) {
            process.memory() as f64 / (1024.0 * 1024.0)
        } else {
            0.0
        }
    }

    /// 生成健康诊断报告
    pub fn collect(
        &self,
        db_size_bytes: u64,
        circuit_breakers: Vec<CircuitBreakerStatus>,
    ) -> HealthMetrics {
        // 1. 计算 RSS
        let memory_rss_mb = self.get_process_rss_mb();

        // 2. 计算平均 emit 延迟
        let emit_latency_ms = self.avg_emit_ms();

        // 3. 组装结果（优先使用传入的断路器状态，否则使用缓存的）
        let cb_status = if circuit_breakers.is_empty() {
            self.cached_circuit_breakers.clone()
        } else {
            circuit_breakers
        };

        HealthMetrics {
            last_refresh_ms: self.last_refresh_ms,
            avg_refresh_ms: self.avg_refresh_ms(),
            emit_latency_ms,
            circuit_breaker_status: cb_status,
            memory_rss_mb,
            db_size_mb: db_size_bytes as f64 / (1024.0 * 1024.0),
            uptime_secs: self.start_time.elapsed().as_secs(),
        }
    }
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for HealthCollector {
    fn default() -> Self {
        Self::new()
    }
}
