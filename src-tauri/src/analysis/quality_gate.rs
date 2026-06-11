//! 质量门控
//! A-F 分级，基于硬检查规则和LLM复审
//!
//! 参考: doc/impl/S19-AI分析引擎核心.md §10.2

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::engine::DimensionReport;
#[cfg(test)]
use super::engine::{AnalysisDimension, DimensionRating};
use super::llm::LlmClient;

/// 质量门控评级
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum QualityGrade {
    A,
    B,
    C,
    D,
    F,
}

impl QualityGrade {
    pub fn display(&self) -> &'static str {
        match self {
            Self::A => "A (优秀)",
            Self::B => "B (良好)",
            Self::C => "C (合格)",
            Self::D => "D (较差)",
            Self::F => "F (不合格)",
        }
    }

    /// 是否为可接受的质量级别（A/B/C）
    pub fn is_acceptable(&self) -> bool {
        matches!(self, Self::A | Self::B | Self::C)
    }

    /// 是否需要降低权重（D/F）
    pub fn should_downgrade_weight(&self) -> bool {
        matches!(self, Self::D | Self::F)
    }
}

/// 质量门控检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    pub grade: QualityGrade,
    pub score: f64,
    pub confidence: f64,
    pub data_completeness: f64,
    pub logic_consistency: f64,
    pub issues: Vec<String>,
}

/// 质量门控评估总结
#[derive(Debug, Clone, Serialize)]
pub struct QualitySummary {
    /// 各维度的评级
    pub grades: HashMap<String, QualityGrade>,
    /// LLM复审结果（如果触发）
    pub llm_review: Option<String>,
    /// 整体置信度 0.0-1.0
    pub overall_confidence: f64,
    /// 失败维度数量
    pub failed_count: u32,
}

/// 质量门控检查器
pub struct QualityGate {
    llm: Arc<dyn LlmClient>,
}

impl QualityGate {
    pub fn new(llm: Arc<dyn LlmClient>) -> Self {
        Self { llm }
    }

    /// 硬检查（代码级，无 LLM 调用）
    ///
    /// 五层硬检查规则：
    /// - F级：报告为空或summary为空
    /// - D级：summary过短（<50字符）或包含失败标记
    /// - C级：缺失数据≥3处
    /// - B级：缺失数据>0或无汇总表格
    /// - A级：通过所有检查
    pub fn hard_check(&self, report: &DimensionReport) -> QualityGrade {
        // F级：报告为空或summary为空
        if report.summary.trim().is_empty() {
            return QualityGrade::F;
        }

        // D级：summary过短（<50字符）或包含失败标记
        let failure_markers = [
            "无法获取",
            "I cannot retrieve",
            "unable to fetch",
            "工具调用失败",
            "获取数据失败",
            "网络错误",
            "request failed",
        ];
        for marker in &failure_markers {
            if report.summary.contains(marker) {
                return QualityGrade::D;
            }
        }
        if report.summary.len() < 50 {
            return QualityGrade::D;
        }

        // C级：缺失数据≥3处
        let missing_count = report.summary.matches("[数据缺失").count();
        if missing_count >= 3 {
            return QualityGrade::C;
        }

        // B级：缺失数据>0或无汇总表格
        if missing_count > 0 {
            return QualityGrade::B;
        }
        let has_table = report.summary.contains('|') && report.summary.contains("---");
        if !has_table {
            return QualityGrade::B;
        }

        // A级：通过所有检查
        QualityGrade::A
    }

    /// 综合质量评估
    ///
    /// 对所有维度报告进行硬检查，如果≤3个维度失败则触发LLM复审
    pub async fn evaluate(&self, reports: &[DimensionReport]) -> QualitySummary {
        let mut grades = HashMap::new();
        let mut fail_count = 0u32;
        let mut failed_reports: Vec<&DimensionReport> = Vec::new();

        for report in reports {
            let grade = self.hard_check(report);
            let dimension_name = format!("{:?}", report.dimension);

            if matches!(grade, QualityGrade::D | QualityGrade::F) {
                fail_count += 1;
                failed_reports.push(report);
            }

            grades.insert(dimension_name, grade);
        }

        // 如果 ≤3 个维度失败，用 LLM 复审判断是否可接受
        let llm_review = if fail_count <= 3 && !failed_reports.is_empty() {
            match self.llm_review(&failed_reports).await {
                Ok(review) => Some(review),
                Err(e) => {
                    tracing::warn!("LLM复审失败: {}", e);
                    None
                }
            }
        } else {
            None  // 多数失败，跳过复审
        };

        let overall_confidence = Self::calc_confidence(&grades);

        QualitySummary {
            grades,
            llm_review,
            overall_confidence,
            failed_count: fail_count,
        }
    }

