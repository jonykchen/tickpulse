//! Ticker 格式归一化
//! 5 种格式互转：东财 secid / 腾讯 / 新浪 / 证券代码 / 巨潮

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// 股票代码正则：6位数字
static CODE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d{6}$").unwrap());

/// Ticker 归一化结构
#[derive(Debug, Clone, PartialEq)]
pub struct UnifiedTicker {
    /// 市场编号 "0"=深 "1"=沪
    pub market: String,
    /// 6位股票代码
    pub code: String,
}

impl UnifiedTicker {
    /// 从任意格式解析
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();

        // 1. 东财 secid 格式: "1.600519" / "0.000001"
        if let Some((m, c)) = input.split_once('.') {
            if (m == "0" || m == "1" || m == "116" || m == "134") && CODE_PATTERN.is_match(c) {
                return Some(Self {
                    market: normalize_market(m),
                    code: c.to_string(),
                });
            }
        }

        // 2. 腾讯格式: "sh600519" / "sz000001"
        if input.starts_with("sh") || input.starts_with("SH") {
            let code = &input[2..];
            if CODE_PATTERN.is_match(code) {
                return Some(Self { market: "1".to_string(), code: code.to_string() });
            }
        }
        if input.starts_with("sz") || input.starts_with("SZ") {
            let code = &input[2..];
            if CODE_PATTERN.is_match(code) {
                return Some(Self { market: "0".to_string(), code: code.to_string() });
            }
        }

        // 3. 新浪格式同腾讯
        // 4. 纯6位代码：从代码推断市场
        if CODE_PATTERN.is_match(input) {
            let market = infer_market(input);
            return Some(Self { market, code: input.to_string() });
        }

        None
    }

    /// 转为东财 secid 格式
    pub fn to_eastmoney(&self) -> String {
        format!("{}.{}", self.market, self.code)
    }

    /// 转为腾讯格式
    pub fn to_tencent(&self) -> String {
        let prefix = if self.market == "1" { "sh" } else { "sz" };
        format!("{}{}", prefix, self.code)
    }

    /// 转为新浪格式（同腾讯）
    pub fn to_sina(&self) -> String {
        self.to_tencent()
    }

    /// 转为巨潮资讯格式（cninfo）
    pub fn to_cninfo(&self) -> String {
        // 巨潮用全小写市场前缀
        let prefix = if self.market == "1" { "sh" } else { "sz" };
        format!("{}{}", prefix, self.code)
    }
}

/// 市场编号归一化
fn normalize_market(m: &str) -> String {
    match m {
        "1" | "116" | "134" => "1".to_string(), // 沪市/港股市场编号统一
        "0" => "0".to_string(),
        _ => m.to_string(),
    }
}

/// 从6位代码推断市场
fn infer_market(code: &str) -> String {
    // 沪市：6开头 / 科创板688
    if code.starts_with('6') {
        return "1".to_string();
    }
    // 深市：0/3开头 / 创业板300
    if code.starts_with('0') || code.starts_with('3') {
        return "0".to_string();
    }
    // 北交所：8/4开头
    if code.starts_with('8') || code.starts_with('4') {
        return "0".to_string(); // 北交所在东财用0
    }
    "1".to_string() // 默认沪市
}

/// 数据供应商能力映射
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataCapability {
    Quotes,
    KlineDaily,
    KlineMinute,
    Timeline,
    Search,
    Exrights,
    BlockTrade,
    Northbound,
    Zttt,
}

/// 数据供应商
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataVendor {
    EastMoney,
    Tencent,
    Ths,
    Sina,
    Fund,
    Dzh,
}

impl DataVendor {
    pub fn name(&self) -> &'static str {
        match self {
            Self::EastMoney => "东方财富",
            Self::Tencent => "腾讯财经",
            Self::Ths => "同花顺",
            Self::Sina => "新浪财经",
            Self::Fund => "天天基金",
            Self::Dzh => "大智慧",
        }
    }
}

/// 供应商能力降级链
pub static VENDOR_FALLBACK_CHAIN: Lazy<HashMap<DataCapability, Vec<DataVendor>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(DataCapability::Quotes, vec![DataVendor::EastMoney, DataVendor::Tencent]);
    m.insert(DataCapability::KlineDaily, vec![DataVendor::EastMoney, DataVendor::Tencent, DataVendor::Sina]);
    m.insert(DataCapability::KlineMinute, vec![DataVendor::EastMoney, DataVendor::Tencent]);
    m.insert(DataCapability::Timeline, vec![DataVendor::EastMoney]);
    m.insert(DataCapability::Search, vec![DataVendor::EastMoney, DataVendor::Tencent]);
    m.insert(DataCapability::Exrights, vec![DataVendor::EastMoney]);
    m.insert(DataCapability::BlockTrade, vec![DataVendor::EastMoney]);
    m.insert(DataCapability::Northbound, vec![DataVendor::EastMoney, DataVendor::Ths]);
    m.insert(DataCapability::Zttt, vec![DataVendor::EastMoney, DataVendor::Dzh]);
    m
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_eastmoney_secid() {
        let t = UnifiedTicker::parse("1.600519").unwrap();
        assert_eq!(t.market, "1");
        assert_eq!(t.code, "600519");
    }

    #[test]
    fn test_parse_tencent_format() {
        let t = UnifiedTicker::parse("sh600519").unwrap();
        assert_eq!(t.market, "1");
        assert_eq!(t.code, "600519");

        let t2 = UnifiedTicker::parse("sz000001").unwrap();
        assert_eq!(t2.market, "0");
        assert_eq!(t2.code, "000001");
    }

    #[test]
    fn test_parse_pure_code() {
        let t = UnifiedTicker::parse("600519").unwrap();
        assert_eq!(t.market, "1");

        let t2 = UnifiedTicker::parse("000001").unwrap();
        assert_eq!(t2.market, "0");
    }

    #[test]
    fn test_roundtrip_conversions() {
        let t = UnifiedTicker::parse("1.600519").unwrap();
        assert_eq!(t.to_eastmoney(), "1.600519");
        assert_eq!(t.to_tencent(), "sh600519");
        assert_eq!(t.to_sina(), "sh600519");
        assert_eq!(t.to_cninfo(), "sh600519");
    }

    #[test]
    fn test_invalid_input() {
        assert!(UnifiedTicker::parse("").is_none());
        assert!(UnifiedTicker::parse("abc").is_none());
        assert!(UnifiedTicker::parse("1.<script>").is_none());
    }
}
