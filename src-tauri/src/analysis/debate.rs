//! 多空辩论系统
//! Bull/Bear/Research Manager 三方辩论 + 5 级评级输出

use serde::{Deserialize, Serialize};

use super::engine::{OverallRating, AnalysisDimension};

/// 辩论参与方
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DebateRole {
    /// 多方（看好）
    Bull,
    /// 空方（看空）
    Bear,
    /// 研究经理（中立裁决）
    ResearchManager,
}

/// 辩论论点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateArgument {
    pub role: DebateRole,
    pub dimension: AnalysisDimension,
    pub argument: String,
    pub evidence: Vec<String>,
    pub score: f64, // 0-100
}

/// 辩论结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateResult {
    pub bull_score: f64,
    pub bear_score: f64,
    pub final_score: f64,
    pub overall_rating: OverallRating,
    pub bull_argument: String,
    pub bear_argument: String,
    pub verdict: String,
    pub rounds: u32,
}

/// 辩论引擎
pub struct DebateEngine;

impl DebateEngine {
    /// 基于维度报告执行多空辩论
    pub fn debate(
        dimension_scores: &[(AnalysisDimension, f64)],
        bull_points: &[String],
        bear_points: &[String],
    ) -> DebateResult {
        // 计算加权平均分
        let weights: std::collections::HashMap<AnalysisDimension, f64> = [
            (AnalysisDimension::Valuation, 0.25),
            (AnalysisDimension::GrowthPotential, 0.20),
            (AnalysisDimension::FinancialHealth, 0.20),
            (AnalysisDimension::IndustryTrend, 0.10),
            (AnalysisDimension::CompetitivePosition, 0.10),
            (AnalysisDimension::ManagementQuality, 0.08),
            (AnalysisDimension::TechnicalSignals, 0.07),
        ].into_iter().collect();

        let total_weight: f64 = dimension_scores.iter()
            .map(|(d, _)| weights.get(d).copied().unwrap_or(0.1))
            .sum();

        let weighted_score: f64 = if total_weight > 0.0 {
            dimension_scores.iter()
                .map(|(d, s)| s * weights.get(d).copied().unwrap_or(0.1))
                .sum::<f64>() / total_weight
        } else {
            50.0
        };

        // 多方/空方得分
        let bull_bonus = bull_points.len() as f64 * 2.0;
        let bear_penalty = bear_points.len() as f64 * 2.0;

        let bull_score = (weighted_score + bull_bonus).min(100.0);
        let bear_score = (100.0 - weighted_score + bear_penalty).min(100.0);

        let final_score = weighted_score;
        let overall_rating = OverallRating::from_score(final_score);

        let bull_argument = bull_points.join("；");
        let bear_argument = bear_points.join("；");

        let verdict = format!(
            "综合评分 {:.1}/100，{}。多空比 {:.0}:{:.0}",
            final_score,
            overall_rating.display(),
            bull_score,
            bear_score
        );

        DebateResult {
            bull_score,
            bear_score,
            final_score,
            overall_rating,
            bull_argument,
            bear_argument,
            verdict,
            rounds: 1,
        }
    }
}
