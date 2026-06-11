/// 请求限速器
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

/// 令牌桶限速器
pub struct RateLimiter {
    buckets: Mutex<HashMap<String, TokenBucket>>,
    default_rate: f64,       // 每秒令牌数
    default_burst: u32,      // 桶容量
}

struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    last_refill: Instant,
    rate: f64,
}

impl RateLimiter {
    pub fn new(default_rate: f64, default_burst: u32) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            default_rate,
            default_burst,
        }
    }

    /// 尝试获取一个令牌
    pub fn try_acquire(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().unwrap();
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket {
                tokens: self.default_burst as f64,
                max_tokens: self.default_burst as f64,
                last_refill: Instant::now(),
                rate: self.default_rate,
            });

        // 补充令牌
        let elapsed = bucket.last_refill.elapsed().as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * bucket.rate).min(bucket.max_tokens);
        bucket.last_refill = Instant::now();

        // 尝试消费
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows() {
        let limiter = RateLimiter::new(10.0, 5);
        assert!(limiter.try_acquire("test"));
    }

    #[test]
    fn test_rate_limiter_burst() {
        let limiter = RateLimiter::new(1.0, 3);
        assert!(limiter.try_acquire("test"));
        assert!(limiter.try_acquire("test"));
        assert!(limiter.try_acquire("test"));
        // 桶空了
        assert!(!limiter.try_acquire("test"));
    }
}
