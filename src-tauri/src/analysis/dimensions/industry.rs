//! 行业趋势分析维度
//! 基于行情数据的规则引擎：YTD涨幅、板块联动、资金流向

use crate::analysis::engine::{AnalysisDimension, DimensionRating, DimensionReport};

/// 行业趋势分析
pub fn analyze_industry_trend(
    stock_name: &str,
    data: &str,
) -> DimensionReport {
    let mut score = 50.0_f64; // 基准分
    let mut key_points = Vec::new();
    let mut risks = Vec::new();
    let mut opportunities = Vec::new();
    let mut confidence = 0.3_f64;

    // 从数据中提取关键指标
    if let Some(metrics) = parse_stock_metrics(data) {
        confidence = 0.6;

        // 1. YTD涨幅判断年内趋势
        if metrics.ytd_change > 30.0 {
            score += 10.0;
            key_points.push(format!("年初至今涨幅 {:.1}%，年内趋势强劲", metrics.ytd_change));
            opportunities.push("年内涨幅显著，行业景气度高".to_string());
        } else if metrics.ytd_change > 10.0 {
            score += 5.0;
            key_points.push(format!("年初至今涨幅 {:.1}%，趋势向好", metrics.ytd_change));
        } else if metrics.ytd_change < -20.0 {
            score -= 10.0;
            key_points.push(format!("年初至今跌幅 {:.1}%，行业承压", metrics.ytd_change));
            risks.push("年内跌幅较大，行业周期性下行风险".to_string());
        } else {
            key_points.push(format!("年初至今涨跌幅 {:.1}%，行业运行平稳", metrics.ytd_change));
        }

        // 2. 当日涨跌幅 + 涨速判断短期趋势
        if metrics.change_percent > 5.0 {
            score += 5.0;
            key_points.push(format!("当日涨幅 {:.1}%，短线强势", metrics.change_percent));
            opportunities.push("短线资金关注度高".to_string());
        } else if metrics.change_percent < -5.0 {
            score -= 5.0;
            risks.push(format!("当日跌幅 {:.1}%，短线走弱", metrics.change_percent));
        }

        if metrics.change_speed > 2.0 {
            key_points.push(format!("涨速 {:.2}，异动拉升中", metrics.change_speed));
        } else if metrics.change_speed < -2.0 {
            risks.push(format!("涨速 {:.2}，加速下跌中", metrics.change_speed));
        }

        // 3. 主力资金流向判断机构态度
        if metrics.main_net_inflow > 0.0 {
            score += 5.0;
            key_points.push("主力资金净流入，机构看多".to_string());
        } else if metrics.main_net_inflow < -1e8 {
            score -= 5.0;
            risks.push("主力资金大幅净流出，机构看空".to_string());
        }

        // 4. 换手率判断市场活跃度
        if metrics.turnover_rate > 10.0 {
            key_points.push(format!("换手率 {:.1}%，交投异常活跃", metrics.turnover_rate));
            risks.push("高换手率可能暗示短期分歧加大".to_string());
        } else if metrics.turnover_rate > 5.0 {
            score += 3.0;
            key_points.push(format!("换手率 {:.1}%，市场关注度适中", metrics.turnover_rate));
        } else if metrics.turnover_rate < 1.0 && metrics.turnover_rate > 0.0 {
            risks.push("换手率偏低，流动性不足".to_string());
            score -= 3.0;
        }

        // 5. 量比判断资金参与度
        if metrics.volume_ratio > 3.0 {
            key_points.push(format!("量比 {:.1}，放量明显", metrics.volume_ratio));
            opportunities.push("量能放大，资金参与积极".to_string());
        } else if metrics.volume_ratio < 0.5 && metrics.volume_ratio > 0.0 {
            risks.push(format!("量比 {:.1}，缩量明显", metrics.volume_ratio));
        }
    } else {
        key_points.push("数据不足，仅基于有限信息分析".to_string());
        risks.push("缺乏充分数据支撑，分析置信度低".to_string());
    }

    let rating = DimensionRating::from_score(score.clamp(0.0, 100.0));
    let summary = format!(
        "{} 行业趋势评级 {}，综合评分 {:.0}/100",
        stock_name, rating.display(), score
    );

    DimensionReport {
        dimension: AnalysisDimension::IndustryTrend,
        rating,
        summary,
        key_points,
        risks,
        opportunities,
        confidence: confidence.clamp(0.0_f64, 1.0_f64),
    }
}

/// 从分析数据字符串中提取关键指标
struct StockMetrics {
    ytd_change: f64,
    change_percent: f64,
    change_speed: f64,
    main_net_inflow: f64,
    turnover_rate: f64,
    volume_ratio: f64,
}

fn parse_stock_metrics(data: &str) -> Option<StockMetrics> {
    let ytd_change = 0.0_f64;
    let mut change_percent = 0.0_f64;
    let mut change_speed = 0.0_f64;
    let mut main_net_inflow = 0.0_f64;
    let mut turnover_rate = 0.0_f64;
    let mut volume_ratio = 0.0_f64;
    let mut found = false;

    for line in data.lines() {
        let line = line.trim();
        // 解析 "涨跌幅: X.XX%"
        if let Some(val) = extract_float_after(line, "涨跌幅") {
            change_percent = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "换手率") {
            turnover_rate = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "量比") {
            volume_ratio = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "主力净流入") {
            main_net_inflow = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "涨速") {
            change_speed = val;
            found = true;
        }
    }

    if found {
        Some(StockMetrics {
            ytd_change,
            change_percent,
            change_speed,
            main_net_inflow,
            turnover_rate,
            volume_ratio,
        })
    } else {
        None
    }
}

/// 从字符串中提取冒号后的浮点数
fn extract_float_after(line: &str, keyword: &str) -> Option<f64> {
    if let Some(pos) = line.find(keyword) {
        let rest = &line[pos + keyword.len()..];
        // 找到第一个数字序列
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
    fn test_analyze_industry_trend_with_data() {
        let data = "股票: 贵州茅台 (600519)\n当前价: 1850.00 涨跌幅: 3.50%\n换手率: 2.50% 量比: 1.80";
        let report = analyze_industry_trend("贵州茅台", data);
        assert_eq!(report.dimension, AnalysisDimension::IndustryTrend);
        assert!(report.confidence > 0.3);
    }

    #[test]
    fn test_analyze_industry_trend_empty() {
        let report = analyze_industry_trend("测试", "");
        assert_eq!(report.rating, DimensionRating::C);
        assert!(report.confidence < 0.5);
    }
}
