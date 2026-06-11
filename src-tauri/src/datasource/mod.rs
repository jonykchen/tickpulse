/// 数据源管理（断路器+限流+自愈）
pub mod rate_limiter;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 断路器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// 断路器
pub struct CircuitBreaker {
    state: std::sync::atomic::AtomicU8,
    failure_count: std::sync::atomic::AtomicU32,
    threshold: u32,
    reset_timeout: Duration,
    last_failure: std::sync::Mutex<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            state: std::sync::atomic::AtomicU8::new(0), // Closed
            failure_count: std::sync::atomic::AtomicU32::new(0),
            threshold,
            reset_timeout,
            last_failure: std::sync::Mutex::new(None),
        }
    }

    pub fn is_available(&self) -> bool {
        let state = self.state.load(std::sync::atomic::Ordering::Relaxed);
        match state {
            0 => true,     // Closed
            1 => {         // Open - 检查是否超时
                let last = self.last_failure.lock().unwrap();
                if let Some(t) = *last {
                    if t.elapsed() >= self.reset_timeout {
                        self.state.store(2, std::sync::atomic::Ordering::Relaxed); // HalfOpen
                        return true;
                    }
                }
                false
            }
            2 => true,     // HalfOpen
            _ => true,
        }
    }

    pub fn record_success(&self) {
        self.failure_count.store(0, std::sync::atomic::Ordering::Relaxed);
        self.state.store(0, std::sync::atomic::Ordering::Relaxed); // Closed
    }

    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        if count >= self.threshold {
            self.state.store(1, std::sync::atomic::Ordering::Relaxed); // Open
            *self.last_failure.lock().unwrap() = Some(Instant::now());
        }
    }

    pub fn state(&self) -> CircuitState {
        match self.state.load(std::sync::atomic::Ordering::Relaxed) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }
}

/// 数据源管理器
pub struct DataSourceManager {
    sources: Vec<Arc<dyn crate::market::MarketDataSource>>,
    breakers: DashMap<String, CircuitBreaker>,
}

impl DataSourceManager {
    pub fn new(sources: Vec<Arc<dyn crate::market::MarketDataSource>>) -> Self {
        let breakers = DashMap::new();
        for source in &sources {
            breakers.insert(
                source.name().to_string(),
                CircuitBreaker::new(5, Duration::from_secs(60)),
            );
        }
        Self { sources, breakers }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        assert!(cb.is_available());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_opens() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.is_available());
    }

    #[test]
    fn test_circuit_breaker_success_resets() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
