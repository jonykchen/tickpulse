//! 同花顺数据源
//! 热门股/涨跌分布/北向资金/一致预期

use crate::config::constants;
use crate::market::{
    AdjustType, ExRightInfo, KlineBar, KlinePeriod, MarketDataSource, SearchResult, StockQuote,
    TimelineData,
};
use async_trait::async_trait;

/// 同花顺数据源
pub struct ThsSource {
    client: reqwest::Client,
}

impl ThsSource {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(constants::HTTP_TIMEOUT_SECS))
            .gzip(true)
            .build()
            .expect("Failed to build THS HTTP client");
        Self { client }
    }
}

#[async_trait]
impl MarketDataSource for ThsSource {
    async fn fetch_quotes(&self, secids: &[String]) -> Result<Vec<StockQuote>, String> {
        // 同花顺不直接提供批量行情接口，返回空让主数据源处理
        Err("同花顺不提供批量行情API".to_string())
    }

    async fn fetch_kline(
        &self,
        _secid: &str,
        _period: KlinePeriod,
        _limit: u32,
        _adjust: AdjustType,
    ) -> Result<Vec<KlineBar>, String> {
        Err("同花顺K线暂未实现".to_string())
    }

    async fn fetch_timeline(&self, _secid: &str) -> Result<TimelineData, String> {
        Err("同花顺分时暂未实现".to_string())
    }

    async fn search(&self, _keyword: &str) -> Result<Vec<SearchResult>, String> {
        Err("同花顺搜索暂未实现".to_string())
    }

    async fn fetch_exrights(&self, _secid: &str) -> Result<Vec<ExRightInfo>, String> {
        Err("同花顺除权暂未实现".to_string())
    }

    fn name(&self) -> &'static str {
        "同花顺"
    }

    fn priority(&self) -> u8 {
        3 // 东方财富(0)之后
    }
}
