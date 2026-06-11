//! 估值分析维度（含 PEG）
//! 基于PE/PB/PEG/市值评估估值水平

use crate::analysis::engine::{AnalysisDimension, DimensionRating, DimensionReport};
use crate::analysis::peg::PegCalculator;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

/// 估值分析
pub fn analyze_valuation(
    stock_name: &str,
    data: &str,
) -> DimensionReport {
    let mut score = 50.0_f64;
    let mut key_points = Vec::new();
    let mut risks = Vec::new();
    let mut opportunities = Vec::new();
    let mut confidence = 0.3_f64;

    if let Some(metrics) = parse_metrics(data) {
        confidence = 0.7; // 估值维度数据充分

        // 1. PE(TTM) 估值区间判断
        if metrics.pe_ttm > 0.0 && metrics.pe_ttm < 10.0 {
            score += 15.0;
            key_points.push(format!("PE(TTM) {:.1}，深度低估值", metrics.pe_ttm));
            opportunities.push("极低PE，存在估值修复空间，安全边际高".to_string());
        } else if metrics.pe_ttm >= 10.0 && metrics.pe_ttm < 20.0 {
            score += 10.0;
            key_points.push(format!("PE(TTM) {:.1}，估值偏低", metrics.pe_ttm));
            opportunities.push("低估值区域，性价比好".to_string());
        } else if metrics.pe_ttm >= 20.0 && metrics.pe_ttm < 35.0 {
            score += 3.0;
            key_points.push(format!("PE(TTM) {:.1}，估值合理", metrics.pe_ttm));
        } else if metrics.pe_ttm >= 35.0 && metrics.pe_ttm < 60.0 {
            score -= 5.0;
            key_points.push(format!("PE(TTM) {:.1}，估值偏高", metrics.pe_ttm));
            risks.push("PE偏高，需业绩高增长支撑".to_string());
        } else if metrics.pe_ttm >= 60.0 {
            score -= 12.0;
            risks.push(format!("PE(TTM) {:.1} 极高，估值泡沫风险", metrics.pe_ttm));
        } else {
            score -= 10.0;
            key_points.push("PE(TTM)为负，公司亏损，无法用PE估值".to_string());
            risks.push("亏损公司，估值分析需使用PB/PS替代".to_string());
        }

        // 2. PB 评估
        if metrics.pb > 0.0 && metrics.pb < 0.8 {
            score += 8.0;
            key_points.push(format!("PB {:.2}，深度破净", metrics.pb));
            opportunities.push("深度破净，若基本面改善则有估值修复空间".to_string());
        } else if metrics.pb >= 0.8 && metrics.pb < 1.5 {
            score += 3.0;
            key_points.push(format!("PB {:.2}，估值合理偏低", metrics.pb));
        } else if metrics.pb >= 1.5 && metrics.pb < 5.0 {
            key_points.push(format!("PB {:.2}，正常估值水平", metrics.pb));
        } else if metrics.pb >= 5.0 && metrics.pb < 10.0 {
            key_points.push(format!("PB {:.2}，存在一定溢价", metrics.pb));
        } else if metrics.pb >= 10.0 {
            score -= 5.0;
            risks.push(format!("PB {:.2} 极高，资产溢价显著", metrics.pb));
        }

        // 3. PEG 计算（如果有隐含增长率）
        if metrics.pe_ttm > 0.0 {
            // 假设增长率从YTD涨幅推断（粗略）
            let estimated_growth = if metrics.change_percent.abs() > 0.0 {
                metrics.change_percent.abs() * 4.0 // 粗略年化
            } else {
                10.0 // 默认10%
            };
            if let Some(pe_d) = Decimal::from_f64(metrics.pe_ttm) {
                if let Some(growth_d) = Decimal::from_f64(estimated_growth) {
                    if let Some(peg) = PegCalculator::calc_peg(pe_d, growth_d) {
                        let peg_f64 = peg.to_f64().unwrap_or(0.0);
                        let rating = crate::analysis::peg::PegRating::from_peg(peg);
                        key_points.push(format!("估算PEG {:.2}（{}）", peg_f64, rating.display()));
                        if peg_f64 < 0.8 {
                            score += 8.0;
                            opportunities.push(format!("PEG {:.2} 偏低，增长未被充分定价", peg_f64));
                        } else if peg_f64 > 1.5 {
                            score -= 5.0;
                            risks.push(format!("PEG {:.2} 偏高，增长可能无法支撑估值", peg_f64));
                        }
                    }
                }
            }
        }

        // 4. 市值规模
        if metrics.market_cap_billion > 500.0 {
            key_points.push(format!("大市值 {:.0}亿，估值稳定性高", metrics.market_cap_billion));
        } else if metrics.market_cap_billion < 50.0 && metrics.market_cap_billion > 0.0 {
            risks.push("小市值，估值波动性大".to_string());
        }
    } else {
        key_points.push("估值分析数据不足".to_string());
        risks.push("缺乏PE/PB数据".to_string());
    }

    let rating = DimensionRating::from_score(score.clamp(0.0, 100.0));
    let summary = format!(
        "{} 估值评级 {}，综合评分 {:.0}/100",
        stock_name, rating.display(), score
    );

    DimensionReport {
        dimension: AnalysisDimension::Valuation,
        rating,
        summary,
        key_points,
        risks,
        opportunities,
        confidence: confidence.clamp(0.0_f64, 1.0_f64),
    }
}

struct ParsedMetrics {
    pe_ttm: f64,
    pb: f64,
    market_cap_billion: f64,
    change_percent: f64,
}

fn parse_metrics(data: &str) -> Option<ParsedMetrics> {
    let mut pe_ttm = f64::NAN;
    let mut pb = f64::NAN;
    let mut market_cap_billion = 0.0;
    let mut change_percent = 0.0;
    let mut found = false;

    for line in data.lines() {
        let line = line.trim();
        if let Some(val) = extract_float_after(line, "PE(TTM)") {
            pe_ttm = val;
            found = true;
        } else if let Some(val) = extract_float_after(line, "PE") {
            if pe_ttm.is_nan() { pe_ttm = val; found = true; }
        }
        if let Some(val) = extract_float_after(line, "PB") {
            pb = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "市值") {
            market_cap_billion = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "涨跌幅") {
            change_percent = val;
            found = true;
        }
    }

    if found {
        Some(ParsedMetrics {
            pe_ttm: if pe_ttm.is_nan() { 0.0 } else { pe_ttm },
            pb: if pb.is_nan() { 0.0 } else { pb },
            market_cap_billion,
            change_percent,
        })
    } else {
        None
    }
}

fn extract_float_after(line: &str, keyword: &str) -> Option<f64> {
    if let Some(pos) = line.find(keyword) {
        let rest = &line[pos + keyword.len()..];
        let start = rest.find(|c: char| c.is_ascii_digit() || c == '-')?;
        let num_str: String = rest[start..]
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect();
        num_str.parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valuation_analysis() {
        let data = "PE(TTM): 22.5 PB: 6.8\n市值: 2100亿\n涨跌幅: 1.50%";
        let report = analyze_valuation("贵州茅台", data);
        assert_eq!(report.dimension, AnalysisDimension::Valuation);
        assert!(report.confidence > 0.5);
    }

    #[test]
    fn test_valuation_low_pe() {
        let data = "PE(TTM): 8.5 PB: 0.9\n市值: 300亿";
        let report = analyze_valuation("低估值股", data);
        assert!(report.opportunities.len() > 0);
    }
}
