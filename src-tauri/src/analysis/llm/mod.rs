//! LLM 客户端接口 + 供应商适配

pub mod anthropic;
pub mod cost;
pub mod fallback;
pub mod json_extract;
pub mod ollama;
pub mod openai_compat;
pub mod router;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::engine::{CloudProvider, LlmConfig};

/// LLM 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub reasoning: Option<String>,
    pub usage: TokenUsage,
    pub model: String,
}

/// Token 使用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl Default for TokenUsage {
    fn default() -> Self {
        Self {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }
    }
}

/// LLM 客户端 trait
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// 发送聊天请求
    async fn chat(&self, messages: &[ChatMessage], config: &LlmConfig) -> Result<LlmResponse, String>;

    /// 供应商名称
    fn provider_name(&self) -> &'static str;

    /// 简单文本生成（用于质量门控复审等场景）
    async fn generate(&self, prompt: &str) -> Result<String, String> {
        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: prompt.to_string(),
            },
        ];
        let config = LlmConfig {
            provider: CloudProvider::Anthropic,
            api_key: None,
            model: "default".to_string(),
            base_url: None,
            mode: super::engine::LlmMode::Cloud,
            thinking_enabled: false,
        };
        self.chat(&messages, &config).await.map(|r| r.content)
    }
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

/// 消息角色
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// 根据 config 创建对应的 LLM 客户端
pub fn create_client(config: &LlmConfig) -> Box<dyn LlmClient> {
    match config.provider {
        CloudProvider::Anthropic => Box::new(anthropic::AnthropicClient::new()),
        CloudProvider::OpenAI | CloudProvider::DeepSeek => {
            Box::new(openai_compat::OpenAICompatClient::new())
        }
        CloudProvider::Qwen | CloudProvider::GLM | CloudProvider::MiniMax => {
            Box::new(openai_compat::OpenAICompatClient::new())
        }
        CloudProvider::Ollama => Box::new(ollama::OllamaClient::new()),
    }
}

/// Mock LLM 客户端（用于测试）
pub struct MockLlmClient;

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn chat(&self, _messages: &[ChatMessage], _config: &LlmConfig) -> Result<LlmResponse, String> {
        Ok(LlmResponse {
            content: r#"{"dimension":"FinancialHealth","rating":"B","summary":"财务状况良好","key_points":["ROE稳健","现金流充裕"],"risks":["负债率偏高"],"opportunities":["行业增长"],"confidence":0.75}"#.to_string(),
            reasoning: None,
            usage: TokenUsage::default(),
            model: "mock".to_string(),
        })
    }

    fn provider_name(&self) -> &'static str {
        "mock"
    }
}
