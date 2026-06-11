//! 结构化输出降级 + Token 预算

use serde::{Deserialize, Serialize};

use super::engine::CloudProvider;

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

/// 结构化输出降级策略
///
/// Level 1: Native — 使用 response_format: {"type": "json_object"}（最佳精度）
/// Level 2: TextWithJsonExtract — 自由文本 + JSON 提取（降级）
/// Level 3: RawText — 纯文本，不解析 JSON（兜底）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum StructuredOutputMode {
    /// 使用 response_format: {"type": "json_object"}（供应商原生支持）
    Native,
    /// 请求自由文本，从响应中提取 JSON
    TextWithJsonExtract,
    /// 接受纯文本，跳过结构化解析
    RawText,
}

impl StructuredOutputMode {
    /// 根据供应商能力、模型特性和 Thinking 模式，确定结构化输出策略
    ///
    /// 核心规则：DeepSeek Thinking 模式 + response_format = 不兼容
    pub fn resolve(provider: CloudProvider, model: &str, thinking_enabled: bool) -> Self {
        let model_lower = model.to_lowercase();

        // 规则 1: DeepSeek Thinking 模式与 response_format 不兼容
        if thinking_enabled && provider == CloudProvider::DeepSeek {
            return Self::TextWithJsonExtract;
        }

        // 规则 2: 按供应商能力判断
        match provider {
            CloudProvider::OpenAI => {
                if model_lower.contains("gpt-3.5") {
                    Self::TextWithJsonExtract
                } else {
                    Self::Native
                }
            }
            CloudProvider::Anthropic => Self::Native,
            CloudProvider::DeepSeek => {
                if model_lower.contains("reasoner") {
                    Self::TextWithJsonExtract
                } else {
                    Self::Native
                }
            }
            CloudProvider::Qwen | CloudProvider::GLM | CloudProvider::MiniMax => Self::Native,
            CloudProvider::Ollama => Self::TextWithJsonExtract,
        }
    }
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
