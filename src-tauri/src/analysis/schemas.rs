//! 结构化输出降级 + Token 预算

use serde::{Deserialize, Serialize};

/// 结构化输出 schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionOutputSchema {
    pub dimension: String,
    pub rating: String,
    pub summary: String,
    pub key_points: Vec<String>,
    pub risks: Vec<String>,
    pub opportunities: Vec<String>,
    pub confidence: f64,
}

/// Token 预算分配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    /// 总预算
    pub total_tokens: u32,
    /// 系统提示词预算
    pub system_prompt_tokens: u32,
    /// 每维度分配
    pub per_dimension_tokens: u32,
    /// 辩论预算
    pub debate_tokens: u32,
    /// 综合结论预算
    pub summary_tokens: u32,
}

impl TokenBudget {
    /// 根据总预算自动分配
    pub fn allocate(total_tokens: u32, num_dimensions: u32) -> Self {
        let system_tokens = total_tokens / 10; // 10% for system prompt
        let debate_tokens = total_tokens / 5;  // 20% for debate
        let summary_tokens = total_tokens / 10; // 10% for summary
        let remaining = total_tokens - system_tokens - debate_tokens - summary_tokens;
        let per_dimension = if num_dimensions > 0 {
            remaining / num_dimensions
        } else {
            remaining
        };

        Self {
            total_tokens,
            system_prompt_tokens: system_tokens,
            per_dimension_tokens: per_dimension,
            debate_tokens,
            summary_tokens,
        }
    }

    /// 默认预算（100k tokens）
    pub fn default_budget() -> Self {
        Self::allocate(100_000, 7)
    }
}