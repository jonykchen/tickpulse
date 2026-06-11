//! 天天基金数据源
//! 基金实时估值（JSONP）/ 基金数据

use crate::config::constants;
use crate::market::{
    AdjustType, ExRightInfo, KlineBar, KlinePeriod, MarketDataSource, SearchResult, StockQuote,
    TimelineData,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// ==================== 基金数据结构 ====================

/// 基金实时估值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundEstimate {
    /// 基金代码
    pub fundcode: String,
    /// 基金名称
    pub name: String,
    /// 净值日期（YYYY-MM-DD）
    pub jzrq: String,
    /// 单位净值
    pub dwjz: f64,
    /// 估值净值
    pub gsz: f64,
    /// 估值涨幅百分比
    pub gszzl: f64,
    /// 估值时间（YYYY-MM-DD HH:MM）
    pub gztime: String,
}

/// 基金列表项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundListItem {
    /// 基金代码
    #[serde(rename = "FCODE")]
    pub fcode: String,
    /// 基金名称
    #[serde(rename = "SHORTNAME")]
    pub shortname: String,
    /// 基金类型
    #[serde(rename = "FTYPE")]
    pub ftype: Option<String>,
    /// 单位净值
    #[serde(rename = "NAV")]
    pub nav: Option<f64>,
    /// 累计净值
    #[serde(rename = "ACCNAV")]
    pub accnav: Option<f64>,
    /// 净值日期
    #[serde(rename = "PDATE")]
    pub pdate: Option<String>,
    /// 日涨跌幅
    #[serde(rename = "CHANGEPCT")]
    pub changepct: Option<f64>,
    /// 近一周涨幅
    #[serde(rename = "W1")]
    pub w1: Option<f64>,
    /// 近一月涨幅
    #[serde(rename = "M1")]
    pub m1: Option<f64>,
    /// 近三月涨幅
    #[serde(rename = "M3")]
    pub m3: Option<f64>,
    /// 近六月涨幅
    #[serde(rename = "M6")]
    pub m6: Option<f64>,
    /// 近一年涨幅
    #[serde(rename = "Y1")]
    pub y1: Option<f64>,
    /// 今年以来涨幅
    #[serde(rename = "TOYEAR")]
    pub toyear: Option<f64>,
    /// 成立以来涨幅
    #[serde(rename = "INCEXPFROM")]
    pub incexpfrom: Option<f64>,
}

/// 基金详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundDetail {
    /// 基金代码
    #[serde(rename = "FCODE")]
    pub fcode: String,
    /// 基金全称
    #[serde(rename = "FULLNAME")]
    pub fullname: Option<String>,
    /// 基金简称
    #[serde(rename = "SHORTNAME")]
    pub shortname: Option<String>,
    /// 基金类型
    #[serde(rename = "FTYPE")]
    pub ftype: Option<String>,
    /// 基金管理人
    #[serde(rename = "MNGCOMNAME")]
    pub mngcomname: Option<String>,
    /// 基金经理
    #[serde(rename = "MANAGERS")]
    pub managers: Option<String>,
    /// 成立日期
    #[serde(rename = "FOUNDDATE")]
    pub founddate: Option<String>,
    /// 基金规模（亿元）
    #[serde(rename = "FUNDSCALE")]
    pub fundscale: Option<f64>,
    /// 单位净值
    #[serde(rename = "NAV")]
    pub nav: Option<f64>,
    /// 累计净值
    #[serde(rename = "ACCNAV")]
    pub accnav: Option<f64>,
    /// 净值日期
    #[serde(rename = "PDATE")]
    pub pdate: Option<String>,
    /// 日涨跌幅
    #[serde(rename = "CHANGEPCT")]
    pub changepct: Option<f64>,
}

// ==================== API 响应结构 ====================

