/// 按周期 TTL 策略 + 增量更新 + 复权对齐
use serde::{Deserialize, Serialize};

/// K线缓存 TTL 策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CachePolicy {
    /// 缓存周期
    pub period: &'static str,
    /// TTL（秒）
    pub ttl_secs: i64,
}

/// 各周期 TTL 配置
pub const CACHE_POLICIES: &[CachePolicy] = &[
    CachePolicy { period: "1m", ttl_secs: 60 },
    CachePolicy { period: "5m", ttl_secs: 300 },
    CachePolicy { period: "15m", ttl_secs: 300 },
    CachePolicy { period: "30m", ttl_secs: 300 },
    CachePolicy { period: "60m", ttl_secs: 600 },
    CachePolicy { period: "day", ttl_secs: 86400 },
    CachePolicy { period: "week", ttl_secs: 86400 * 7 },
    CachePolicy { period: "month", ttl_secs: 86400 * 30 },
];

/// 获取指定周期的 TTL
pub fn get_ttl(period: &str) -> i64 {
    CACHE_POLICIES
        .iter()
        .find(|p| p.period == period)
        .map(|p| p.ttl_secs)
        .unwrap_or(86400)
}

/// 交易时间内缩短日K缓存TTL
pub fn get_ttl_with_market_status(period: &str, is_trading: bool) -> i64 {
    if is_trading && period == "day" {
        300 // 交易时间日K缓存5分钟
    } else {
        get_ttl(period)
    }
}
