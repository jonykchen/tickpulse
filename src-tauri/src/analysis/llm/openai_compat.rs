//! OpenAI 兼容客户端（支持 DeepSeek reasoning_content 回传 + Thinking 模式控制）
//!
//! 支持的供应商：OpenAI / DeepSeek / Qwen / GLM / MiniMax
//! 均使用 OpenAI Chat Completions 兼容 API

use async_trait::async_trait;
use serde::Deserialize;

use super::{ChatMessage, LlmClient, LlmResponse, MessageRole, TokenUsage};
use crate::analysis::engine::{CloudProvider, LlmConfig};
use crate::analysis::schemas::StructuredOutputMode;

/// OpenAI Chat Completions API 响应
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    usage: OpenAIUsage,
    model: String,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: Option<String>,
    reasoning_content: Option<String>, // DeepSeek 特有
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI 兼容客户端
pub struct OpenAICompatClient {
    client: reqwest::Client,
}

impl OpenAICompatClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to build OpenAI HTTP client");
        Self { client }
    }

    /// DeepSeek Reasoner 模型要求：thinking模式下，下次调用必须原样回传 reasoning_content
    /// 否则 HTTP 400 错误
    ///
    /// # Arguments
    /// * `messages` - 消息列表，会被修改
    /// * `last_reasoning` - 上一次响应中的 reasoning_content
    ///
    /// # DeepSeek 格式要求
    /// 当使用 DeepSeek Reasoner 模型（如 deepseek-reasoner）时，
    /// 需要在最后一条 assistant 消息中添加 reasoning_content 字段
    pub fn adapt_request_for_deepseek(&self, messages: &mut Vec<ChatMessage>, last_reasoning: Option<&str>) {
        if let Some(reasoning) = last_reasoning {
            // 查找最后一条 assistant 消息
            if let Some(last_assistant) = messages.iter_mut().rev().find(|m| m.role == MessageRole::Assistant) {
                // DeepSeek 要求在 assistant 消息中添加 reasoning_content
                // 这里我们使用特殊标记来存储，实际发送时会转换为正确格式
                // 格式: [REASONING]...[/REASONING]\n原始内容
                if !last_assistant.content.contains("[REASONING]") {
                    last_assistant.content = format!("[REASONING]{}[/REASONING]\n{}", reasoning, last_assistant.content);
                }
            }
        }
    }

    /// 判断是否支持结构化输出（JSON Schema / Response Format）
    ///
    /// # Returns
    /// * `true` - 支持（OpenAI、Anthropic、DeepSeek 非 reasoner 模型、Qwen/GLM/MiniMax）
    /// * `false` - 不支持（DeepSeek Reasoner、Ollama）
    pub fn supports_structured_output(&self, model: &str) -> bool {
        let model_lower = model.to_lowercase();

        // DeepSeek Reasoner 系列不支持结构化输出
        if model_lower.contains("reasoner") {
            return false;
        }

        true
    }

    /// 根据供应商和模型判断是否支持结构化输出
    ///
    /// # Arguments
    /// * `provider` - 供应商类型
    /// * `model` - 模型名称
    pub fn supports_structured_output_for_provider(
        &self,
        provider: CloudProvider,
        model: &str,
    ) -> bool {
        let model_lower = model.to_lowercase();

        match provider {
            CloudProvider::OpenAI => {
                // OpenAI 支持（需要 gpt-4-turbo 及以上版本）
                !model_lower.contains("gpt-3.5")
            }
            CloudProvider::Anthropic => {
                // Anthropic 支持
                true
            }
            CloudProvider::DeepSeek => {
                // DeepSeek: 只有非 reasoner 模型支持
                !model_lower.contains("reasoner")
            }
            CloudProvider::Qwen | CloudProvider::GLM | CloudProvider::MiniMax => {
                // Qwen/GLM/MiniMax 通过 OpenAI 兼容 API 支持结构化输出
                true
            }
            CloudProvider::Ollama => {
                // Ollama 当前版本不支持结构化输出
                false
            }
        }
    }

    /// 适配响应：归一化 content，提取 reasoning_content
    ///
    /// # Arguments
    /// * `response` - API 原始响应（JSON）
    ///
    /// # Returns
    /// * 归一化后的 content 内容
    /// * 提取的 reasoning_content（如果有）
    pub fn adapt_response(&self, response: &mut serde_json::Value) -> (String, Option<String>) {
        let mut content = String::new();
        let mut reasoning: Option<String> = None;

        // 提取 choices[0].message.content
        if let Some(choices) = response.get("choices").and_then(|c| c.as_array()) {
            if let Some(first_choice) = choices.first() {
                if let Some(message) = first_choice.get("message") {
                    // 提取 content
                    content = message.get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or_default()
                        .to_string();

                    // 提取 reasoning_content（DeepSeek 特有）
                    reasoning = message.get("reasoning_content")
                        .and_then(|r| r.as_str())
                        .map(|s| s.to_string());

                    // 处理 Anthropic typed blocks 格式
                    // Anthropic 返回 content 为数组，每个元素有 type 和 text
                    if content.is_empty() {
                        if let Some(blocks) = message.get("content").and_then(|c| c.as_array()) {
                            let texts: Vec<String> = blocks.iter()
                                .filter_map(|b| {
                                    if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                                        b.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            content = texts.join("\n");
                        }
                    }
                }
            }
        }

        (content, reasoning)
    }

    /// 将消息转换为 DeepSeek 兼容的请求格式
    ///
    /// # Arguments
    /// * `messages` - 原始消息列表
    ///
    /// # Returns
    /// * DeepSeek 格式的消息 JSON 数组
    fn messages_to_deepseek_format(&self, messages: &[ChatMessage]) -> Vec<serde_json::Value> {
        messages.iter().map(|m| {
            // 检查内容中是否包含 [REASONING] 标记
            let (content, reasoning) = if m.content.starts_with("[REASONING]") && m.role == MessageRole::Assistant {
                // 提取 reasoning 和实际内容
                let parts: Vec<&str> = m.content.splitn(3, &['[', ']']).collect();
                if parts.len() >= 4 {
                    let reasoning_content = parts[2].trim_start_matches("REASONING]").trim_end_matches("/").to_string();
                    let actual_content = if parts.len() > 4 { parts[4].trim().to_string() } else { String::new() };
                    (actual_content, Some(reasoning_content))
                } else {
                    (m.content.clone(), None)
                }
            } else {
                (m.content.clone(), None)
            };

            let mut msg = serde_json::json!({
                "role": match m.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                },
                "content": content,
            });

            // DeepSeek 要求 reasoning_content 作为单独字段
            if let Some(reasoning_content) = reasoning {
                msg["reasoning_content"] = serde_json::json!(reasoning_content);
            }

            msg
        }).collect()
    }

    /// 将消息转换为标准 OpenAI 格式
    fn messages_to_standard_format(&self, messages: &[ChatMessage]) -> Vec<serde_json::Value> {
        messages.iter()
            .map(|m| serde_json::json!({
                "role": match m.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                },
                "content": m.content,
            }))
            .collect()
    }
}