    /// 计算整体置信度
    fn calc_confidence(grades: &HashMap<String, QualityGrade>) -> f64 {
        if grades.is_empty() {
            return 0.0;
        }

        let score_sum: f64 = grades.values().map(|g| match g {
            QualityGrade::A => 1.0,
            QualityGrade::B => 0.8,
            QualityGrade::C => 0.6,
            QualityGrade::D => 0.3,
            QualityGrade::F => 0.0,
        }).sum();

        score_sum / grades.len() as f64
    }

    /// 当≤3个维度失败时，用LLM复审判断是否可接受
    ///
    /// 返回复审结论字符串
    pub async fn llm_review(&self, failed_reports: &[&DimensionReport]) -> Result<String, String> {
        let mut prompt = String::from("你是分析质量审核员。以下维度的分析报告未通过硬检查规则。\n\n");

        for (idx, report) in failed_reports.iter().enumerate() {
            prompt.push_str(&format!(
                "--- 维度 {:?} (第{}个失败) ---\n{}\n\n",
                report.dimension,
                idx + 1,
                report.summary
            ));
        }

        prompt.push_str(
            "请评估：\n\
            1. 这些失败的报告是否仍有部分有价值信息？（是/否）\n\
            2. 失败原因是什么？（数据缺失/工具失败/格式问题/其他）\n\
            3. 是否建议继续进行综合决策？（继续/跳过）\n\n\
            请用2-4句话简洁回答。"
        );

        self.llm.generate(&prompt).await.map_err(|e| format!("LLM复审调用失败: {:?}", e))
    }

    /// 生成质量摘要Prompt片段
    ///
    /// 注入到综合决策Prompt中，提醒LLM注意数据质量
    pub fn generate_quality_prompt(&self, grades: &HashMap<String, QualityGrade>) -> String {
        if grades.is_empty() {
            return String::new();
        }

        let grade_str = grades.iter()
            .map(|(dim, grade)| format!("{}: {}", dim, grade.display()))
            .collect::<Vec<_>>()
            .join(" | ");

        let confidence = Self::calc_confidence(grades);

        let mut prompt = format!(
            "\n\n**数据质量门控结果**：\n{}\n整体数据可信度: {:.0}%\n",
            grade_str,
            confidence * 100.0
        );

        // 检查是否有D/F级别的维度
        let low_quality_dims: Vec<&str> = grades.iter()
            .filter(|(_, g)| g.should_downgrade_weight())
            .map(|(dim, _)| dim.as_str())
            .collect();

        if !low_quality_dims.is_empty() {
            prompt.push_str(&format!(
                "\n**注意**：以下维度数据不可靠，决策时请降低权重：{}\n",
                low_quality_dims.join(", ")
            ));
        }

        prompt
    }
}

/// 旧的评估函数，保持向后兼容
pub fn evaluate_quality(
    dimension_confidences: &[f64],
    data_completeness: f64,
    logic_consistency: f64,
) -> QualityGateResult {
    let mut issues = Vec::new();

    // 平均置信度
    let avg_confidence = if dimension_confidences.is_empty() {
        0.0
    } else {
        dimension_confidences.iter().sum::<f64>() / dimension_confidences.len() as f64
    };

    if avg_confidence < 0.5 {
        issues.push("整体置信度低于50%".to_string());
    }
    if data_completeness < 0.7 {
        issues.push(format!("数据完整度仅 {:.0}%", data_completeness * 100.0));
    }
    if logic_consistency < 0.6 {
        issues.push("逻辑一致性不足".to_string());
    }

    // 综合评分 = 0.4*置信度 + 0.3*数据完整度 + 0.3*逻辑一致性
    let score = (avg_confidence * 0.4 + data_completeness * 0.3 + logic_consistency * 0.3) * 100.0;

    let grade = QualityGrade::from_score(score);

    QualityGateResult {
        grade,
        score,
        confidence: avg_confidence,
        data_completeness,
        logic_consistency,
        issues,
    }
}

