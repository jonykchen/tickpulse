//! 东方财富统一限流器
//! 串行限流（≥1s）+ 随机抖动（0.1-0.5s）+ Keep-Alive 会话复用

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// 东方财富请求限流器
pub struct EmRateLimiter {
    /// 上次请求时间
    last_request: Arc<Mutex<Instant>>,
    /// 最小请求间隔
    min_interval: Duration,
    /// 抖动范围（秒）
    jitter_range: (f64, f64),
}

impl EmRateLimiter {
    pub fn new() -> Self {
        Self {
            last_request: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(10))),
            min_interval: Duration::from_secs(1),
            jitter_range: (0.1, 0.5),
        }
    }

    /// 自定义参数创建
    pub fn with_config(min_interval_secs: f64, jitter_min: f64, jitter_max: f64) -> Self {
        Self {
            last_request: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(10))),
            min_interval: Duration::from_secs_f64(min_interval_secs),
            jitter_range: (jitter_min, jitter_max),
        }
    }

    /// 等待限流窗口
    /// 确保两次请求间隔 ≥ min_interval + 随机抖动
    pub async fn acquire(&self) {
        let mut last = self.last_request.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last);

        // 计算随机抖动（0.1-0.5s 之间）
        // 使用简单的时间戳作为伪随机种子
        let jitter_secs = self.jitter_range.0
            + (now.elapsed().as_nanos() as f64 % (self.jitter_range.1 - self.jitter_range.0));

        let required_interval = self.min_interval + Duration::from_secs_f64(jitter_secs);

        if elapsed < required_interval {
            let wait = required_interval - elapsed;
            tokio::time::sleep(wait).await;
        }

        *last = Instant::now();
    }
}

impl Default for EmRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}
