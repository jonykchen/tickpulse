//! 新浪财经数据源
//! K线 fallback / 财报三表

use crate::config::constants;
use crate::market::{
    AdjustType, ExRightInfo, KlineBar, KlinePeriod, MarketDataSource, SearchResult, StockQuote,
    TimelineData,
};
use async_trait::async_trait;

/// 新浪财经数据源
pub struct SinaSource {
    client: reqwest::Client,
}

impl SinaSource {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(constants::HTTP_TIMEOUT_SECS))
            .gzip(true)
            .build()
            .expect("Failed to build Sina HTTP client");
        Self { client }
    }
}

#[async_trait]
impl MarketDataSource for SinaSource {
    async fn fetch_quotes(&self, _secids: &[String]) -> Result<Vec<StockQuote>, String> {
        // 新浪不直接提供批量行情接口
        Err("新浪不提供批量行情API".to_string())
    }

    async fn fetch_kline(
        &self,
        secid: &str,
        period: KlinePeriod,
        limit: u32,
        _adjust: AdjustType,
    ) -> Result<Vec<KlineBar>, String> {
        // 新浪K线 fallback
        // 将东财 secid 格式 "1.600519" 转换为新浪格式 "sh600519"
        let (market, code) = secid.split_once('.').unwrap_or(("1", ""));
        let prefix = if market == "1" { "sh" } else { "sz" };
        let sina_code = format!("{}{}", prefix, code);

        let period_str = match period {
            KlinePeriod::Daily => "daily",
            KlinePeriod::Weekly => "weekly",
            KlinePeriod::Monthly => "monthly",
            _ => "daily", // 新浪不支持分钟级K线
        };

        let url = format!(
            "https://money.finance.sina.com.cn/quotes_service/api/json_v2.php/CN_MarketData.getKLineData?symbol={}&scale={}&ma=no&datalen={}",
            sina_code, period_str, limit
        );

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://finance.sina.com.cn/")
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|e| format!("新浪K线请求失败: {}", e))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("新浪K线解析失败: {}", e))?;

        let mut bars = Vec::new();
        if let Some(arr) = body.as_array() {
            for item in arr {
                let time_str = item.get("day")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let time = chrono::NaiveDate::parse_from_str(time_str, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
                    .unwrap_or(0);

                bars.push(KlineBar {
                    time,
                    open: item.get("open").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                    close: item.get("close").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                    high: item.get("high").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                    low: item.get("low").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                    volume: item.get("volume").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0),
                    amount: 0.0, // 新浪不直接提供成交额
                    change_percent: 0.0, // 需计算
                });
            }
        }

        Ok(bars)
    }

    async fn fetch_timeline(&self, _secid: &str) -> Result<TimelineData, String> {
        Err("新浪分时暂未实现".to_string())
    }

    async fn search(&self, _keyword: &str) -> Result<Vec<SearchResult>, String> {
        Err("新浪搜索暂未实现".to_string())
    }

    async fn fetch_exrights(&self, _secid: &str) -> Result<Vec<ExRightInfo>, String> {
        Err("新浪除权暂未实现".to_string())
    }

    fn name(&self) -> &'static str {
        "新浪财经"
    }

    fn priority(&self) -> u8 {
        4 // 同花顺(3)之后
    }
}
