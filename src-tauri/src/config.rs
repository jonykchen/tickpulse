/// 应用配置常量
pub mod constants {
    /// 应用名称
    pub const APP_NAME: &str = "TickPulse";

    /// 数据库文件名
    pub const DB_NAME: &str = "stock-monitor.db";

    /// 默认行情刷新间隔（秒）
    pub const DEFAULT_REFRESH_INTERVAL_SECS: u64 = 10;

    /// 批量行情请求最大数量
    pub const BATCH_QUOTE_SIZE: usize = 50;

    /// 东方财富 API 基础 URL
    pub const EASTMONEY_BASE_URL: &str = "https://push2delay.eastmoney.com";

    /// 东方财富历史数据 API 基础 URL
    pub const EASTMONEY_HISTORY_URL: &str = "https://push2his.eastmoney.com";

    /// 东方财富搜索 API 基础 URL
    pub const EASTMONEY_SEARCH_URL: &str = "https://searchapi.eastmoney.com";

    /// 东方财富交易日历 API
    pub const EASTMONEY_CALENDAR_URL: &str =
        "https://push2his.eastmoney.com/api/qt/stock/tradeCalendar/get";

    /// HTTP 请求超时（秒）
    pub const HTTP_TIMEOUT_SECS: u64 = 10;

    /// 连续错误阈值（触发重建 HTTP 客户端）
    pub const CONSECUTIVE_ERROR_THRESHOLD: u32 = 10;

    /// 最大重试退避秒数
    pub const MAX_BACKOFF_SECS: u64 = 30;

    /// 大智慧涨停天梯 API
    pub const DZH_ZTTT_URL: &str = "https://webrelease.dzh.com.cn:8088/api/zttt";
}

// ==================== 安全校验 ====================

use once_cell::sync::Lazy;
use regex::Regex;

/// secid 正则校验：市场编号.股票代码
/// 格式：1-3位数字.1-10位字母数字（如 "1.600519", "0.000001", "0.399006"）
pub static SECID_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\d{1,3}\.\w{1,10}$").expect("Invalid SECID_PATTERN regex"));

/// 校验 secid 格式
pub fn validate_secid(secid: &str) -> bool {
    SECID_PATTERN.is_match(secid)
}

/// 校验 secid 并返回错误信息
pub fn validate_secid_or_error(secid: &str) -> Result<(), String> {
    if validate_secid(secid) {
        Ok(())
    } else {
        Err(format!("非法证券ID格式: '{}', 期望格式: 市场编号.股票代码 (如 1.600519)", secid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_secids() {
        assert!(validate_secid("1.600519"));
        assert!(validate_secid("0.000001"));
        assert!(validate_secid("0.399006"));
        assert!(validate_secid("116.HK00700"));
        assert!(validate_secid("1.688981"));
    }

    #[test]
    fn test_invalid_secids() {
        assert!(!validate_secid(""));
        assert!(!validate_secid("600519"));
        assert!(!validate_secid("1."));
        assert!(!validate_secid(".600519"));
        assert!(!validate_secid("1.600519<script>"));
        assert!(!validate_secid("'; DROP TABLE positions;--"));
    }
}
