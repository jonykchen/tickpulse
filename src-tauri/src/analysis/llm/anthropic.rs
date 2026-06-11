//! Anthropic Claude 客户端适配

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{ChatMessage, LlmClient, LlmResponse, MessageRole, TokenUsage};
use crate::analysis::engine::LlmConfig;

/// Anthropic Claude API 响应结构
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    usage: AnthropicUsage,
    model: String,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    text: Option<String>,
    #[serde(rename = "type")]
    block_type: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic 客户端
pub struct AnthropicClient {
    client: reqwest::Client,
}

impl AnthropicClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to build Anthropic HTTP client");
        Self { client }
    }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn chat(&self, messages: &[ChatMessage], config: &LlmConfig) -> Result<LlmResponse, String> {
        let api_key = config.api_key.as_ref().ok_or("Anthropic API Key 未配置")?;

        // 分离 system 消息和 user/assistant 消息
        let system_msg = messages.iter()
            .find(|m| m.role == MessageRole::System)
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let chat_msgs: Vec<serde_json::Value> = messages.iter()
            .filter(|m| m.role != MessageRole::System)
            .map(|m| serde_json::json!({
                "role": match m.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    _ => "user",
                },
                "content": m.content,
            }))
            .collect();

        let body = serde_json::json!({
            "model": config.model,
            "max_tokens": 4096,
            "system": system_msg,
            "messages": chat_msgs,
        });

        let resp = self.client.post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send().await
            .map_err(|e| format!("Anthropic 请求失败: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(format!("Anthropic API 错误 ({}): {}", status, error_text));
        }

        let api_resp: AnthropicResponse = resp.json().await
            .map_err(|e| format!("Anthropic 解析失败: {}", e))?;

        let text = api_resp.content.iter()
            .filter_map(|b| b.text.clone())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(LlmResponse {
            content: text,
            reasoning: None,
            usage: TokenUsage {
                prompt_tokens: api_resp.usage.input_tokens,
                completion_tokens: api_resp.usage.output_tokens,
                total_tokens: api_resp.usage.input_tokens + api_resp.usage.output_tokens,
            },
            model: api_resp.model,
        })
    }

    fn provider_name(&self) -> &'static str {
        "anthropic"
    }
}