#[async_trait]
impl LlmClient for OpenAICompatClient {
    async fn chat(&self, messages: &[ChatMessage], config: &LlmConfig) -> Result<LlmResponse, String> {
        let api_key = config.api_key.as_ref().ok_or("API Key 未配置")?;

        // 使用供应商默认 URL（如果未配置自定义 URL）
        let base_url = config.base_url.as_deref()
            .unwrap_or_else(|| config.provider.default_base_url());
        let url = format!("{}/chat/completions", base_url);

        // 判断是否为 DeepSeek 供应商（需要特殊消息格式）
        let is_deepseek = config.provider == CloudProvider::DeepSeek;

        // DeepSeek 使用特殊格式处理 reasoning_content
        let chat_msgs = if is_deepseek {
            self.messages_to_deepseek_format(messages)
        } else {
            self.messages_to_standard_format(messages)
        };

        let mut body = serde_json::json!({
            "model": config.model,
            "messages": chat_msgs,
            "max_tokens": 4096,
        });

        // === Thinking 模式控制 ===
        // DeepSeek Thinking 模式与 response_format 不兼容
        if config.thinking_enabled && is_deepseek {
            // 开启 Thinking 模式：注入 reasoning_effort，不设 response_format
            body["reasoning_effort"] = serde_json::json!("high");
            tracing::debug!(
                "DeepSeek Thinking 模式已启用，跳过 response_format（不兼容）"
            );
        } else {
            // === 结构化输出控制 ===
            // 根据 StructuredOutputMode::resolve() 决定是否设置 response_format
            let structured_mode = StructuredOutputMode::resolve(
                config.provider,
                &config.model,
                config.thinking_enabled,
            );

            match structured_mode {
                StructuredOutputMode::Native => {
                    body["response_format"] = serde_json::json!({"type": "json_object"});
                }
                StructuredOutputMode::TextWithJsonExtract => {
                    // 不设 response_format，依赖 json_extract 从文本中提取 JSON
                    // 在 system prompt 中注入 JSON 输出指令（如果尚未包含）
                    tracing::debug!(
                        "结构化输出降级为 TextWithJsonExtract 模式（供应商={}, model={})",
                        config.provider.as_str(),
                        config.model
                    );
                }
                StructuredOutputMode::RawText => {
                    // 纯文本模式，不做任何额外处理
                }
            }
        }

        let resp = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send().await
            .map_err(|e| format!("OpenAI 兼容请求失败: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(format!("API 错误 ({}): {}", status, error_text));
        }

        let mut response_json: serde_json::Value = resp.json().await
            .map_err(|e| format!("API 解析失败: {}", e))?;

        // 使用 adapt_response 处理响应
        let (content, reasoning) = self.adapt_response(&mut response_json);

        // 提取 usage 信息
        let usage = response_json.get("usage")
            .map(|u| TokenUsage {
                prompt_tokens: u.get("prompt_tokens").and_then(|p| p.as_u64()).unwrap_or(0) as u32,
                completion_tokens: u.get("completion_tokens").and_then(|p| p.as_u64()).unwrap_or(0) as u32,
                total_tokens: u.get("total_tokens").and_then(|p| p.as_u64()).unwrap_or(0) as u32,
            })
            .unwrap_or_default();

        // 提取 model 信息
        let model = response_json.get("model")
            .and_then(|m| m.as_str())
            .unwrap_or(&config.model)
            .to_string();

        Ok(LlmResponse {
            content,
            reasoning,
            usage,
            model,
        })
    }

    fn provider_name(&self) -> &'static str {
        "openai-compat"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_structured_output() {
        let client = OpenAICompatClient::new();

        // DeepSeek Reasoner 不支持
        assert!(!client.supports_structured_output("deepseek-reasoner"));
        assert!(!client.supports_structured_output("DEEPSEEK-REASONER"));

        // 普通模型支持
        assert!(client.supports_structured_output("gpt-4"));
        assert!(client.supports_structured_output("deepseek-chat"));
        assert!(client.supports_structured_output("qwen-plus"));
        assert!(client.supports_structured_output("glm-4"));
    }

    #[test]
    fn test_supports_structured_output_for_provider() {
        let client = OpenAICompatClient::new();

        // Qwen/GLM/MiniMax 支持
        assert!(client.supports_structured_output_for_provider(CloudProvider::Qwen, "qwen-plus"));
        assert!(client.supports_structured_output_for_provider(CloudProvider::GLM, "glm-4"));
        assert!(client.supports_structured_output_for_provider(CloudProvider::MiniMax, "MiniMax-Text-01"));

        // Ollama 不支持
        assert!(!client.supports_structured_output_for_provider(CloudProvider::Ollama, "llama3"));
    }

    #[test]
    fn test_adapt_response_with_reasoning() {
        let client = OpenAICompatClient::new();

        let mut response = serde_json::json!({
            "choices": [{
                "message": {
                    "content": "Final answer",
                    "reasoning_content": "Thinking process..."
                }
            }],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "total_tokens": 150
            },
            "model": "deepseek-reasoner"
        });

        let (content, reasoning) = client.adapt_response(&mut response);

        assert_eq!(content, "Final answer");
        assert_eq!(reasoning, Some("Thinking process...".to_string()));
    }

