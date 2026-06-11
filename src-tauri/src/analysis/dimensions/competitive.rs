//! 竞争格局分析维度
//! 基于市值/换手率/主力资金判断竞争地位

use crate::analysis::engine::{AnalysisDimension, DimensionRating, DimensionReport};

/// 竞争格局分析
pub fn analyze_competitive_position(
    stock_name: &str,
    data: &str,
) -> DimensionReport {
    let mut score = 50.0_f64;
    let mut key_points = Vec::new();
    let mut risks = Vec::new();
    let mut opportunities = Vec::new();
    let mut confidence = 0.3;

    if let Some(metrics) = parse_metrics(data) {
        confidence = 0.55;

        // 1. 市值规模判断行业地位
        if metrics.market_cap_billion > 2000.0 {
            score += 15.0;
            key_points.push(format!("总市值 {:.0}亿，行业龙头级别", metrics.market_cap_billion));
            opportunities.push("龙头地位稳固，抗风险能力强".to_string());
        } else if metrics.market_cap_billion > 500.0 {
            score += 8.0;
            key_points.push(format!("总市值 {:.0}亿，行业二线蓝筹", metrics.market_cap_billion));
        } else if metrics.market_cap_billion > 100.0 {
            score += 3.0;
            key_points.push(format!("总市值 {:.0}亿，中盘股", metrics.market_cap_billion));
        } else {
            key_points.push(format!("总市值 {:.0}亿，小盘股", metrics.market_cap_billion));
            risks.push("市值偏小，流动性风险和竞争压力较大".to_string());
            score -= 5.0;
        }

        // 2. 换手率判断市场认可度
        if metrics.turnover_rate > 0.0 && metrics.turnover_rate < 3.0 {
            score += 5.0;
            key_points.push("换手率适中，筹码结构稳定".to_string());
        } else if metrics.turnover_rate > 15.0 {
            score -= 5.0;
            risks.push(format!("换手率 {:.1}% 过高，筹码松动", metrics.turnover_rate));
        }

        // 3. 主力资金态度
        if metrics.main_net_inflow > 5e7 {
            score += 8.0;
            key_points.push("主力资金大幅净流入，机构增持".to_string());
            opportunities.push("机构资金持续流入，竞争壁垒加固".to_string());
        } else if metrics.main_net_inflow < -5e7 {
            score -= 8.0;
            risks.push("主力资金大幅净流出，机构减持".to_string());
        }

        // 4. PB判断估值溢价（高PB=市场认可护城河）
        if metrics.pb > 5.0 {
            key_points.push(format!("市净率 {:.1}，市场认可较高护城河", metrics.pb));
            score += 5.0;
        } else if metrics.pb < 1.0 && metrics.pb > 0.0 {
            risks.push(format!("市净率 {:.1} 破净，市场对资产质量存疑", metrics.pb));
            score -= 5.0;
        }

        // 5. 两融标的判断
        if metrics.is_margin_target {
            key_points.push("两融标的，机构参与度高".to_string());
            score += 3.0;
        }
    } else {
        key_points.push("数据不足，竞争格局分析受限".to_string());
    }

    let rating = DimensionRating::from_score(score.clamp(0.0, 100.0));
    let summary = format!(
        "{} 竞争格局评级 {}，综合评分 {:.0}/100",
        stock_name, rating.display(), score
    );

    DimensionReport {
        dimension: AnalysisDimension::CompetitivePosition,
        rating,
        summary,
        key_points,
        risks,
        opportunities,
        confidence: confidence.clamp(0.0, 1.0),
    }
}

struct ParsedMetrics {
    market_cap_billion: f64,
    turnover_rate: f64,
    main_net_inflow: f64,
    pb: f64,
    is_margin_target: bool,
}

fn parse_metrics(data: &str) -> Option<ParsedMetrics> {
    let mut market_cap_billion = 0.0;
    let mut turnover_rate = 0.0;
    let mut main_net_inflow = 0.0;
    let mut pb = 0.0;
    let mut found = false;

    for line in data.lines() {
        let line = line.trim();
        if let Some(val) = extract_float_after(line, "市值") {
            market_cap_billion = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "换手率") {
            turnover_rate = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "PB") {
            pb = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "主力净流入") {
            main_net_inflow = val;
            found = true;
        }
    }

    if found {
        Some(ParsedMetrics {
            market_cap_billion,
            turnover_rate,
            main_net_inflow,
            pb,
            is_margin_target: data.contains("两融"),
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
    fn test_competitive_analysis() {
        let data = "股票: 贵州茅台 (600519)\n市值: 23200亿\n换手率: 1.50%\nPB: 10.5";
        let report = analyze_competitive_position("贵州茅台", data);
        assert_eq!(report.dimension, AnalysisDimension::CompetitivePosition);
        assert!(report.confidence > 0.3);
    }
}
