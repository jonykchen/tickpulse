//! 多供应商降级链 — API 失败时自动切换备用供应商
//!
//! 降级链：主选 → 备选 1 → ... → 备选 N → 本地兜底（Ollama）
//!
//! 错误分类：
//! - Transient（429/5xx）: 同供应商重试，超时后切换下一供应商
//! - Permanent（401/403/400）: 跳过当前供应商，立即切换
//! - Network（连接拒绝/DNS 失败）: 立即切换下一供应商

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::{ChatMessage, LlmClient, LlmResponse};
use super::super::engine::LlmConfig;

/// LLM 错误分类
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LlmErrorKind {
    /// 暂时性错误：429 限速、5xx 服务器错误 → 重试或降级
    Transient,
    /// 永久性错误：401/403 认证失败、400 请求无效 → 跳过供应商
    Permanent,
    /// 网络错误：连接拒绝、DNS 失败 → 立即降级
    Network,
}

/// 降级链配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// 降级链：按优先级排列的 LLM 配置列表（第一个为主选）
    pub chain: Vec<LlmConfig>,
    /// 每个供应商最大重试次数（默认 2）
    #[serde(default = "default_max_retries")]
    pub max_retries_per_provider: u32,
    /// 每次请求超时秒数（默认 60）
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_max_retries() -> u32 { 2 }
fn default_timeout_secs() -> u64 { 60 }

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            chain: Vec::new(),
            max_retries_per_provider: 2,
            timeout_secs: 60,
        }
    }
}

impl FallbackConfig {
    /// 从单一 LlmConfig 构造（无降级链）
    pub fn from_single(config: LlmConfig) -> Self {
        Self {
            chain: vec![config],
            ..Default::default()
        }
    }
}

/// 降级链执行结果
#[derive(Debug, Clone)]
pub struct FallbackResult {
    /// 最终响应（成功时）
    pub response: Option<LlmResponse>,
    /// 使用的供应商索引
    pub provider_index: usize,
    /// 总尝试次数
    pub total_attempts: u32,
    /// 各供应商的错误记录
    pub errors: Vec<(usize, String)>,
}

/// 分类 LLM 错误
pub fn classify_error(error: &str) -> LlmErrorKind {
    let error_lower = error.to_lowercase();

    // 认证/授权失败 → Permanent
    if error_lower.contains("401") || error_lower.contains("403") || error_lower.contains("unauthorized") || error_lower.contains("forbidden") {
        return LlmErrorKind::Permanent;
    }

    // 请求格式错误 → Permanent
    if error_lower.contains("400") || error_lower.contains("bad request") || error_lower.contains("invalid request") {
        return LlmErrorKind::Permanent;
    }

    // 限速 → Transient（可重试）
    if error_lower.contains("429") || error_lower.contains("rate limit") || error_lower.contains("too many requests") {
        return LlmErrorKind::Transient;
    }

    // 服务器错误 → Transient
    if error_lower.contains("500") || error_lower.contains("502") || error_lower.contains("503") || error_lower.contains("504")
        || error_lower.contains("server error") || error_lower.contains("internal error") {
        return LlmErrorKind::Transient;
    }

    // 网络错误 → Network
    if error_lower.contains("connection refused") || error_lower.contains("dns") || error_lower.contains("timeout")
        || error_lower.contains("connect error") || error_lower.contains("network") {
        return LlmErrorKind::Network;
    }

    // 默认归为 Transient（保守策略，允许重试）
    LlmErrorKind::Transient
}

