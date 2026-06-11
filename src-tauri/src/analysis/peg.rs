//! PEG 估值引擎
//! rate/pe_digestion_years/calc_cagr + 五级评级

use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// PEG 评级
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PegRating {
    /// PEG < 0.5 — 严重低估
    ExtremelyUndervalued,
    /// PEG 0.5-0.8 — 低估
    Undervalued,
    /// PEG 0.8-1.2 — 合理
    Fair,
    /// PEG 1.2-1.5 — 高估
    Overvalued,
    /// PEG > 1.5 — 严重高估
    ExtremelyOvervalued,
}

impl PegRating {
    pub fn from_peg(peg: Decimal) -> Self {
        if peg < Decimal::from_str("0.5").unwrap() { Self::ExtremelyUndervalued }
        else if peg < Decimal::from_str("0.8").unwrap() { Self::Undervalued }
        else if peg <= Decimal::from_str("1.2").unwrap() { Self::Fair }
        else if peg <= Decimal::from_str("1.5").unwrap() { Self::Overvalued }
        else { Self::ExtremelyOvervalued }
    }

    pub fn display(&self) -> &'static str {
        match self {
            Self::ExtremelyUndervalued => "严重低估",
            Self::Undervalued => "低估",
            Self::Fair => "合理",
            Self::Overvalued => "高估",
            Self::ExtremelyOvervalued => "严重高估",
        }
    }

    pub fn score(&self) -> f64 {
        match self {
            Self::ExtremelyUndervalued => 95.0,
            Self::Undervalued => 75.0,
            Self::Fair => 50.0,
            Self::Overvalued => 30.0,
            Self::ExtremelyOvervalued => 10.0,
        }
    }
}

/// PEG 计算器
pub struct PegCalculator;

impl PegCalculator {
    /// 计算 PEG = PE / CAGR
    /// pe: 市盈率(TTM)
    /// cagr: 复合年增长率(%)
    /// 返回 None 当 cagr <= 0 时（PEG 无意义）
    pub fn calc_peg(pe: Decimal, cagr: Decimal) -> Option<Decimal> {
        if cagr <= Decimal::ZERO {
            return None;
        }
        Some(pe / cagr)
    }

    /// 计算复合年增长率 CAGR
    /// begin_value: 起始值
    /// end_value: 终止值
    /// years: 年数
    pub fn calc_cagr(begin_value: Decimal, end_value: Decimal, years: Decimal) -> Option<Decimal> {
        if begin_value <= Decimal::ZERO || end_value <= Decimal::ZERO || years <= Decimal::ZERO {
            return None;
        }
        // CAGR = (end/begin)^(1/years) - 1
        let ratio = end_value / begin_value;
        // 使用 f64 近似计算（Decimal 不支持幂运算）
        let ratio_f64 = ratio.to_f64()?;
        let years_f64 = years.to_f64()?;
        let cagr_f64 = ratio_f64.powf(1.0 / years_f64) - 1.0;
        Decimal::from_f64(cagr_f64 * 100.0) // 转为百分比
    }

    /// PE 消化年数 = PE × (1 - 留存比率) / 增长率
    /// pe: 市盈率
    /// retention_ratio: 留存比率 (0-1)
    /// growth_rate: 增长率(%)
    pub fn pe_digestion_years(pe: Decimal, retention_ratio: Decimal, growth_rate: Decimal) -> Option<Decimal> {
        if growth_rate <= Decimal::ZERO {
            return None;
        }
        Some(pe * (Decimal::ONE - retention_ratio) / (growth_rate / Decimal::from(100)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_peg() {
        let pe = Decimal::from(20);
        let cagr = Decimal::from_str("15.0").unwrap(); // 15%
        let peg = PegCalculator::calc_peg(pe, cagr).unwrap();
        assert_eq!(peg, Decimal::from_str("1.3333").unwrap().round_dp(4));
    }

    #[test]
    fn test_calc_peg_zero_growth() {
        let pe = Decimal::from(20);
        let cagr = Decimal::ZERO;
        assert!(PegCalculator::calc_peg(pe, cagr).is_none());
    }

    #[test]
    fn test_peg_rating_boundaries() {
        assert_eq!(PegRating::from_peg(Decimal::from_str("0.3").unwrap()), PegRating::ExtremelyUndervalued);
        assert_eq!(PegRating::from_peg(Decimal::from_str("0.7").unwrap()), PegRating::Undervalued);
        assert_eq!(PegRating::from_peg(Decimal::from_str("1.0").unwrap()), PegRating::Fair);
        assert_eq!(PegRating::from_peg(Decimal::from_str("1.3").unwrap()), PegRating::Overvalued);
        assert_eq!(PegRating::from_peg(Decimal::from_str("2.0").unwrap()), PegRating::ExtremelyOvervalued);
    }

    #[test]
    fn test_calc_cagr() {
        let begin = Decimal::from(100);
        let end = Decimal::from(200);
        let years = Decimal::from(5);
        let cagr = PegCalculator::calc_cagr(begin, end, years).unwrap();
        // CAGR = (200/100)^(1/5) - 1 ≈ 14.87%
        assert!(cagr > Decimal::from_str("14.0").unwrap());
        assert!(cagr < Decimal::from_str("15.0").unwrap());
    }
}
