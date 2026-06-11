use serde::{Deserialize, Serialize};

/// 板块类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardType {
    /// 沪市主板
    MainBoardSH,
    /// 深市主板
    MainBoardSZ,
    /// 创业板
    ChiNext,
    /// 科创板
    StarMarket,
    /// 北交所
    BSE,
}

/// 股票状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StockStatus {
    /// 正常交易
    Normal,
    /// ST股
    ST,
    /// 退市整理
    Delisting,
    /// 新股（上市首日/次日起）
    NewStock,
    /// 停牌
    Suspended,
    /// 临时停牌
    TempSuspended,
}

impl BoardType {
    /// 从股票代码判断板块类型
    pub fn from_code(code: &str) -> Self {
        if code.starts_with("688") {
            Self::StarMarket
        } else if code.starts_with("300") || code.starts_with("301") {
            Self::ChiNext
        } else if code.starts_with('8') || code.starts_with('4') {
            Self::BSE
        } else if code.starts_with('6') {
            Self::MainBoardSH
        } else {
            Self::MainBoardSZ
        }
    }

    /// 获取涨跌停幅度
    pub fn limit_rate(&self, is_st: bool) -> f64 {
        if is_st {
            return 5.0;
        }
        match self {
            Self::MainBoardSH | Self::MainBoardSZ => 10.0,
            Self::ChiNext | Self::StarMarket => 20.0,
            Self::BSE => 30.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_type_from_code() {
        assert_eq!(BoardType::from_code("600000"), BoardType::MainBoardSH);
        assert_eq!(BoardType::from_code("000001"), BoardType::MainBoardSZ);
        assert_eq!(BoardType::from_code("300001"), BoardType::ChiNext);
        assert_eq!(BoardType::from_code("688001"), BoardType::StarMarket);
        assert_eq!(BoardType::from_code("830001"), BoardType::BSE);
    }

    #[test]
    fn test_limit_rate() {
        assert_eq!(BoardType::MainBoardSH.limit_rate(false), 10.0);
        assert_eq!(BoardType::ChiNext.limit_rate(false), 20.0);
        assert_eq!(BoardType::StarMarket.limit_rate(false), 20.0);
        assert_eq!(BoardType::BSE.limit_rate(false), 30.0);
        assert_eq!(BoardType::MainBoardSH.limit_rate(true), 5.0);
    }
}
