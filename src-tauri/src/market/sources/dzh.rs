//! 大智慧数据源
//! 涨停天梯

use crate::config::constants;
use crate::market::{
    AdjustType, ExRightInfo, KlineBar, KlinePeriod, MarketDataSource, SearchResult, StockQuote,
    TimelineData,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 涨停天梯记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZtttRecord {
    /// 股票代码
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 连板天数
    pub limit_days: u32,
    /// 涨跌幅
    pub change_percent: f64,
    /// 涨停时间
    pub limit_time: String,
    /// 所属行业
    pub industry: String,
    /// 涨停原因
    pub reason: String,
}

/// 大智慧数据源
pub struct DzhSource {
    client: reqwest::Client,
}

impl DzhSource {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(constants::HTTP_TIMEOUT_SECS))
            .gzip(true)
            .build()
            .expect("Failed to build DZH HTTP client");
        Self { client }
    }

    /// 获取涨停天梯数据
    ///
    /// 返回所有涨停股票的连板数据，按连板天数降序排列
    pub async fn fetch_zttt(&self) -> Result<Vec<ZtttRecord>, String> {
        let resp = self
            .client
            .get(constants::DZH_ZTTT_URL)
            .header("Referer", "https://www.dzh.com.cn/")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await
            .map_err(|e| format!("大智慧涨停天梯请求失败: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!(
                "大智慧涨停天梯请求失败，状态码: {}",
                resp.status()
            ));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("大智慧涨停天梯解析失败: {}", e))?;

        Self::parse_zttt_response(&body)
    }

    /// 解析涨停天梯响应
    fn parse_zttt_response(data: &serde_json::Value) -> Result<Vec<ZtttRecord>, String> {
        // 大智慧API响应结构:
        // {
        //   "code": 0,
        //   "message": "success",
        //   "data": {
        //     "list": [
        //       {
        //         "code": "000001",
        //         "name": "平安银行",
        //         "lbDays": 3,
        //         "changePercent": 10.01,
        //         "limitTime": "09:30:00",
        //         "industry": "银行",
        //         "reason": "金融改革"
        //       }
        //     ]
        //   }
        // }
        let list = data
            .get("data")
            .and_then(|d| d.get("list"))
            .and_then(|l| l.as_array())
            .ok_or_else(|| "大智慧涨停天梯响应格式错误: 缺少 data.list".to_string())?;

        let mut records = Vec::new();
        for item in list {
            let record = ZtttRecord {
                code: item
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                name: item
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                limit_days: item
                    .get("lbDays")
                    .or_else(|| item.get("limit_days"))
                    .or_else(|| item.get("lb_days"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u32,
                change_percent: item
                    .get("changePercent")
                    .or_else(|| item.get("change_percent"))
                    .or_else(|| item.get("chgPct"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                limit_time: item
                    .get("limitTime")
                    .or_else(|| item.get("limit_time"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                industry: item
                    .get("industry")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                reason: item
                    .get("reason")
                    .or_else(|| item.get("limitReason"))
                    .or_else(|| item.get("limit_reason"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            };
            records.push(record);
        }

        // 按连板天数降序排序
        records.sort_by(|a, b| b.limit_days.cmp(&a.limit_days));

        Ok(records)
    }
}

impl Default for DzhSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MarketDataSource for DzhSource {
    async fn fetch_quotes(&self, _secids: &[String]) -> Result<Vec<StockQuote>, String> {
        // 大智慧不直接提供批量行情接口
        Err("大智慧不提供批量行情API".to_string())
    }

    async fn fetch_kline(
        &self,
        _secid: &str,
        _period: KlinePeriod,
        _limit: u32,
        _adjust: AdjustType,
    ) -> Result<Vec<KlineBar>, String> {
        Err("大智慧K线暂未实现".to_string())
    }

    async fn fetch_timeline(&self, _secid: &str) -> Result<TimelineData, String> {
        Err("大智慧分时暂未实现".to_string())
    }

    async fn search(&self, _keyword: &str) -> Result<Vec<SearchResult>, String> {
        Err("大智慧搜索暂未实现".to_string())
    }

    async fn fetch_exrights(&self, _secid: &str) -> Result<Vec<ExRightInfo>, String> {
        Err("大智慧除权暂未实现".to_string())
    }

    fn name(&self) -> &'static str {
        "大智慧"
    }

    fn priority(&self) -> u8 {
        6 // 天天基金(5)之后
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_zttt_response_empty() {
        let data = serde_json::json!({});
        let result = DzhSource::parse_zttt_response(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("缺少 data.list"));
    }

    #[test]
    fn test_parse_zttt_response_valid() {
        let data = serde_json::json!({
            "code": 0,
            "message": "success",
            "data": {
                "list": [
                    {
                        "code": "000001",
                        "name": "平安银行",
                        "lbDays": 3,
                        "changePercent": 10.01,
                        "limitTime": "09:30:00",
                        "industry": "银行",
                        "reason": "金融改革"
                    },
                    {
                        "code": "600519",
                        "name": "贵州茅台",
                        "lbDays": 5,
                        "changePercent": 10.00,
                        "limitTime": "09:35:00",
                        "industry": "白酒",
                        "reason": "消费复苏"
                    }
                ]
            }
        });

        let records = DzhSource::parse_zttt_response(&data).unwrap();
        assert_eq!(records.len(), 2);

        // 验证排序（连板天数降序）
        assert_eq!(records[0].code, "600519");
        assert_eq!(records[0].limit_days, 5);
        assert_eq!(records[1].code, "000001");
        assert_eq!(records[1].limit_days, 3);
    }

    #[test]
    fn test_parse_zttt_response_fallback_fields() {
        // 测试不同字段名的兼容性
        let data = serde_json::json!({
            "data": {
                "list": [
                    {
                        "code": "000001",
                        "name": "测试股票",
                        "limit_days": 2,
                        "change_percent": 9.98,
                        "limit_time": "10:00:00",
                        "industry": "科技",
                        "limitReason": "概念炒作"
                    }
                ]
            }
        });

        let records = DzhSource::parse_zttt_response(&data).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].limit_days, 2);
        assert!((records[0].change_percent - 9.98).abs() < 0.01);
        assert_eq!(records[0].reason, "概念炒作");
    }

    #[test]
    fn test_parse_zttt_response_defaults() {
        // 测试缺少字段时的默认值
        let data = serde_json::json!({
            "data": {
                "list": [
                    {}
                ]
            }
        });

        let records = DzhSource::parse_zttt_response(&data).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].code, "");
        assert_eq!(records[0].name, "");
        assert_eq!(records[0].limit_days, 1); // 默认1板
        assert_eq!(records[0].change_percent, 0.0);
    }
}
