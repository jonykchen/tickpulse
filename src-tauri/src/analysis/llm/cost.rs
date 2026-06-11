//! LLM 成本估算 + 中文 Token 比率校准
//!
//! 各供应商定价（每百万 token，单位：元人民币）
//! 数据来源：各供应商官方 API 定价页，2026 年 6 月

use super::super::engine::CloudProvider;
use super::TokenUsage;

/// 成本估算器
pub struct CostEstimator;

impl CostEstimator {
    /// 获取供应商/模型的定价 (input_price_per_million, output_price_per_million)
    /// 单位：元人民币 / 百万 token
    ///
    /// 注意：价格可能随供应商调整，此处为参考值
    pub fn pricing(provider: CloudProvider, model: &str) -> (f64, f64) {
        let model_lower = model.to_lowercase();

        match provider {
            CloudProvider::Anthropic => {
                if model_lower.contains("opus") {
                    (112.0, 560.0)   // Claude Opus
                } else if model_lower.contains("sonnet") {
                    (21.0, 105.0)    // Claude Sonnet
                } else {
                    (1.4, 7.0)       // Claude Haiku
                }
            }
            CloudProvider::OpenAI => {
                if model_lower.contains("gpt-4") {
                    (21.0, 63.0)     // GPT-4 系列
                } else {
                    (0.7, 2.1)       // GPT-3.5
                }
            }
            CloudProvider::DeepSeek => {
                if model_lower.contains("reasoner") || model_lower.contains("pro") {
                    (2.0, 8.0)       // DeepSeek Reasoner/Pro
                } else if model_lower.contains("chat") || model_lower.contains("v3") {
                    (0.7, 1.4)       // DeepSeek Chat/V3
                } else {
                    (0.7, 1.4)       // DeepSeek 默认
                }
            }
            CloudProvider::Qwen => {
                if model_lower.contains("max") {
                    (14.0, 28.0)     // Qwen-Max
                } else if model_lower.contains("plus") {
                    (1.4, 2.8)       // Qwen-Plus
                } else if model_lower.contains("turbo") {
                    (0.3, 0.6)       // Qwen-Turbo
                } else {
                    (1.4, 2.8)       // Qwen 默认
                }
            }
            CloudProvider::GLM => {
                if model_lower.contains("4") {
                    (7.0, 14.0)      // GLM-4
                } else {
                    (0.7, 0.7)       // GLM-4-Flash（免费）
                }
            }
            CloudProvider::MiniMax => {
                (0.7, 0.7)           // MiniMax 价格极低
            }
            CloudProvider::Ollama => {
                (0.0, 0.0)           // 本地模型无 API 成本
            }
        }
    }

    /// 估算单次请求成本
    /// 返回值单位：元人民币
    pub fn estimate_cost(provider: CloudProvider, model: &str, usage: &TokenUsage) -> f64 {
        let (input_price, output_price) = Self::pricing(provider, model);
        let input_cost = (usage.prompt_tokens as f64 / 1_000_000.0) * input_price;
        let output_cost = (usage.completion_tokens as f64 / 1_000_000.0) * output_price;
        input_cost + output_cost
    }

    /// 格式化成本为可读字符串
    pub fn format_cost(cost: f64) -> String {
        if cost < 0.01 {
            format!("¥{:.4}", cost)
        } else if cost < 1.0 {
            format!("¥{:.2}", cost)
        } else {
            format!("¥{:.2}", cost)
        }
    }
}

/// 中文 Token 比率校准
///
/// 不同供应商的中文 token 编码效率不同：
/// - GLM: ~1 token ≈ 1.6 个汉字
/// - DeepSeek/Qwen: ~1 token ≈ 1.5 个汉字
/// - Anthropic/OpenAI: ~1 token ≈ 0.8 个汉字（中文效率较低）
///
/// 用于成本预估时校准 token 数量
pub fn adjust_for_chinese(provider: CloudProvider, char_count: usize) -> u32 {
    let ratio = match provider {
        CloudProvider::GLM => 1.6,
        CloudProvider::DeepSeek | CloudProvider::Qwen | CloudProvider::MiniMax => 1.5,
        CloudProvider::Anthropic | CloudProvider::OpenAI => 0.8,
        CloudProvider::Ollama => 1.0, // 取决于本地模型，默认 1.0
    };
    (char_count as f64 / ratio) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pricing_deepseek() {
        let (input, output) = CostEstimator::pricing(CloudProvider::DeepSeek, "deepseek-chat");
        assert!(input > 0.0);
        assert!(output > 0.0);
    }

    #[test]
    fn test_pricing_qwen() {
        let (input, output) = CostEstimator::pricing(CloudProvider::Qwen, "qwen-plus");
        assert!(input > 0.0);
        assert!(output > 0.0);
    }

    #[test]
    fn test_pricing_ollama_free() {
        let (input, output) = CostEstimator::pricing(CloudProvider::Ollama, "llama3");
        assert_eq!(input, 0.0);
        assert_eq!(output, 0.0);
    }

    #[test]
    fn test_estimate_cost() {
        let usage = TokenUsage {
            prompt_tokens: 10_000,
            completion_tokens: 2_000,
            total_tokens: 12_000,
        };
        let cost = CostEstimator::estimate_cost(CloudProvider::DeepSeek, "deepseek-chat", &usage);
        assert!(cost > 0.0);
        // DeepSeek chat: input 0.7/百万, output 1.4/百万
        // expected ≈ 10000/1000000 * 0.7 + 2000/1000000 * 1.4 = 0.007 + 0.0028 = 0.0098
        assert!(cost < 0.1);
    }

    #[test]
    fn test_adjust_for_chinese_glm() {
        // GLM: 1 token ≈ 1.6 个汉字 → 1600 个汉字 ≈ 1000 tokens
        let tokens = adjust_for_chinese(CloudProvider::GLM, 1600);
        assert_eq!(tokens, 1000);
    }

    #[test]
    fn test_adjust_for_chinese_qwen() {
        // Qwen: 1 token ≈ 1.5 个汉字 → 1500 个汉字 ≈ 1000 tokens
        let tokens = adjust_for_chinese(CloudProvider::Qwen, 1500);
        assert_eq!(tokens, 1000);
    }

    #[test]
    fn test_format_cost() {
        assert!(CostEstimator::format_cost(0.005).contains("0.005"));
        assert!(CostEstimator::format_cost(1.23).contains("1.23"));
    }
}