impl QualityGrade {
    pub fn from_score(score: f64) -> Self {
        if score >= 90.0 { Self::A }
        else if score >= 75.0 { Self::B }
        else if score >= 60.0 { Self::C }
        else if score >= 40.0 { Self::D }
        else { Self::F }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用的模拟DimensionReport
    fn make_report(summary: &str) -> DimensionReport {
        DimensionReport {
            dimension: AnalysisDimension::TechnicalSignals,
            rating: DimensionRating::C,
            confidence: 0.8,
            summary: summary.to_string(),
            key_points: vec![],
            risks: vec![],
            opportunities: vec![],
        }
    }

    #[test]
    fn test_hard_check_empty_summary() {
        let gate = QualityGate::new(Arc::new(crate::analysis::llm::MockLlmClient));
        let report = make_report("");
        assert_eq!(gate.hard_check(&report), QualityGrade::F);

        let report = make_report("   ");
        assert_eq!(gate.hard_check(&report), QualityGrade::F);
    }

    #[test]
    fn test_hard_check_failure_markers() {
        let gate = QualityGate::new(Arc::new(crate::analysis::llm::MockLlmClient));

        let report = make_report("无法获取数据，请稍后重试");
        assert_eq!(gate.hard_check(&report), QualityGrade::D);

        let report = make_report("I cannot retrieve the data at this time");
        assert_eq!(gate.hard_check(&report), QualityGrade::D);

        let report = make_report("工具调用失败，请检查网络连接");
        assert_eq!(gate.hard_check(&report), QualityGrade::D);
    }

    #[test]
    fn test_hard_check_too_short() {
        let gate = QualityGate::new(Arc::new(crate::analysis::llm::MockLlmClient));
        let report = make_report("技术面分析显示股价上涨。");
        assert_eq!(gate.hard_check(&report), QualityGrade::D);
    }

    #[test]
    fn test_hard_check_missing_data_c_grade() {
        let gate = QualityGate::new(Arc::new(crate::analysis::llm::MockLlmClient));
        let report = make_report(
            "技术分析结果：\n\
            1. [数据缺失: K线数据]\n\
            2. [数据缺失: MACD指标]\n\
            3. [数据缺失: 成交量]\n\
            综合判断为中性。"
        );
        assert_eq!(gate.hard_check(&report), QualityGrade::C);
    }

    #[test]
    fn test_hard_check_missing_data_b_grade() {
        let gate = QualityGate::new(Arc::new(crate::analysis::llm::MockLlmClient));
        let report = make_report(
            "技术分析结果：\n\
            1. K线形态：上升趋势\n\
            2. [数据缺失: MACD指标]\n\
            3. 成交量：放量\n\
            综合判断为看多。"
        );
        assert_eq!(gate.hard_check(&report), QualityGrade::B);
    }

    #[test]
    fn test_hard_check_no_table_b_grade() {
        let gate = QualityGate::new(Arc::new(crate::analysis::llm::MockLlmClient));
        let report = make_report(
            "技术分析结果显示当前趋势向上，成交量放大，MACD金叉形成，建议关注。\
            支撑位在10元附近，压力位在15元附近。"
        );
        assert_eq!(gate.hard_check(&report), QualityGrade::B);
    }

    #[test]
    fn test_hard_check_grade_a() {
        let gate = QualityGate::new(Arc::new(crate::analysis::llm::MockLlmClient));
        let report = make_report(
            "技术分析结果：\n\n\
            | 指标 | 数值 | 判断 |\n\
            | --- | --- | --- |\n\
            | MACD | 金叉 | 看多 |\n\
            | KDJ | 超买区 | 谨慎 |\n\
            | 成交量 | 放量 | 确认趋势 |\n\n\
            综合判断：看多，建议逢低建仓。"
        );
        assert_eq!(gate.hard_check(&report), QualityGrade::A);
    }

    #[test]
    fn test_calc_confidence() {
        let mut grades = HashMap::new();
        grades.insert("Technical".to_string(), QualityGrade::A);
        grades.insert("News".to_string(), QualityGrade::B);
        grades.insert("Fundamentals".to_string(), QualityGrade::C);

        let confidence = QualityGate::calc_confidence(&grades);
        // (1.0 + 0.8 + 0.6) / 3 = 0.8
        assert!((confidence - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_generate_quality_prompt() {
        let gate = QualityGate::new(Arc::new(crate::analysis::llm::MockLlmClient));
        let mut grades = HashMap::new();
        grades.insert("Technical".to_string(), QualityGrade::A);
        grades.insert("News".to_string(), QualityGrade::D);
        grades.insert("Fundamentals".to_string(), QualityGrade::B);

        let prompt = gate.generate_quality_prompt(&grades);
        assert!(prompt.contains("数据质量门控结果"));
        assert!(prompt.contains("Technical: A (优秀)"));
        assert!(prompt.contains("News: D (较差)"));
        assert!(prompt.contains("降低权重"));
    }

    #[test]
    fn test_grade_is_acceptable() {
        assert!(QualityGrade::A.is_acceptable());
        assert!(QualityGrade::B.is_acceptable());
        assert!(QualityGrade::C.is_acceptable());
        assert!(!QualityGrade::D.is_acceptable());
        assert!(!QualityGrade::F.is_acceptable());
    }

    #[test]
    fn test_grade_should_downgrade_weight() {
        assert!(!QualityGrade::A.should_downgrade_weight());
        assert!(!QualityGrade::B.should_downgrade_weight());
        assert!(!QualityGrade::C.should_downgrade_weight());
        assert!(QualityGrade::D.should_downgrade_weight());
        assert!(QualityGrade::F.should_downgrade_weight());
    }
}
