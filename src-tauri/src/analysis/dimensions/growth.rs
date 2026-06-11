//! 成长性评估维度
//! 基于PE/涨幅/资金流向/量比评估增长潜力

use crate::analysis::engine::{AnalysisDimension, DimensionRating, DimensionReport};

/// 成长性评估
pub fn analyze_growth_potential(
    stock_name: &str,
    data: &str,
) -> DimensionReport {
    let mut score = 50.0_f64;
    let mut key_points = Vec::new();
    let mut risks = Vec::new();
    let mut opportunities = Vec::new();
    let mut confidence = 0.3;

    if let Some(metrics) = parse_metrics(data) {
        confidence = 0.6;

        // 1. YTD涨幅 — 年内成长性验证
        if metrics.ytd_change > 50.0 {
            score += 15.0;
            key_points.push(format!("年初至今涨幅 {:.1}%，成长性得到市场验证", metrics.ytd_change));
            opportunities.push("年内涨幅显著，高成长预期".to_string());
        } else if metrics.ytd_change > 20.0 {
            score += 8.0;
            key_points.push(format!("年初至今涨幅 {:.1}%，增长态势良好", metrics.ytd_change));
        } else if metrics.ytd_change > 0.0 {
            score += 3.0;
            key_points.push(format!("年初至今涨幅 {:.1}%，温和增长", metrics.ytd_change));
        } else if metrics.ytd_change > -20.0 {
            key_points.push(format!("年初至今跌幅 {:.1}%，增长放缓", metrics.ytd_change));
        } else {
            score -= 10.0;
            risks.push(format!("年初至今跌幅 {:.1}%，成长预期恶化", metrics.ytd_change));
        }

        // 2. PE与成长匹配度（高PE需高增长支撑）
        if metrics.pe_ttm > 0.0 && metrics.pe_ttm < 20.0 {
            if metrics.ytd_change > 10.0 {
                key_points.push("低PE+正增长，成长性价比较高".to_string());
                score += 8.0;
                opportunities.push("低估值+增长，戴维斯双击潜力".to_string());
            } else {
                key_points.push("低PE但增长乏力，可能属于价值陷阱".to_string());
            }
        } else if metrics.pe_ttm > 40.0 {
            if metrics.ytd_change > 20.0 {
                key_points.push("高PE+高增长，市场给予成长溢价".to_string());
                score += 5.0;
            } else {
                risks.push(format!("PE {:.1} 高但增长不足，估值泡沫风险", metrics.pe_ttm));
                score -= 8.0;
            }
        }

        // 3. 量比 — 增长是否有资金配合
        if metrics.volume_ratio > 2.0 {
            key_points.push(format!("量比 {:.1}，放量验证增长预期", metrics.volume_ratio));
            score += 5.0;
            opportunities.push("放量突破，资金推动增长预期".to_string());
        } else if metrics.volume_ratio < 0.5 && metrics.volume_ratio > 0.0 {
            risks.push(format!("量比 {:.1}，缩量暗示增长动力不足", metrics.volume_ratio));
            score -= 3.0;
        }

        // 4. 主力资金流向 — 机构对成长的判断
        if metrics.main_net_inflow > 1e8 {
            key_points.push("主力资金大幅流入，机构看好成长前景".to_string());
            score += 5.0;
        } else if metrics.main_net_inflow < -1e8 {
            risks.push("主力资金流出，机构看淡成长前景".to_string());
            score -= 5.0;
        }

        // 5. 换手率 — 活跃度
        if metrics.turnover_rate > 3.0 && metrics.turnover_rate < 10.0 {
            key_points.push("换手率适中偏活跃，市场关注度好".to_string());
        }
    } else {
        key_points.push("成长性评估数据不足".to_string());
        risks.push("缺乏YTD涨幅和PE数据，无法有效评估成长性".to_string());
    }

    let rating = DimensionRating::from_score(score.clamp(0.0, 100.0));
    let summary = format!(
        "{} 成长性评级 {}，综合评分 {:.0}/100",
        stock_name, rating.display(), score
    );

    DimensionReport {
        dimension: AnalysisDimension::GrowthPotential,
        rating,
        summary,
        key_points,
        risks,
        opportunities,
        confidence: confidence.clamp(0.0, 1.0),
    }
}

struct ParsedMetrics {
    pe_ttm: f64,
    ytd_change: f64,
    volume_ratio: f64,
    main_net_inflow: f64,
    turnover_rate: f64,
}

fn parse_metrics(data: &str) -> Option<ParsedMetrics> {
    let mut pe_ttm = f64::NAN;
    let mut ytd_change = 0.0;
    let mut volume_ratio = 0.0;
    let mut main_net_inflow = 0.0;
    let mut turnover_rate = 0.0;
    let mut found = false;

    for line in data.lines() {
        let line = line.trim();
        if let Some(val) = extract_float_after(line, "PE(TTM)") {
            pe_ttm = val;
            found = true;
        } else if let Some(val) = extract_float_after(line, "PE") {
            if pe_ttm.is_nan() { pe_ttm = val; found = true; }
        }
        if let Some(val) = extract_float_after(line, "量比") {
            volume_ratio = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "主力净流入") {
            main_net_inflow = val;
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
            ytd_change,
            volume_ratio,
            main_net_inflow,
            turnover_rate,
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
    fn test_growth_analysis() {
        let data = "PE(TTM): 25.0\n量比: 2.50\n换手率: 3.50%\n涨跌幅: 1.50%";
        let report = analyze_growth_potential("贵州茅台", data);
        assert_eq!(report.dimension, AnalysisDimension::GrowthPotential);
        assert!(report.confidence > 0.3);
    }
}
