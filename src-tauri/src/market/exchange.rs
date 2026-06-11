use serde::{Deserialize, Serialize};

/// 交易所标识 — 影响收盘竞价规则
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Exchange {
    SSE,  // 上交所
    SZSE, // 深交所
    BSE,  // 北交所
    HK,   // 港股
    US,   // 美股
}

impl Exchange {
    /// 从 secid 解析交易所
    /// secid 格式: "市场.代码"，0=深 1=沪
    pub fn from_secid(secid: &str) -> Self {
        if let Some(market_str) = secid.split('.').next() {
            match market_str {
                "1" => Self::SSE,
                "0" => Self::SZSE,
                _ => Self::SZSE, // 默认深交所
            }
        } else {
            Self::SZSE
        }
    }

    /// 从代码前缀判断交易所和板块
    pub fn from_code(code: &str) -> Self {
        if code.starts_with('6') || code.starts_with("688") {
            Self::SSE
        } else if code.starts_with('8') || code.starts_with('4') {
            Self::BSE
        } else {
            Self::SZSE
        }
    }
}

/// 沪深收盘价规则差异
///
/// | 交易所 | 14:57-15:00 | 收盘价确定方式 |
/// |--------|-------------|---------------|
/// | 深交所 | 收盘集合竞价 | 收盘竞价成交价 |
/// | 沪市主板 | 连续竞价 | 最后一笔交易前1分钟加权平均价 |
/// | 科创板 | 收盘集合竞价 | 收盘竞价成交价 |
/// | 北交所 | 收盘集合竞价 | 收盘竞价成交价 |
pub fn has_closing_auction(exchange: Exchange, code: &str) -> bool {
    match exchange {
        Exchange::SSE => {
            // 科创板(688)有收盘竞价，沪市主板无
            code.starts_with("688")
        }
        Exchange::SZSE => true, // 深交所全部有收盘竞价
        Exchange::BSE => true,  // 北交所有收盘竞价
        _ => true,
    }
}

/// 判断是否为科创板股票
pub fn is_star_market(code: &str) -> bool {
    code.starts_with("688")
}

/// 判断是否为创业板股票
pub fn is_chinext(code: &str) -> bool {
    code.starts_with("300") || code.starts_with("301")
}

/// 判断是否为北交所股票
pub fn is_bse(code: &str) -> bool {
    code.starts_with('8') || code.starts_with('4')
}

/// 判断是否为ST股（含全角星号/退市前缀）
pub fn is_st_stock(name: &str) -> bool {
    let pattern = regex::Regex::new(r"[SＳ][TＴ]|[\*＊]ST|退[A-Z]").unwrap();
    pattern.is_match(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_from_secid() {
        assert_eq!(Exchange::from_secid("1.600000"), Exchange::SSE);
        assert_eq!(Exchange::from_secid("0.000001"), Exchange::SZSE);
    }

    #[test]
    fn test_has_closing_auction() {
        assert!(has_closing_auction(Exchange::SZSE, "000001"));
        assert!(!has_closing_auction(Exchange::SSE, "600000"));
        assert!(has_closing_auction(Exchange::SSE, "688000"));
        assert!(has_closing_auction(Exchange::BSE, "830000"));
    }

    #[test]
    fn test_is_star_market() {
        assert!(is_star_market("688001"));
        assert!(!is_star_market("600000"));
    }

    #[test]
    fn test_is_chinext() {
        assert!(is_chinext("300001"));
        assert!(is_chinext("301001"));
        assert!(!is_chinext("000001"));
    }

    #[test]
    fn test_is_st_stock() {
        assert!(is_st_stock("ST某某"));
        assert!(is_st_stock("*ST某某"));
        assert!(is_st_stock("ＳＴ某某"));
        assert!(is_st_stock("退市某某"));
        assert!(!is_st_stock("某某股份"));
    }
}
