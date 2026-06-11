use crate::config::constants;
use crate::market::{
    AdjustType, ExRightInfo, KlineBar, KlinePeriod, MarketDataSource, SearchResult, StockQuote,
    TimelineData, TimelinePoint,
};
use async_trait::async_trait;

/// 东方财富数据源
pub struct EastMoneySource {
    client: reqwest::Client,
}

impl EastMoneySource {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(constants::HTTP_TIMEOUT_SECS))
            .gzip(true)
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }

    /// 构建行情请求 URL
    fn build_quote_url(secids: &[String]) -> String {
        let fields = "f2,f3,f4,f5,f6,f8,f9,f10,f12,f13,f14,f15,f16,f17,f18,f20,f22,f23,f25,f62,f136";
        let secid_list = secids.join(",");
        format!(
            "{}/api/qt/ulist.np/get?fields={}&secids={}",
            constants::EASTMONEY_BASE_URL, fields, secid_list
        )
    }

    /// 解析行情数据
    fn parse_quotes(data: &serde_json::Value) -> Vec<StockQuote> {
        let mut quotes = Vec::new();

        if let Some(diff) = data.get("data").and_then(|d| d.get("diff")).and_then(|d| d.as_array()) {
            for item in diff {
                let market = item.get("f13").and_then(|v| v.as_i64()).unwrap_or(0);
                let code = item
                    .get("f12")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let secid = format!("{}.{}", market, code);

                let price = item.get("f2").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let pre_close = item.get("f18").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let is_suspended = price <= 0.0 || item.get("f6").and_then(|v| v.as_f64()).unwrap_or(0.0) == 0.0;

                quotes.push(StockQuote {
                    secid,
                    code,
                    name: item
                        .get("f14")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    price: if price < 0.0 { 0.0 } else { price },
                    change: item.get("f4").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    change_percent: item.get("f3").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    volume: item.get("f5").and_then(|v| v.as_i64()).unwrap_or(0),
                    amount: item.get("f6").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    turnover_rate: item.get("f8").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    volume_ratio: item.get("f10").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    high: item.get("f15").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    low: item.get("f16").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    open: item.get("f17").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    pre_close,
                    total_market_cap: item.get("f20").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    pe_ttm: item.get("f9").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    pe_static: item.get("f23").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    pb: item.get("f136").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    change_speed: item.get("f22").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    ytd_change: item.get("f25").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    main_net_inflow: item.get("f62").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    market,
                    is_suspended,
                });
            }
        }

        quotes
    }
}

#[async_trait]
impl MarketDataSource for EastMoneySource {
    async fn fetch_quotes(&self, secids: &[String]) -> Result<Vec<StockQuote>, String> {
        if secids.is_empty() {
            return Ok(Vec::new());
        }

        // 分批请求，每批最多 BATCH_QUOTE_SIZE 只
        let mut all_quotes = Vec::new();
        for chunk in secids.chunks(constants::BATCH_QUOTE_SIZE) {
            let url = Self::build_quote_url(chunk);
            let resp = self
                .client
                .get(&url)
                .header("Referer", "https://www.eastmoney.com/")
                .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
                .send()
                .await
                .map_err(|e| format!("东方财富行情请求失败: {}", e))?;

            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("东方财富行情解析失败: {}", e))?;

            all_quotes.extend(Self::parse_quotes(&body));
        }

