//! LLM Router — 根据 TaskComplexity 选择 Quick/Deep 客户端
//! 解析 StructuredOutputMode，协调降级链

use super::fallback::{self, FallbackConfig};
use super::{create_client, ChatMessage, LlmClient, LlmResponse};
use crate::analysis::engine::{DualLlmConfig, LlmConfig, TaskComplexity};
use crate::analysis::schemas::StructuredOutputMode;

/// LLM 路由器：根据任务复杂度选择客户端，支持降级链
pub struct LlmRouter {
    dual_config: DualLlmConfig,
    fallback_config: FallbackConfig,
}

impl LlmRouter {
    /// 创建 LLM 路由器
    pub fn new(dual_config: DualLlmConfig, fallback_config: FallbackConfig) -> Self {
        Self {
            dual_config,
            fallback_config,
        }
    }

    /// 从单一 LlmConfig 创建（向后兼容，无降级链）
    pub fn from_single(config: LlmConfig) -> Self {
        let dual_config = DualLlmConfig::from_single(config);
        let fallback_config = FallbackConfig::from_single(dual_config.quick_think.clone());
        Self {
            dual_config,
            fallback_config,
        }
    }

    /// 获取 DualLlmConfig 引用
    pub fn dual_config(&self) -> &DualLlmConfig {
        &self.dual_config
    }

    /// 根据 TaskComplexity 获取对应的 LlmConfig
    pub fn config_for_complexity(&self, complexity: TaskComplexity) -> &LlmConfig {
        match complexity {
            TaskComplexity::Quick => &self.dual_config.quick_think,
            TaskComplexity::Deep => self.dual_config.deep_think_config(),
        }
    }

    /// 获取当前任务的有效 StructuredOutputMode
    pub fn structured_mode(&self, complexity: TaskComplexity) -> StructuredOutputMode {
        let config = self.config_for_complexity(complexity);
        StructuredOutputMode::resolve(config.provider, &config.model, config.thinking_enabled)
    }

    /// 路由 chat 请求到合适的客户端（含降级链）
    pub async fn route_chat(
        &self,
        messages: &[ChatMessage],
        complexity: TaskComplexity,
    ) -> Result<LlmResponse, String> {
        let config = self.config_for_complexity(complexity);

        // 如果降级链中有多个供应商，使用降级链
        if self.fallback_config.chain.len() > 1 {
            let clients: Vec<Box<dyn LlmClient>> = self.fallback_config.chain.iter()
                .map(create_client)
                .collect();
            fallback::chat_with_fallback(
                messages,
                &clients,
                &self.fallback_config.chain,
                self.fallback_config.max_retries_per_provider,
            ).await
        } else {
            // 单供应商，直接调用
            let client = create_client(config);
            client.chat(messages, config).await
        }
    }

    /// Quick-Think 请求（维度分析、结构化输出）
    pub async fn quick_chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse, String> {
        self.route_chat(messages, TaskComplexity::Quick).await
    }

    /// Deep-Think 请求（辩论、质量门控、综合决策）
    pub async fn deep_chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse, String> {
        self.route_chat(messages, TaskComplexity::Deep).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::engine::{CloudProvider, LlmMode};

    fn make_config(provider: CloudProvider, model: &str) -> LlmConfig {
        LlmConfig {
            provider,
            api_key: Some("test-key".to_string()),
            model: model.to_string(),
            base_url: None,
            mode: LlmMode::Cloud,
            thinking_enabled: false,
        }
    }

    #[test]
    fn test_router_config_for_complexity_quick() {
        let quick = make_config(CloudProvider::DeepSeek, "deepseek-chat");
        let dual = DualLlmConfig::from_single(quick.clone());
        let router = LlmRouter::from_single(quick);

        let config = router.config_for_complexity(TaskComplexity::Quick);
        assert_eq!(config.model, "deepseek-chat");
    }

    #[test]
    fn test_router_config_for_complexity_deep_fallback() {
        let quick = make_config(CloudProvider::DeepSeek, "deepseek-chat");
        let dual = DualLlmConfig::from_single(quick);
        let router = LlmRouter::from_single(dual.quick_think.clone());

        // 无 deep_think 配置时，Deep 降级为 Quick
        let config = router.config_for_complexity(TaskComplexity::Deep);
        assert_eq!(config.model, "deepseek-chat");
    }

    #[test]
    fn test_router_config_for_complexity_deep_configured() {
        let quick = make_config(CloudProvider::DeepSeek, "deepseek-chat");
        let deep = make_config(CloudProvider::DeepSeek, "deepseek-reasoner");
        let dual = DualLlmConfig {
            quick_think: quick,
            deep_think: Some(deep),
        };
        let router = LlmRouter::new(dual, FallbackConfig::default());

        let config = router.config_for_complexity(TaskComplexity::Deep);
        assert_eq!(config.model, "deepseek-reasoner");
    }

    #[test]
    fn test_router_structured_mode() {
        let quick = make_config(CloudProvider::Qwen, "qwen-plus");
        let dual = DualLlmConfig::from_single(quick);
        let router = LlmRouter::from_single(dual.quick_think);

        let mode = router.structured_mode(TaskComplexity::Quick);
        assert_eq!(mode, StructuredOutputMode::Native);
    }

    #[test]
    fn test_router_structured_mode_deepseek_thinking() {
        let quick = LlmConfig {
            provider: CloudProvider::DeepSeek,
            api_key: Some("test-key".to_string()),
            model: "deepseek-reasoner".to_string(),
            base_url: None,
            mode: LlmMode::Cloud,
            thinking_enabled: true,
        };
        let dual = DualLlmConfig::from_single(quick);
        let router = LlmRouter::from_single(dual.quick_think);

        let mode = router.structured_mode(TaskComplexity::Quick);
        assert_eq!(mode, StructuredOutputMode::TextWithJsonExtract);
    }
}