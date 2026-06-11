/// 数据源管理（断路器+限流+自愈+多源容灾）
pub mod rate_limiter;

use crate::config::constants;
use crate::market::{
    AdjustType, ExRightInfo, KlineBar, KlinePeriod, MarketDataSource, StockQuote, TimelineData,
};
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
/// 按优先级尝试多个数据源，断路器保护，限速器控制请求频率
pub struct DataSourceManager {
    sources: Vec<Arc<dyn MarketDataSource>>,
    breakers: DashMap<String, CircuitBreaker>,
    rate_limiter: Arc<rate_limiter::RateLimiter>,
}

impl DataSourceManager {
    pub fn new(sources: Vec<Arc<dyn MarketDataSource>>) -> Self {
        let breakers = DashMap::new();
        for source in &sources {
            breakers.insert(
                source.name().to_string(),
                CircuitBreaker::new(5, Duration::from_secs(60)),
            );
        }
        // 默认限速：每秒5次请求，突发10次
        let rate_limiter = Arc::new(rate_limiter::RateLimiter::new(5.0, 10));
        Self { sources, breakers, rate_limiter }
    }

    /// 按优先级尝试获取行情，断路器保护 + 限速器
    pub async fn fetch_quotes(&self, secids: &[String]) -> Result<Vec<StockQuote>, String> {
        if secids.is_empty() {
            return Ok(Vec::new());
        }

        for source in &self.sources {
            let source_name = source.name();

            // 检查断路器是否放行
            if let Some(breaker) = self.breakers.get(source_name) {
                if !breaker.is_available() {
                    tracing::warn!("数据源 {} 断路器打开，跳过", source_name);
                    continue;
                }
            }

            // 限速器检查
            if !self.rate_limiter.try_acquire(source_name) {
                tracing::debug!("数据源 {} 限速，跳过本次请求", source_name);
                continue;
            }

            match source.fetch_quotes(secids).await {
                Ok(quotes) => {
                    // 记录成功
                    if let Some(breaker) = self.breakers.get(source_name) {
                        breaker.record_success();
                    }
                    return Ok(quotes);
                }
                Err(e) => {
                    // 记录失败
                    if let Some(breaker) = self.breakers.get(source_name) {
                        breaker.record_failure();
                    }
                    tracing::warn!("数据源 {} 请求失败: {}", source_name, e);
                    continue;
                }
            }
        }

        Err("所有数据源均不可用".to_string())
    }

    /// 并行分批请求（200只→4批×50并行）
    pub async fn fetch_quotes_parallel(&self, secids: &[String]) -> Result<Vec<StockQuote>, String> {
        if secids.is_empty() {
            return Ok(Vec::new());
        }

        let batch_size = constants::BATCH_QUOTE_SIZE; // 50
        let chunks: Vec<_> = secids.chunks(batch_size).collect();

        // 并行请求各批次
        let mut handles = Vec::new();
        for chunk in chunks {
            let secids_chunk: Vec<String> = chunk.to_vec();
            let sources = self.sources.clone();
            let breakers = self.breakers.clone();
            let rate_limiter = self.rate_limiter.clone();

            // 每个 chunk 独立执行 fetch
            let handle = tokio::spawn(async move {
                // 简化：直接用第一个可用数据源
                for source in &sources {
                    let source_name = source.name();
                    if let Some(breaker) = breakers.get(source_name) {
                        if !breaker.is_available() {
                            continue;
                        }
                    }
                    if !rate_limiter.try_acquire(source_name) {
                        continue;
                    }
                    match source.fetch_quotes(&secids_chunk).await {
                        Ok(quotes) => {
                            if let Some(breaker) = breakers.get(source_name) {
                                breaker.record_success();
                            }
                            return Ok(quotes);
                        }
                        Err(e) => {
                            if let Some(breaker) = breakers.get(source_name) {
                                breaker.record_failure();
                            }
                            tracing::warn!("并行批次: 数据源 {} 失败: {}", source_name, e);
                            continue;
                        }
                    }
                }
                Err("并行批次: 所有数据源均不可用".to_string())
            });
            handles.push(handle);
        }

        // 收集结果
        let mut all_quotes = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(quotes)) => all_quotes.extend(quotes),
                Ok(Err(e)) => tracing::error!("并行批次失败: {}", e),
                Err(e) => tracing::error!("并行批次任务异常: {}", e),
            }
        }

        Ok(all_quotes)
    }

    /// 按优先级获取K线
    pub async fn fetch_kline(
        &self,
        secid: &str,
        period: KlinePeriod,
        limit: u32,
        adjust: AdjustType,
    ) -> Result<Vec<KlineBar>, String> {
        for source in &self.sources {
            let source_name = source.name();
            if let Some(breaker) = self.breakers.get(source_name) {
                if !breaker.is_available() {
                    continue;
                }
            }
            match source.fetch_kline(secid, period, limit, adjust).await {
                Ok(bars) => {
                    if let Some(breaker) = self.breakers.get(source_name) {
                        breaker.record_success();
                    }
                    return Ok(bars);
                }
                Err(e) => {
                    if let Some(breaker) = self.breakers.get(source_name) {
                        breaker.record_failure();
                    }
                    tracing::warn!("K线: 数据源 {} 失败: {}", source_name, e);
                    continue;
                }
            }
        }
        Err("所有数据源均不可用(K线)".to_string())
    }

    /// 按优先级获取分时走势
    pub async fn fetch_timeline(&self, secid: &str) -> Result<TimelineData, String> {
        for source in &self.sources {
            let source_name = source.name();
            if let Some(breaker) = self.breakers.get(source_name) {
                if !breaker.is_available() {
                    continue;
                }
            }
            match source.fetch_timeline(secid).await {
                Ok(data) => {
                    if let Some(breaker) = self.breakers.get(source_name) {
                        breaker.record_success();
                    }
                    return Ok(data);
                }
                Err(e) => {
                    if let Some(breaker) = self.breakers.get(source_name) {
                        breaker.record_failure();
                    }
                    tracing::warn!("分时: 数据源 {} 失败: {}", source_name, e);
                    continue;
                }
            }
        }
        Err("所有数据源均不可用(分时)".to_string())
    }

    /// 按优先级获取除权除息
    pub async fn fetch_exrights(&self, secid: &str) -> Result<Vec<ExRightInfo>, String> {
        for source in &self.sources {
            let source_name = source.name();
            if let Some(breaker) = self.breakers.get(source_name) {
                if !breaker.is_available() {
                    continue;
                }
            }
            match source.fetch_exrights(secid).await {
                Ok(data) => {
                    if let Some(breaker) = self.breakers.get(source_name) {
                        breaker.record_success();
                    }
                    return Ok(data);
                }
                Err(e) => {
                    if let Some(breaker) = self.breakers.get(source_name) {
                        breaker.record_failure();
                    }
                    tracing::warn!("除权: 数据源 {} 失败: {}", source_name, e);
                    continue;
                }
            }
        }
        Err("所有数据源均不可用(除权)".to_string())
    }

    /// 获取所有断路器状态
    pub fn circuit_breaker_status(&self) -> Vec<(String, CircuitState)> {
        self.breakers
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().state()))
            .collect()
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

    #[test]
    fn test_circuit_breaker_half_open() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(100));
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.is_available());

        // 等待超时后应变为 HalfOpen
        std::thread::sleep(Duration::from_millis(150));
        assert!(cb.is_available());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }
}