/// 基金列表 API 响应
#[derive(Debug, Deserialize)]
struct FundListResponse {
    #[serde(rename = "Datas")]
    datas: Vec<FundListItem>,
    #[serde(rename = "AllRecords")]
    all_records: Option<i32>,
    #[serde(rename = "PageIndex")]
    page_index: Option<i32>,
    #[serde(rename = "PageNum")]
    page_num: Option<i32>,
}

/// 基金详情 API 响应
#[derive(Debug, Deserialize)]
struct FundDetailResponse {
    #[serde(rename = "Datas")]
    datas: Vec<FundDetail>,
}

// ==================== 天天基金数据源 ====================

/// 天天基金数据源
pub struct FundSource {
    client: reqwest::Client,
}

impl FundSource {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(constants::HTTP_TIMEOUT_SECS))
            .gzip(true)
            .build()
            .expect("Failed to build Fund HTTP client");
        Self { client }
    }

    /// 解析 JSONP 响应为 JSON 字符串
    ///
    /// 天天基金返回格式示例：
    /// - `jsonpgz({"fundcode":"000001",...});`
    /// - `jQuery183({...})`
    ///
    /// 提取括号内的 JSON 部分
    pub fn jsonp_to_json(jsonp: &str) -> Option<String> {
        let trimmed = jsonp.trim();

        // 查找第一个左括号
        let start = trimmed.find('(')?;
        // 查找最后一个右括号（分号之前）
        let end = trimmed.rfind(')')?;

        if start >= end {
            return None;
        }

        Some(trimmed[start + 1..end].to_string())
    }

    /// 获取基金实时估值
    ///
    /// 端点：https://fundgz.1234567.com.cn/js/{fund_code}.js
    /// 返回 JSONP 格式数据
    pub async fn fetch_fund_estimate(&self, fund_code: &str) -> Result<FundEstimate, String> {
        let url = format!("https://fundgz.1234567.com.cn/js/{}.js", fund_code);

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://fund.eastmoney.com/")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await
            .map_err(|e| format!("基金估值请求失败: {}", e))?;

        let text = resp
            .text()
            .await
            .map_err(|e| format!("基金估值响应读取失败: {}", e))?;

        // 解析 JSONP
        let json_str = Self::jsonp_to_json(&text)
            .ok_or_else(|| format!("基金估值JSONP解析失败: {}", text))?;

        // 解析 JSON
        let estimate: FundEstimate = serde_json::from_str(&json_str)
            .map_err(|e| format!("基金估值JSON解析失败: {}", e))?;

        Ok(estimate)
    }

    /// 查询基金列表
    ///
    /// 端点：https://fundmobapi.eastmoney.com/FundMApi/FundNetList.ashx
    /// 参数：fundType=all, SortColumn=yz, Sort=desc, page, pagesize
    pub async fn fetch_fund_list(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<FundListItem>, i32), String> {
        let url = format!(
            "https://fundmobapi.eastmoney.com/FundMApi/FundNetList.ashx?\
             fundType=all&SortColumn=yz&Sort=desc&page={}&pagesize={}&version=WAP",
            page, page_size
        );

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://fund.eastmoney.com/")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await
            .map_err(|e| format!("基金列表请求失败: {}", e))?;

        let response: FundListResponse = resp
            .json()
            .await
            .map_err(|e| format!("基金列表解析失败: {}", e))?;

        let total = response.all_records.unwrap_or(0);
        Ok((response.datas, total))
    }

    /// 获取基金详情
    ///
    /// 端点：https://fundmobapi.eastmoney.com/FundMApi/FundDetail.ashx
    /// 参数：FCODE={fund_code}
    pub async fn fetch_fund_detail(&self, fund_code: &str) -> Result<Option<FundDetail>, String> {
        let url = format!(
            "https://fundmobapi.eastmoney.com/FundMApi/FundDetail.ashx?FCODE={}&version=WAP",
            fund_code
        );

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://fund.eastmoney.com/")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await
            .map_err(|e| format!("基金详情请求失败: {}", e))?;

        let response: FundDetailResponse = resp
            .json()
            .await
            .map_err(|e| format!("基金详情解析失败: {}", e))?;

        Ok(response.datas.into_iter().next())
    }
}

