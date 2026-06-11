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
}
