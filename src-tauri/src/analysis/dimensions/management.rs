//! 管理层质量分析维度
//! 基于市值规模/融资融券/主力资金流向间接评估管理层

use crate::analysis::engine::{AnalysisDimension, DimensionRating, DimensionReport};

/// 管理层质量分析
pub fn analyze_management_quality(
    stock_name: &str,
    data: &str,
) -> DimensionReport {
    let mut score = 50.0_f64;
    let mut key_points = Vec::new();
    let mut risks = Vec::new();
    let mut opportunities = Vec::new();
    let mut confidence = 0.25; // 间接指标，置信度较低

    if let Some(metrics) = parse_metrics(data) {
        confidence = 0.45;

        // 1. 市值规模 — 大市值暗示管理层长期经营能力
        if metrics.market_cap_billion > 1000.0 {
            score += 10.0;
            key_points.push(format!("市值 {:.0}亿，大市值公司治理结构通常更完善", metrics.market_cap_billion));
        } else if metrics.market_cap_billion > 200.0 {
            score += 5.0;
            key_points.push(format!("市值 {:.0}亿，中等规模，治理水平参差", metrics.market_cap_billion));
        } else {
            key_points.push(format!("市值 {:.0}亿，小公司治理风险较高", metrics.market_cap_billion));
            risks.push("小市值公司治理结构可能不够完善".to_string());
        }

        // 2. 主力资金流向 — 机构用脚投票
        if metrics.main_net_inflow > 1e8 {
            score += 5.0;
            key_points.push("主力资金大幅流入，机构认可管理层战略方向".to_string());
        } else if metrics.main_net_inflow < -1e8 {
            score -= 5.0;
            risks.push("主力资金大幅流出，机构对管理层前景存疑".to_string());
        }

        // 3. 涨跌幅 — 市场对管理层经营的投票
        if metrics.change_percent > 5.0 {
            key_points.push("股价大幅上涨，市场对管理层表现肯定".to_string());
            score += 3.0;
        } else if metrics.change_percent < -5.0 {
            risks.push("股价大幅下跌，市场对管理层信心不足".to_string());
            score -= 5.0;
        }

        // 4. 两融标的 — 合规性和市场认可的代理指标
        if metrics.is_margin_target {
            key_points.push("两融标的，满足交易所合规要求".to_string());
            score += 3.0;
        }

        // 5. 换手率 — 稳定换手暗示长期股东信任
        if metrics.turnover_rate > 0.0 && metrics.turnover_rate < 2.0 {
            key_points.push("低换手率，股东长期持有，信任管理层".to_string());
            score += 3.0;
        } else if metrics.turnover_rate > 10.0 {
            risks.push("高换手率，股东短线交易为主，缺乏长期信任".to_string());
        }

        // 置信度说明
        key_points.push("注意：管理层质量主要依赖财报数据，当前为间接推断".to_string());
    } else {
        key_points.push("数据不足，管理层质量分析严重受限".to_string());
        risks.push("需要财报/公告/股权结构等数据支撑".to_string());
    }

    let rating = DimensionRating::from_score(score.clamp(0.0, 100.0));
    let summary = format!(
        "{} 管理层质量评级 {}，综合评分 {:.0}/100（间接推断）",
        stock_name, rating.display(), score
    );

    DimensionReport {
        dimension: AnalysisDimension::ManagementQuality,
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
    main_net_inflow: f64,
    change_percent: f64,
    turnover_rate: f64,
    is_margin_target: bool,
}

fn parse_metrics(data: &str) -> Option<ParsedMetrics> {
    let mut market_cap_billion = 0.0;
    let mut main_net_inflow = 0.0;
    let mut change_percent = 0.0;
    let mut turnover_rate = 0.0;
    let mut found = false;

    for line in data.lines() {
        let line = line.trim();
        if let Some(val) = extract_float_after(line, "市值") {
            market_cap_billion = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "主力净流入") {
            main_net_inflow = val;
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
            market_cap_billion,
            main_net_inflow,
            change_percent,
            turnover_rate,
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
    fn test_management_analysis() {
        let data = "市值: 1500亿\n主力净流入: 2.5亿\n换手率: 1.50%\n涨跌幅: 1.20%";
        let report = analyze_management_quality("贵州茅台", data);
        assert_eq!(report.dimension, AnalysisDimension::ManagementQuality);
        assert!(report.confidence > 0.2);
    }
}
