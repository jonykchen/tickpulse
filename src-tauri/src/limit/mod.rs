/// 涨跌停判断引擎
/// 6板块差异化引擎 + 向上/向下取整 + 历史切换
pub mod board_type;

use crate::market::exchange::{is_bse, is_chinext, is_star_market, is_st_stock, Exchange};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

/// 涨停价计算（向上取整到0.01）
pub fn calc_limit_up_price(pre_close: f64, limit_rate: f64) -> f64 {
    let pre = Decimal::from_f64(pre_close).unwrap_or(Decimal::ZERO);
    let rate = Decimal::from_f64(limit_rate).unwrap_or(Decimal::ZERO);
    let limit = pre * (Decimal::ONE + rate / Decimal::from(100));
    // 向上取整到0.01
    let factor = Decimal::from_f64(0.01).unwrap_or(Decimal::from_str("0.01").unwrap());
    let rounded = (limit / factor).ceil() * factor;
    rounded.to_f64().unwrap_or(0.0)
}

/// 跌停价计算（向下取整到0.01）
pub fn calc_limit_down_price(pre_close: f64, limit_rate: f64) -> f64 {
    let pre = Decimal::from_f64(pre_close).unwrap_or(Decimal::ZERO);
    let rate = Decimal::from_f64(limit_rate).unwrap_or(Decimal::ZERO);
    let limit = pre * (Decimal::ONE - rate / Decimal::from(100));
    // 向下取整到0.01
    let factor = Decimal::from_f64(0.01).unwrap_or(Decimal::from_str("0.01").unwrap());
    let rounded = (limit / factor).floor() * factor;
    rounded.to_f64().unwrap_or(0.0)
}

/// 获取涨跌停幅度
/// 创业板2020.8.24前10%，之后20%
pub fn get_limit_rate(code: &str, name: &str, date: chrono::NaiveDate) -> f64 {
    // ST/退市股：5%
    if is_st_stock(name) {
        return 5.0;
    }

    // 北交所：30%
    if is_bse(code) {
        return 30.0;
    }

    // 科创板：20%
    if is_star_market(code) {
        return 20.0;
    }

    // 创业板：2020.8.24前10%，之后20%
    if is_chinext(code) {
        let switch_date = chrono::NaiveDate::from_ymd_opt(2020, 8, 24).unwrap();
        return if date < switch_date { 10.0 } else { 20.0 };
    }

    // 主板：10%
    10.0
}

/// 封板率计算（排除午休）
/// 有效交易时长 = 240分钟（4小时）
pub fn calc_seal_strength(
    first_seal_time: Option<chrono::NaiveTime>,
    current_time: chrono::NaiveTime,
    break_count: u32,
) -> f64 {
    if let Some(seal_time) = first_seal_time {
        let total_minutes = 240.0; // 有效交易时长
        let minutes_sealed = calc_effective_minutes(seal_time, current_time);
        let strength = minutes_sealed / total_minutes;
        // 炸板惩罚
        let penalty = break_count as f64 * 0.05;
        (strength - penalty).max(0.0).min(1.0)
    } else {
        0.0
    }
}

/// 计算有效交易分钟数（排除午休11:30-13:00）
fn calc_effective_minutes(start: chrono::NaiveTime, end: chrono::NaiveTime) -> f64 {
    let morning_end = chrono::NaiveTime::from_hms_opt(11, 30, 0).unwrap();
    let afternoon_start = chrono::NaiveTime::from_hms_opt(13, 0, 0).unwrap();

    let total_secs = (end - start).num_seconds() as f64;
    let lunch_secs = 90 * 60; // 90分钟午休

    // 如果跨越午休，减去午休时间
    if start < morning_end && end > afternoon_start {
        (total_secs - lunch_secs as f64) / 60.0
    } else if start >= afternoon_start && end >= afternoon_start {
        total_secs / 60.0
    } else if start < morning_end && end <= morning_end {
        total_secs / 60.0
    } else {
        total_secs / 60.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_up_price() {
        // 主板10%涨停
        let up = calc_limit_up_price(10.0, 10.0);
        assert!((up - 11.0).abs() < 0.01);

        // 创业板20%涨停
        let up = calc_limit_up_price(10.0, 20.0);
        assert!((up - 12.0).abs() < 0.01);
    }

    #[test]
    fn test_limit_down_price() {
        // 主板10%跌停
        let down = calc_limit_down_price(10.0, 10.0);
        assert!((down - 9.0).abs() < 0.01);

        // 创业板20%跌停
        let down = calc_limit_down_price(10.0, 20.0);
        assert!((down - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_limit_up_rounding() {
        // 向上取整测试：10.35 * 1.1 = 11.385 → 11.39
        let up = calc_limit_up_price(10.35, 10.0);
        assert!((up - 11.39).abs() < 0.01);
    }

    #[test]
    fn test_limit_down_rounding() {
        // 向下取整测试：10.35 * 0.9 = 9.315 → 9.31
        let down = calc_limit_down_price(10.35, 10.0);
        assert!((down - 9.31).abs() < 0.01);
    }

    #[test]
    fn test_get_limit_rate() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();

        // 主板
        assert_eq!(get_limit_rate("600000", "浦发银行", date), 10.0);
        // 创业板（2024年）
        assert_eq!(get_limit_rate("300001", "特锐德", date), 20.0);
        // 科创板
        assert_eq!(get_limit_rate("688001", "华兴源创", date), 20.0);
        // ST
        assert_eq!(get_limit_rate("600000", "ST某某", date), 5.0);
        // 北交所
        assert_eq!(get_limit_rate("830001", "某北交所", date), 30.0);

        // 创业板历史切换（2020.8.24前10%）
        let old_date = chrono::NaiveDate::from_ymd_opt(2020, 6, 1).unwrap();
        assert_eq!(get_limit_rate("300001", "特锐德", old_date), 10.0);
    }

    #[test]
    fn test_seal_strength() {
        let seal_time = chrono::NaiveTime::from_hms_opt(9, 30, 0).unwrap();
        let current = chrono::NaiveTime::from_hms_opt(10, 30, 0).unwrap();
        let strength = calc_seal_strength(Some(seal_time), current, 0);
        assert!(strength > 0.0 && strength <= 1.0);
    }
}
