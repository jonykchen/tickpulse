//! Ollama 本地 LLM 客户端

use async_trait::async_trait;
use serde::Deserialize;

use super::{ChatMessage, LlmClient, LlmResponse, MessageRole, TokenUsage};
use crate::analysis::engine::LlmConfig;

/// Ollama Chat 响应
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
    model: String,
    prompt_eval_count: Option<u32>,
    eval_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}

/// Ollama 客户端
pub struct OllamaClient {
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120)) // 本地推理可能较慢
            .build()
            .expect("Failed to build Ollama HTTP client");
        Self { client }
    }
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn chat(&self, messages: &[ChatMessage], config: &LlmConfig) -> Result<LlmResponse, String> {
        let base_url = config.base_url.as_deref().unwrap_or("http://localhost:11434");
        let url = format!("{}/api/chat", base_url);

        let chat_msgs: Vec<serde_json::Value> = messages.iter()
            .map(|m| serde_json::json!({
                "role": match m.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                },
                "content": m.content,
            }))
            .collect();

        let body = serde_json::json!({
            "model": config.model,
            "messages": chat_msgs,
            "stream": false,
        });

        let resp = self.client.post(&url)
            .header("content-type", "application/json")
            .json(&body)
            .send().await
            .map_err(|e| format!("Ollama 请求失败: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(format!("Ollama 错误 ({}): {}", status, error_text));
        }

        let api_resp: OllamaChatResponse = resp.json().await
            .map_err(|e| format!("Ollama 解析失败: {}", e))?;

        let prompt_tokens = api_resp.prompt_eval_count.unwrap_or(0);
        let completion_tokens = api_resp.eval_count.unwrap_or(0);

        Ok(LlmResponse {
            content: api_resp.message.content,
            reasoning: None,
            usage: TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
            model: api_resp.model,
        })
    }

    fn provider_name(&self) -> &'static str {
        "ollama"
    }
}