        Ok(all_quotes)
    }

    async fn fetch_kline(
        &self,
        secid: &str,
        period: KlinePeriod,
        limit: u32,
        adjust: AdjustType,
    ) -> Result<Vec<KlineBar>, String> {
        let url = format!(
            "{}/api/qt/stock/kline/get?secid={}&fields1=f1,f2,f3,f4,f5,f6&fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61&klt={}&fqt={}&end=20500101&lmt={}",
            constants::EASTMONEY_HISTORY_URL,
            secid,
            period.to_eastmoney_code(),
            adjust.to_fqt(),
            limit
        );

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://www.eastmoney.com/")
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|e| format!("K线请求失败: {}", e))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("K线解析失败: {}", e))?;

        let mut bars = Vec::new();
        if let Some(klines) = body.get("data").and_then(|d| d.get("klines")).and_then(|k| k.as_array()) {
            for kline in klines {
                if let Some(s) = kline.as_str() {
                    let parts: Vec<&str> = s.split(',').collect();
                    if parts.len() >= 11 {
                        let time = chrono::NaiveDate::parse_from_str(parts[0], "%Y-%m-%d")
                            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
                            .unwrap_or(0);
                        bars.push(KlineBar {
                            time,
                            open: parts[1].parse().unwrap_or(0.0),
                            close: parts[2].parse().unwrap_or(0.0),
                            high: parts[3].parse().unwrap_or(0.0),
                            low: parts[4].parse().unwrap_or(0.0),
                            volume: parts[5].parse().unwrap_or(0),
                            amount: parts[6].parse().unwrap_or(0.0),
                            change_percent: parts[8].parse().unwrap_or(0.0),
                        });
                    }
                }
            }
        }

        Ok(bars)
    }

    async fn fetch_timeline(&self, secid: &str) -> Result<TimelineData, String> {
        let url = format!(
            "{}/api/qt/stock/trends2/get?secid={}&fields1=f1,f2,f3,f4,f5,f6,f7,f8,f9,f10,f11,f12,f13&fields2=f51,f52,f53,f54,f55,f56,f57,f58",
            constants::EASTMONEY_BASE_URL,
            secid
        );

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://www.eastmoney.com/")
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|e| format!("分时请求失败: {}", e))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("分时解析失败: {}", e))?;

        let name = body
            .get("data")
            .and_then(|d| d.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let pre_close = body
            .get("data")
            .and_then(|d| d.get("preClose"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let mut points = Vec::new();
        if let Some(trends) = body.get("data").and_then(|d| d.get("trends")).and_then(|t| t.as_array()) {
            for trend in trends {
                if let Some(s) = trend.as_str() {
                    let parts: Vec<&str> = s.split(',').collect();
                    if parts.len() >= 6 {
                        points.push(TimelinePoint {
                            time: parts[0].to_string(),
                            price: parts[1].parse().unwrap_or(0.0),
                            avg_price: parts[3].parse().unwrap_or(0.0),
                            volume: parts[4].parse().unwrap_or(0),
                        });
                    }
                }
            }
        }

        Ok(TimelineData {
            secid: secid.to_string(),
            name,
            pre_close,
            points,
        })
    }

    async fn search(&self, keyword: &str) -> Result<Vec<SearchResult>, String> {
        let url = format!(
            "{}/api/Info/Search?appid=elastic&token=&type=14&keyword={}",
            constants::EASTMONEY_SEARCH_URL,
            urlencoding::encode(keyword)
        );

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://www.eastmoney.com/")
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|e| format!("搜索请求失败: {}", e))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("搜索解析失败: {}", e))?;

        let mut results = Vec::new();
        if let Some(quotes) = body.get("QuotationCodeTable").and_then(|q| q.get("Data")).and_then(|d| d.as_array()) {
            for item in quotes {
                let code = item
                    .get("Code")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let market_id = item
                    .get("MktNum")
                    .and_then(|m| m.as_str())
                    .unwrap_or("0");
                results.push(SearchResult {
                    secid: format!("{}.{}", market_id, code),
                    code,
                    name: item
                        .get("Name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string(),
                    market: market_id.to_string(),
                });
            }
        }

        Ok(results)
    }

    async fn fetch_exrights(&self, secid: &str) -> Result<Vec<ExRightInfo>, String> {
        let (market, code) = secid
            .split_once('.')
            .unwrap_or(("0", ""));
        let url = format!(
            "{}/api/qt/stock/exRight/get?secid={}&fields1=f1,f2,f3,f4,f5,f6&fields2=f51,f52,f53,f54,f55,f56,f57",
            constants::EASTMONEY_HISTORY_URL,
            secid
        );

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://www.eastmoney.com/")
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|e| format!("除权除息请求失败: {}", e))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("除权除息解析失败: {}", e))?;

        let mut exrights = Vec::new();
        if let Some(data) = body.get("data").and_then(|d| d.get("exright")).and_then(|e| e.as_array()) {
            for item in data {
                if let Some(s) = item.as_str() {
                    let parts: Vec<&str> = s.split(',').collect();
                    if parts.len() >= 6 {
                        exrights.push(ExRightInfo {
                            secid: secid.to_string(),
                            ex_date: parts[0].to_string(),
                            bonus_share: parts[1].parse().unwrap_or(0.0),
                            allot_share: parts[2].parse().unwrap_or(0.0),
                            allot_price: parts[3].parse().unwrap_or(0.0),
                            dividend: parts[4].parse().unwrap_or(0.0),
                        });
                    }
                }
            }
        }

        Ok(exrights)
    }

    fn name(&self) -> &'static str {
        "东方财富"
    }

    fn priority(&self) -> u8 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_quote_url() {
        let secids = vec!["1.600000".to_string(), "0.000001".to_string()];
        let url = EastMoneySource::build_quote_url(&secids);
        assert!(url.contains("1.600000,0.000001"));
        assert!(url.contains("f2,f3,f4"));
    }

    #[test]
    fn test_parse_empty_quotes() {
        let data = serde_json::json!({});
        let quotes = EastMoneySource::parse_quotes(&data);
        assert!(quotes.is_empty());
    }

    #[test]
    fn test_parse_single_quote() {
        let data = serde_json::json!({
            "data": {
                "diff": [{
                    "f2": 10.5,
                    "f3": 1.5,
                    "f4": 0.15,
                    "f5": 10000,
                    "f6": 1050000.0,
                    "f8": 2.5,
                    "f9": 15.3,
                    "f10": 1.2,
                    "f12": "000001",
                    "f13": 0,
                    "f14": "平安银行",
                    "f15": 10.8,
                    "f16": 10.3,
                    "f17": 10.35,
                    "f18": 10.35,
                    "f20": 2000000000.0,
                    "f22": 0.1,
                    "f23": 16.0,
                    "f25": 5.0,
                    "f62": 5000000.0,
                    "f136": 1.2
                }]
            }
        });
        let quotes = EastMoneySource::parse_quotes(&data);
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].secid, "0.000001");
        assert_eq!(quotes[0].name, "平安银行");
        assert!((quotes[0].price - 10.5).abs() < 0.01);
    }
}
