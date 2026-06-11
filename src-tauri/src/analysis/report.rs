//! 分析报告生成器
//! 独立于 engine.rs，通过 JSON 中间格式生成报告

use serde::{Deserialize, Serialize};

/// 报告中间结构（从 AnalysisResult JSON 生成）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportData {
    pub stock_name: String,
    pub overall_rating: String,
    pub overall_score: f64,
    pub bull_argument: String,
    pub bear_argument: String,
    pub verdict: String,
    pub quality_grade: String,
    pub quality_score: f64,
    pub dimensions: Vec<DimensionSummary>,
}

/// 维度摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionSummary {
    pub name: String,
    pub rating: String,
    pub summary: String,
    pub key_points: Vec<String>,
    pub risks: Vec<String>,
    pub opportunities: Vec<String>,
    pub confidence: f64,
}

/// 从 JSON 字符串生成可读报告
pub fn generate_report_from_json(json: &str) -> String {
    let data: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return "报告生成失败：数据格式错误".to_string(),
    };

    let mut report = String::new();

    let stock_name = data.get("stock_name").and_then(|v| v.as_str()).unwrap_or("未知");
    let overall_score = data.get("overall_score").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let overall_rating = data.get("overall_rating").and_then(|v| v.as_str()).unwrap_or("Hold");
    let quality_grade = data.get("quality_grade").and_then(|v| v.as_str()).unwrap_or("C");
    let bull = data.get("bull_argument").and_then(|v| v.as_str()).unwrap_or("");
    let bear = data.get("bear_argument").and_then(|v| v.as_str()).unwrap_or("");
    let verdict = data.get("verdict").and_then(|v| v.as_str()).unwrap_or("");

    report.push_str(&format!("# {} 分析报告\n\n", stock_name));
    report.push_str(&format!("**综合评级：{}** (评分: {:.1}/100)\n\n", overall_rating, overall_score));
    report.push_str(&format!("**质量门控：{}**\n\n", quality_grade));

    report.push_str("## 多空观点\n\n");
    report.push_str(&format!("### 🐂 多方观点\n{}\n\n", bull));
    report.push_str(&format!("### 🐻 空方观点\n{}\n\n", bear));
    report.push_str(&format!("### ⚖️ 裁决\n{}\n\n", verdict));

    if let Some(dims) = data.get("dimensions").and_then(|v| v.as_object()) {
        report.push_str("## 维度详情\n\n");
        for (key, val) in dims {
            let rating = val.get("rating").and_then(|v| v.as_str()).unwrap_or("C");
            let summary = val.get("summary").and_then(|v| v.as_str()).unwrap_or("");
            let confidence = val.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0);

            report.push_str(&format!("### {} [{}]\n\n", key, rating));
            report.push_str(&format!("{}\n\n", summary));

            if let Some(points) = val.get("key_points").and_then(|v| v.as_array()) {
                report.push_str("**关键要点：**\n");
                for p in points.iter().filter_map(|v| v.as_str()) {
                    report.push_str(&format!("- {}\n", p));
                }
                report.push('\n');
            }
            if let Some(risks) = val.get("risks").and_then(|v| v.as_array()) {
                report.push_str("**风险：**\n");
                for r in risks.iter().filter_map(|v| v.as_str()) {
                    report.push_str(&format!("- ⚠️ {}\n", r));
                }
                report.push('\n');
            }
            if let Some(opps) = val.get("opportunities").and_then(|v| v.as_array()) {
                report.push_str("**机会：**\n");
                for o in opps.iter().filter_map(|v| v.as_str()) {
                    report.push_str(&format!("- ✅ {}\n", o));
                }
                report.push('\n');
            }
            report.push_str(&format!("置信度: {:.0}%\n\n", confidence * 100.0));
        }
    }

    report
}