/// 执行带降级链的 chat 请求
///
/// 按降级链顺序尝试每个供应商：
/// - Transient 错误：重试 max_retries_per_provider 次
/// - Permanent 错误：跳到下一供应商
/// - Network 错误：立即跳到下一供应商
pub async fn chat_with_fallback(
    messages: &[ChatMessage],
    clients: &[Box<dyn LlmClient>],
    configs: &[LlmConfig],
    max_retries: u32,
) -> Result<LlmResponse, String> {
    if clients.is_empty() || configs.is_empty() {
        return Err("降级链为空，无可用 LLM 客户端".to_string());
    }

    let mut last_error = String::new();
    let min_len = clients.len().min(configs.len());

    for (idx, _) in (0..min_len).enumerate() {
        let client = &clients[idx];
        let config = &configs[idx];

        let error_kind = match client.chat(messages, config).await {
            Ok(response) => {
                tracing::info!(
                    "LLM 请求成功: 供应商={} (索引={}), model={}",
                    config.provider.as_str(),
                    idx,
                    config.model
                );
                return Ok(response);
            }
            Err(e) => {
                let kind = classify_error(&e);
                tracing::warn!(
                    "LLM 请求失败: 供应商={} (索引={}), 错误类型={:?}, 错误={}",
                    config.provider.as_str(),
                    idx,
                    kind,
                    e
                );
                last_error = e;
                kind
            }
        };

        match error_kind {
            LlmErrorKind::Transient => {
                // 重试（指数退避）
                for retry in 1..=max_retries {
                    let delay = Duration::from_secs(1u64 << (retry - 1).min(4)); // 1s, 2s, 4s, 8s, 16s
                    tracing::info!(
                        "重试 {}/{}: 供应商={}, 等待 {:?}",
                        retry, max_retries,
                        config.provider.as_str(),
                        delay
                    );
                    tokio::time::sleep(delay).await;

                    match client.chat(messages, config).await {
                        Ok(response) => {
                            tracing::info!(
                                "重试成功: 供应商={} (第{}次重试)",
                                config.provider.as_str(),
                                retry
                            );
                            return Ok(response);
                        }
                        Err(e) => {
                            last_error = e;
                            tracing::warn!(
                                "重试失败: 供应商={} (第{}次重试), 错误={}",
                                config.provider.as_str(),
                                retry,
                                last_error
                            );
                        }
                    }
                }
                // 重试耗尽，继续下一供应商
            }
            LlmErrorKind::Permanent | LlmErrorKind::Network => {
                // 跳到下一供应商，不重试
                continue;
            }
        }
    }

    Err(format!("所有供应商均失败，最后一个错误: {}", last_error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_error_permanent() {
        assert_eq!(classify_error("API 错误 (401): Unauthorized"), LlmErrorKind::Permanent);
        assert_eq!(classify_error("API 错误 (403): Forbidden"), LlmErrorKind::Permanent);
        assert_eq!(classify_error("API 错误 (400): Bad Request"), LlmErrorKind::Permanent);
    }

    #[test]
    fn test_classify_error_transient() {
        assert_eq!(classify_error("API 错误 (429): Rate limit exceeded"), LlmErrorKind::Transient);
        assert_eq!(classify_error("API 错误 (500): Internal Server Error"), LlmErrorKind::Transient);
        assert_eq!(classify_error("API 错误 (502): Bad Gateway"), LlmErrorKind::Transient);
        assert_eq!(classify_error("API 错误 (503): Service Unavailable"), LlmErrorKind::Transient);
    }

    #[test]
    fn test_classify_error_network() {
        assert_eq!(classify_error("connection refused"), LlmErrorKind::Network);
        assert_eq!(classify_error("DNS resolution failed"), LlmErrorKind::Network);
        assert_eq!(classify_error("request timeout"), LlmErrorKind::Network);
    }

    #[test]
    fn test_classify_error_default() {
        // 未知错误默认为 Transient
        assert_eq!(classify_error("some unknown error"), LlmErrorKind::Transient);
    }

    #[test]
    fn test_fallback_config_default() {
        let config = FallbackConfig::default();
        assert!(config.chain.is_empty());
        assert_eq!(config.max_retries_per_provider, 2);
        assert_eq!(config.timeout_secs, 60);
    }
}