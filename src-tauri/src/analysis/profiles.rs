//! 分析预设 + Prompt 常量

use serde::{Deserialize, Serialize};

/// 分析预设
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub dimensions: Vec<String>,
    pub max_tokens: u32,
    pub temperature: f64,
}

/// 默认分析预设
pub fn default_presets() -> Vec<AnalysisPreset> {
    vec![
        AnalysisPreset {
            id: "quick".to_string(),
            name: "快速分析".to_string(),
            description: "仅核心3维度（估值/成长/财务），5分钟内完成".to_string(),
            dimensions: vec!["Valuation".to_string(), "GrowthPotential".to_string(), "FinancialHealth".to_string()],
            max_tokens: 50_000,
            temperature: 0.3,
        },
        AnalysisPreset {
            id: "standard".to_string(),
            name: "标准分析".to_string(),
            description: "7维度全量分析，含多空辩论".to_string(),
            dimensions: vec![
                "IndustryTrend".to_string(),
                "CompetitivePosition".to_string(),
                "FinancialHealth".to_string(),
                "ManagementQuality".to_string(),
                "GrowthPotential".to_string(),
                "Valuation".to_string(),
                "TechnicalSignals".to_string(),
            ],
            max_tokens: 100_000,
            temperature: 0.3,
        },
        AnalysisPreset {
            id: "deep".to_string(),
            name: "深度分析".to_string(),
            description: "7维度深度分析+历史对比+行业横向".to_string(),
            dimensions: vec![
                "IndustryTrend".to_string(),
                "CompetitivePosition".to_string(),
                "FinancialHealth".to_string(),
                "ManagementQuality".to_string(),
                "GrowthPotential".to_string(),
                "Valuation".to_string(),
                "TechnicalSignals".to_string(),
            ],
            max_tokens: 200_000,
            temperature: 0.2,
        },
    ]
}
