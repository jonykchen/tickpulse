//! 财务健康分析维度
//! 基于PE/PB/换手率/涨跌幅评估财务质量

use crate::analysis::engine::{AnalysisDimension, DimensionRating, DimensionReport};

/// 财务健康分析
pub fn analyze_financial_health(
    stock_name: &str,
    data: &str,
) -> DimensionReport {
    let mut score = 50.0_f64;
    let mut key_points = Vec::new();
    let mut risks = Vec::new();
    let mut opportunities = Vec::new();
    let mut confidence = 0.3_f64;

    if let Some(metrics) = parse_metrics(data) {
        confidence = 0.65;

        // 1. PE_TTM 估值评估
        if metrics.pe_ttm > 0.0 && metrics.pe_ttm < 15.0 {
            score += 10.0;
            key_points.push(format!("PE(TTM) {:.1}，估值偏低，安全边际充足", metrics.pe_ttm));
            opportunities.push("低PE可能存在价值重估机会".to_string());
        } else if metrics.pe_ttm >= 15.0 && metrics.pe_ttm < 30.0 {
            score += 5.0;
            key_points.push(format!("PE(TTM) {:.1}，估值合理", metrics.pe_ttm));
        } else if metrics.pe_ttm >= 30.0 && metrics.pe_ttm < 60.0 {
            key_points.push(format!("PE(TTM) {:.1}，估值偏高", metrics.pe_ttm));
            risks.push("PE偏高，需高增长支撑否则估值回归风险".to_string());
        } else if metrics.pe_ttm >= 60.0 {
            score -= 10.0;
            risks.push(format!("PE(TTM) {:.1} 极高，估值泡沫风险", metrics.pe_ttm));
        } else {
            // PE < 0 亏损
            score -= 15.0;
            key_points.push("PE(TTM)为负，公司亏损".to_string());
            risks.push("公司处于亏损状态，财务健康度堪忧".to_string());
        }

        // 2. PB 评估
        if metrics.pb > 0.0 && metrics.pb < 1.0 {
            score += 5.0;
            key_points.push(format!("PB {:.2} 破净，资产折价", metrics.pb));
            opportunities.push("破净状态下若盈利恢复则有估值修复空间".to_string());
        } else if metrics.pb >= 1.0 && metrics.pb < 3.0 {
            key_points.push(format!("PB {:.2}，正常水平", metrics.pb));
        } else if metrics.pb >= 3.0 && metrics.pb < 8.0 {
            key_points.push(format!("PB {:.2}，存在一定溢价", metrics.pb));
        } else if metrics.pb >= 8.0 {
            score -= 5.0;
            risks.push(format!("PB {:.2} 较高，资产溢价显著", metrics.pb));
        }

        // 3. 涨跌幅判断市场对财务的认可度
        if metrics.change_percent > 3.0 {
            key_points.push("当日上涨明显，市场对财务预期正面".to_string());
        } else if metrics.change_percent < -3.0 {
            risks.push("当日下跌明显，可能反映财务预期恶化".to_string());
        }

        // 4. 换手率+涨跌幅交叉验证
        if metrics.turnover_rate > 5.0 && metrics.change_percent < -2.0 {
            score -= 5.0;
            risks.push("高换手率+下跌，可能暗示机构出逃".to_string());
        } else if metrics.turnover_rate < 1.0 && metrics.change_percent > 2.0 {
            opportunities.push("低换手率+上涨，筹码锁定良好".to_string());
            score += 5.0;
        }

        // 5. 停牌检查
        if metrics.is_suspended {
            score -= 10.0;
            risks.push("股票停牌中，流动性受限".to_string());
            key_points.push("当前停牌状态".to_string());
        }
    } else {
        key_points.push("缺乏财务数据，分析置信度低".to_string());
        risks.push("PE/PB数据缺失，无法评估财务健康".to_string());
    }

    let rating = DimensionRating::from_score(score.clamp(0.0, 100.0));
    let summary = format!(
        "{} 财务健康评级 {}，综合评分 {:.0}/100",
        stock_name, rating.display(), score
    );

    DimensionReport {
        dimension: AnalysisDimension::FinancialHealth,
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
    change_percent: f64,
    turnover_rate: f64,
    is_suspended: bool,
}

fn parse_metrics(data: &str) -> Option<ParsedMetrics> {
    let mut pe_ttm = f64::NAN;
    let mut pb = f64::NAN;
    let mut change_percent = 0.0;
    let mut turnover_rate = 0.0;
    let mut found = false;

    for line in data.lines() {
        let line = line.trim();
        if let Some(val) = extract_float_after(line, "PE(TTM)") {
            pe_ttm = val;
            found = true;
        } else if let Some(val) = extract_float_after(line, "PE") {
            if pe_ttm.is_nan() {
                pe_ttm = val;
                found = true;
            }
        }
        if let Some(val) = extract_float_after(line, "PB") {
            pb = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "涨跌幅") {
            change_percent = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "换手率") {
            turnover_rate = val;
            found = true;
        }
    }

    if found {
        Some(ParsedMetrics {
            pe_ttm: if pe_ttm.is_nan() { 0.0 } else { pe_ttm },
            pb: if pb.is_nan() { 0.0 } else { pb },
            change_percent,
            turnover_rate,
            is_suspended: data.contains("停牌"),
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
    fn test_financial_health_analysis() {
        let data = "股票: 贵州茅台 (600519)\nPE(TTM): 28.5\nPB: 8.2\n换手率: 1.20%\n涨跌幅: 2.30%";
        let report = analyze_financial_health("贵州茅台", data);
        assert_eq!(report.dimension, AnalysisDimension::FinancialHealth);
        assert!(report.confidence > 0.3);
    }

    #[test]
    fn test_financial_loss() {
        let data = "PE(TTM): -5.3 PB: 1.2";
        let report = analyze_financial_health("亏损股", data);
        assert!(report.risks.iter().any(|r| r.contains("亏损")));
    }
}