    #[test]
    fn test_adapt_request_for_deepseek() {
        let client = OpenAICompatClient::new();

        let mut messages = vec![
            ChatMessage { role: MessageRole::User, content: "Hello".to_string() },
            ChatMessage { role: MessageRole::Assistant, content: "Hi there".to_string() },
        ];

        client.adapt_request_for_deepseek(&mut messages, Some("My reasoning..."));

        // 检查最后一条 assistant 消息被修改
        assert!(messages[1].content.contains("[REASONING]"));
        assert!(messages[1].content.contains("My reasoning..."));
        assert!(messages[1].content.contains("Hi there"));
    }

    #[test]
    fn test_structured_output_mode_deepseek_thinking() {
        // DeepSeek + thinking_enabled = TextWithJsonExtract（不兼容）
        let mode = StructuredOutputMode::resolve(
            CloudProvider::DeepSeek,
            "deepseek-reasoner",
            true,
        );
        assert_eq!(mode, StructuredOutputMode::TextWithJsonExtract);
    }

    #[test]
    fn test_structured_output_mode_deepseek_no_thinking() {
        // DeepSeek 非 reasoner + thinking_disabled = Native
        let mode = StructuredOutputMode::resolve(
            CloudProvider::DeepSeek,
            "deepseek-chat",
            false,
        );
        assert_eq!(mode, StructuredOutputMode::Native);
    }

    #[test]
    fn test_structured_output_mode_qwen() {
        // Qwen 始终支持 Native
        let mode = StructuredOutputMode::resolve(CloudProvider::Qwen, "qwen-plus", false);
        assert_eq!(mode, StructuredOutputMode::Native);
    }

    #[test]
    fn test_structured_output_mode_glm() {
        let mode = StructuredOutputMode::resolve(CloudProvider::GLM, "glm-4", false);
        assert_eq!(mode, StructuredOutputMode::Native);
    }

    #[test]
    fn test_structured_output_mode_minimax() {
        let mode = StructuredOutputMode::resolve(CloudProvider::MiniMax, "MiniMax-Text-01", false);
        assert_eq!(mode, StructuredOutputMode::Native);
    }
}