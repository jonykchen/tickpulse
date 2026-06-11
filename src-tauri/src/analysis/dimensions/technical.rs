//! 技术面信号分析维度
//! 基于涨跌幅/涨速/量比/换手率/封板信息评估技术面

use crate::analysis::engine::{AnalysisDimension, DimensionRating, DimensionReport};

/// 技术面信号分析
pub fn analyze_technical_signals(
    stock_name: &str,
    data: &str,
) -> DimensionReport {
    let mut score = 50.0_f64;
    let mut key_points = Vec::new();
    let mut risks = Vec::new();
    let mut opportunities = Vec::new();
    let mut confidence = 0.3;

    if let Some(metrics) = parse_metrics(data) {
        confidence = 0.7; // 技术面数据充分

        // 1. 涨跌幅判断趋势
        if metrics.change_percent > 7.0 {
            score += 12.0;
            key_points.push(format!("涨幅 {:.1}%，强势上涨", metrics.change_percent));
            opportunities.push("短线强势，动量充足".to_string());
        } else if metrics.change_percent > 3.0 {
            score += 6.0;
            key_points.push(format!("涨幅 {:.1}%，偏强", metrics.change_percent));
        } else if metrics.change_percent > 0.0 {
            score += 2.0;
            key_points.push(format!("涨幅 {:.1}%，温和上涨", metrics.change_percent));
        } else if metrics.change_percent > -3.0 {
            score -= 2.0;
            key_points.push(format!("跌幅 {:.1}%，温和回调", metrics.change_percent));
        } else if metrics.change_percent > -7.0 {
            score -= 6.0;
            risks.push(format!("跌幅 {:.1}%，偏弱", metrics.change_percent));
        } else {
            score -= 12.0;
            risks.push(format!("跌幅 {:.1}%，弱势下跌", metrics.change_percent));
        }

        // 2. 涨速（异动检测）
        if metrics.change_speed > 3.0 {
            key_points.push(format!("涨速 {:.2}，异动拉升中", metrics.change_speed));
            opportunities.push("异动拉升，短线交易机会".to_string());
            score += 3.0;
        } else if metrics.change_speed < -3.0 {
            risks.push(format!("涨速 {:.2}，加速下跌中", metrics.change_speed));
            score -= 5.0;
        }

        // 3. 量比 — 量能分析
        if metrics.volume_ratio > 5.0 {
            key_points.push(format!("量比 {:.1}，巨量成交", metrics.volume_ratio));
            if metrics.change_percent > 0.0 {
                opportunities.push("放量上涨，突破信号".to_string());
                score += 5.0;
            } else {
                risks.push("放量下跌，抛压沉重".to_string());
                score -= 5.0;
            }
        } else if metrics.volume_ratio > 2.0 {
            key_points.push(format!("量比 {:.1}，放量", metrics.volume_ratio));
            score += 3.0;
        } else if metrics.volume_ratio > 0.0 && metrics.volume_ratio < 0.5 {
            key_points.push(format!("量比 {:.1}，缩量", metrics.volume_ratio));
            if metrics.change_percent.abs() < 1.0 {
                key_points.push("缩量横盘，可能酝酿变盘".to_string());
            }
        }

        // 4. 换手率 — 筹码活跃度
        if metrics.turnover_rate > 15.0 {
            key_points.push(format!("换手率 {:.1}%，极度活跃", metrics.turnover_rate));
            risks.push("超高换手率，短线博弈激烈，风险高".to_string());
        } else if metrics.turnover_rate > 5.0 {
            key_points.push(format!("换手率 {:.1}%，活跃", metrics.turnover_rate));
        } else if metrics.turnover_rate > 0.0 && metrics.turnover_rate < 1.0 {
            key_points.push(format!("换手率 {:.1}%，低迷", metrics.turnover_rate));
            risks.push("低换手率，流动性不足".to_string());
        }

        // 5. 涨跌停判断
        if metrics.is_limit_up {
            key_points.push("涨停板".to_string());
            if metrics.seal_strength > 0.8 {
                opportunities.push("封板强度高，次日溢价概率大".to_string());
                score += 5.0;
            } else {
                risks.push("封板不牢，炸板风险".to_string());
            }
            if metrics.seal_break_count > 0 {
                risks.push(format!("已炸板 {} 次，多空分歧大", metrics.seal_break_count));
            }
        } else if metrics.is_limit_down {
            risks.push("跌停板，极端弱势".to_string());
            score -= 10.0;
        } else if metrics.is_near_limit_up {
            key_points.push("接近涨停，短线强势".to_string());
            opportunities.push("接近涨停，关注能否封板".to_string());
            score += 3.0;
        }

        // 6. 临时停牌
        if metrics.is_temp_suspended {
            risks.push("临时停牌中，交易受限".to_string());
            score -= 5.0;
        }
    } else {
        key_points.push("技术面数据不足".to_string());
        risks.push("缺乏行情数据，技术分析受限".to_string());
    }

    let rating = DimensionRating::from_score(score.clamp(0.0, 100.0));
    let summary = format!(
        "{} 技术面评级 {}，综合评分 {:.0}/100",
        stock_name, rating.display(), score
    );

    DimensionReport {
        dimension: AnalysisDimension::TechnicalSignals,
        rating,
        summary,
        key_points,
        risks,
        opportunities,
        confidence: confidence.clamp(0.0, 1.0),
    }
}

struct ParsedMetrics {
    change_percent: f64,
    change_speed: f64,
    volume_ratio: f64,
    turnover_rate: f64,
    is_limit_up: bool,
    is_limit_down: bool,
    is_near_limit_up: bool,
    is_temp_suspended: bool,
    seal_strength: f64,
    seal_break_count: u32,
}

fn parse_metrics(data: &str) -> Option<ParsedMetrics> {
    let mut change_percent = 0.0;
    let mut change_speed = 0.0;
    let mut volume_ratio = 0.0;
    let mut turnover_rate = 0.0;
    let mut seal_strength = 0.0;
    let mut seal_break_count = 0u32;
    let mut found = false;

    for line in data.lines() {
        let line = line.trim();
        if let Some(val) = extract_float_after(line, "涨跌幅") {
            change_percent = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "涨速") {
            change_speed = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "量比") {
            volume_ratio = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "换手率") {
            turnover_rate = val;
            found = true;
        }
        if let Some(val) = extract_float_after(line, "封板强度") {
            seal_strength = val;
            found = true;
        }
    }

    if found {
        Some(ParsedMetrics {
            change_percent,
            change_speed,
            volume_ratio,
            turnover_rate,
            is_limit_up: change_percent >= 9.9,
            is_limit_down: change_percent <= -9.9,
            is_near_limit_up: change_percent >= 8.0,
            is_temp_suspended: data.contains("临时停牌"),
            seal_strength,
            seal_break_count,
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
    fn test_technical_analysis() {
        let data = "涨跌幅: 3.50%\n涨速: 0.80\n量比: 2.50\n换手率: 4.50%";
        let report = analyze_technical_signals("贵州茅台", data);
        assert_eq!(report.dimension, AnalysisDimension::TechnicalSignals);
        assert!(report.confidence > 0.5);
    }

    #[test]
    fn test_technical_limit_up() {
        let data = "涨跌幅: 10.00%\n量比: 8.50\n换手率: 12.00%\n封板强度: 0.95";
        let report = analyze_technical_signals("涨停股", data);
        assert!(report.key_points.iter().any(|p| p.contains("涨停")));
    }
}