impl Default for FundSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MarketDataSource for FundSource {
    async fn fetch_quotes(&self, _secids: &[String]) -> Result<Vec<StockQuote>, String> {
        // 天天基金仅提供基金数据，不提供股票行情
        Err("天天基金不提供股票行情API".to_string())
    }

    async fn fetch_kline(
        &self,
        _secid: &str,
        _period: KlinePeriod,
        _limit: u32,
        _adjust: AdjustType,
    ) -> Result<Vec<KlineBar>, String> {
        Err("天天基金K线暂未实现".to_string())
    }

    async fn fetch_timeline(&self, _secid: &str) -> Result<TimelineData, String> {
        Err("天天基金分时暂未实现".to_string())
    }

    async fn search(&self, _keyword: &str) -> Result<Vec<SearchResult>, String> {
        Err("天天基金搜索暂未实现".to_string())
    }

    async fn fetch_exrights(&self, _secid: &str) -> Result<Vec<ExRightInfo>, String> {
        Err("天天基金除权暂未实现".to_string())
    }

    fn name(&self) -> &'static str {
        "天天基金"
    }

    fn priority(&self) -> u8 {
        5 // 新浪(4)之后
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonp_to_json() {
        // 标准格式
        let jsonp = r#"jsonpgz({"fundcode":"000001","name":"华夏成长","jzrq":"2024-01-15","dwjz":"1.234","gsz":"1.250","gszzl":"1.30","gztime":"2024-01-16 15:00"});"#;
        let json = FundSource::jsonp_to_json(jsonp);
        assert!(json.is_some());
        let json = json.unwrap();
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));

        // 解析验证
        let estimate: FundEstimate = serde_json::from_str(&json).unwrap();
        assert_eq!(estimate.fundcode, "000001");
        assert_eq!(estimate.name, "华夏成长");
        assert!((estimate.dwjz - 1.234).abs() < 0.001);
    }

    #[test]
    fn test_jsonp_to_json_with_jquery() {
        // jQuery 格式
        let jsonp = r#"jQuery183({"fundcode":"000001","name":"测试基金"});"#;
        let json = FundSource::jsonp_to_json(jsonp);
        assert!(json.is_some());
    }

    #[test]
    fn test_jsonp_to_json_invalid() {
        // 无效格式
        assert!(FundSource::jsonp_to_json("").is_none());
        assert!(FundSource::jsonp_to_json("no brackets").is_none());
        assert!(FundSource::jsonp_to_json("()").is_none());
    }

    #[test]
    fn test_fund_estimate_deserialize() {
        let json = r#"{"fundcode":"000001","name":"华夏成长","jzrq":"2024-01-15","dwjz":"1.234","gsz":"1.250","gszzl":"1.30","gztime":"2024-01-16 15:00"}"#;
        let estimate: FundEstimate = serde_json::from_str(json).unwrap();
        assert_eq!(estimate.fundcode, "000001");
        assert_eq!(estimate.name, "华夏成长");
        assert_eq!(estimate.jzrq, "2024-01-15");
        assert!((estimate.dwjz - 1.234).abs() < 0.001);
        assert!((estimate.gsz - 1.250).abs() < 0.001);
        assert!((estimate.gszzl - 1.30).abs() < 0.001);
    }

    #[test]
    fn test_fund_list_item_deserialize() {
        let json = r#"{"FCODE":"000001","SHORTNAME":"华夏成长","FTYPE":"混合型","NAV":"1.234","ACCNAV":"2.345","PDATE":"2024-01-15","CHANGEPCT":"1.50"}"#;
        let item: FundListItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.fcode, "000001");
        assert_eq!(item.shortname, "华夏成长");
        assert!(item.nav.is_some());
        assert!((item.nav.unwrap() - 1.234).abs() < 0.001);
    }
}
