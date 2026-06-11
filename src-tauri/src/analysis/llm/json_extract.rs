//! JSON 提取工具 — 从 LLM 自由文本响应中提取结构化 JSON
//!
//! 三级降级策略：
//! 1. ```json 代码块提取
//! 2. 大括号匹配（最外层 {...}）
//! 3. 逐行扫描

use super::super::schemas::{DimensionOutputSchema, StructuredOutputMode};

/// 从文本中提取 JSON 字符串
///
/// 尝试三种策略，返回第一个成功的结果
pub fn extract_json_from_text(text: &str) -> Option<String> {
    // 策略 1: ```json 代码块
    if let Some(json) = extract_from_code_fence(text) {
        return Some(json);
    }
    // 策略 2: 大括号匹配
    if let Some(json) = extract_by_brace_matching(text) {
        return Some(json);
    }
    None
}

/// 从 ```json 代码块中提取 JSON
fn extract_from_code_fence(text: &str) -> Option<String> {
    // 匹配 ```json ... ``` 格式
    let start_marker = "```json";
    let end_marker = "```";

    let start_idx = text.find(start_marker)?;
    let json_start = start_idx + start_marker.len();

    // 找到结束标记（从 json_start 之后开始搜索）
    let end_idx = text[json_start..].find(end_marker)?;
    let json_end = json_start + end_idx;

    let json_str = text[json_start..json_end].trim();
    if json_str.is_empty() {
        return None;
    }

    // 验证是否为合法 JSON
    if serde_json::from_str::<serde_json::Value>(json_str).is_ok() {
        Some(json_str.to_string())
    } else {
        None
    }
}

/// 通过大括号匹配提取最外层 JSON 对象
fn extract_by_brace_matching(text: &str) -> Option<String> {
    // 找到第一个 '{'
    let start_idx = text.find('{')?;
    let mut depth = 0;
    let mut end_idx = start_idx;

    for (i, ch) in text[start_idx..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end_idx = start_idx + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    if depth != 0 {
        return None;
    }

    let json_str = text[start_idx..end_idx].trim();

    // 验证是否为合法 JSON 对象
    if serde_json::from_str::<serde_json::Value>(json_str).is_ok() {
        Some(json_str.to_string())
    } else {
        None
    }
}

/// 解析维度输出（含降级策略）
///
/// 根据 StructuredOutputMode 决定解析方式：
/// - Native: 直接 serde_json::from_str
/// - TextWithJsonExtract: 先尝试直接解析，失败后提取 JSON 再解析
/// - RawText: 不尝试解析，返回错误
pub fn parse_dimension_output(
    text: &str,
    mode: StructuredOutputMode,
) -> Result<DimensionOutputSchema, String> {
    match mode {
        StructuredOutputMode::Native | StructuredOutputMode::TextWithJsonExtract => {
            // 尝试直接解析
            if let Ok(parsed) = serde_json::from_str::<DimensionOutputSchema>(text) {
                return Ok(parsed);
            }

            // 尝试从文本中提取 JSON 再解析
            if let Some(json_str) = extract_json_from_text(text) {
                if let Ok(parsed) = serde_json::from_str::<DimensionOutputSchema>(&json_str) {
                    return Ok(parsed);
                }
            }

            Err("无法从 LLM 响应中解析结构化维度输出".to_string())
        }
        StructuredOutputMode::RawText => {
            Err("RawText 模式 — 不尝试结构化解析".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_code_fence() {
        let text = "Here is the analysis:\n```json\n{\"rating\": \"A\", \"summary\": \"good\"}\n```\nThat's it.";
        let result = extract_json_from_text(text);
        assert!(result.is_some());
        let json = result.unwrap();
        assert!(json.contains("\"rating\""));
        assert!(json.contains("\"summary\""));
    }

    #[test]
    fn test_extract_from_code_fence_no_block() {
        let text = "No code fence here, just plain text.";
        assert_eq!(extract_from_code_fence(text), None);
    }

    #[test]
    fn test_extract_by_brace_matching() {
        let text = "The result is: {\"rating\": \"B\", \"summary\": \"average\", \"key_points\": []} and more text.";
        let result = extract_by_brace_matching(text);
        assert!(result.is_some());
        let json = result.unwrap();
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }

    #[test]
    fn test_extract_by_brace_matching_nested() {
        let text = "Result: {\"outer\": {\"inner\": 1}, \"count\": 2}";
        let result = extract_by_brace_matching(text);
        assert!(result.is_some());
    }

    #[test]
    fn test_extract_by_brace_matching_unbalanced() {
        let text = "Broken: {\"key\": \"value\" missing close";
        assert_eq!(extract_by_brace_matching(text), None);
    }

    #[test]
    fn test_extract_json_priority() {
        // ```json 优先于大括号匹配
        let text = "```json\n{\"a\": 1}\n```\nAlso {\"b\": 2} here.";
        let result = extract_json_from_text(text);
        assert!(result.is_some());
        assert!(result.unwrap().contains("\"a\""));
    }

    #[test]
    fn test_parse_dimension_output_native() {
        let json = r#"{"dimension": "FinancialHealth", "rating": "A", "summary": "Excellent", "key_points": ["Strong ROE"], "risks": [], "opportunities": ["Growth"], "confidence": 0.85}"#;
        let result = parse_dimension_output(json, StructuredOutputMode::Native);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.dimension, "FinancialHealth");
        assert_eq!(parsed.rating, "A");
    }

    #[test]
    fn test_parse_dimension_output_text_with_extract() {
        let text = "分析结果如下：\n```json\n{\"dimension\": \"Valuation\", \"rating\": \"C\", \"summary\": \"Fair\", \"key_points\": [], \"risks\": [\"High PE\"], \"opportunities\": [], \"confidence\": 0.6}\n```";
        let result = parse_dimension_output(text, StructuredOutputMode::TextWithJsonExtract);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.dimension, "Valuation");
        assert_eq!(parsed.rating, "C");
    }

    #[test]
    fn test_parse_dimension_output_raw_text() {
        let text = "Just some plain analysis text.";
        let result = parse_dimension_output(text, StructuredOutputMode::RawText);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("RawText"));
    }

    #[test]
    fn test_parse_dimension_output_fallback_chain() {
        // 文本中嵌入 JSON（无代码块）
        let text = "分析结果: {\"dimension\": \"IndustryTrend\", \"rating\": \"B\", \"summary\": \"Moderate\", \"key_points\": [\"Policy support\"], \"risks\": [], \"opportunities\": [\"AI boom\"], \"confidence\": 0.7}";
        let result = parse_dimension_output(text, StructuredOutputMode::TextWithJsonExtract);
        assert!(result.is_ok());
    }
}