//! 分析进度追踪

use serde::{Deserialize, Serialize};

use super::engine::AnalysisDimension;

/// 分析进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisProgress {
    pub secid: String,
    pub stock_name: String,
    pub total_dimensions: u32,
    pub completed_dimensions: u32,
    pub current_dimension: Option<AnalysisDimension>,
    pub current_step: String,
    pub percent: f64,
    pub started_at: i64,
    pub estimated_remaining_secs: Option<u64>,
}

impl AnalysisProgress {
    pub fn new(secid: &str, stock_name: &str) -> Self {
        Self {
            secid: secid.to_string(),
            stock_name: stock_name.to_string(),
            total_dimensions: 7,
            completed_dimensions: 0,
            current_dimension: None,
            current_step: "初始化".to_string(),
            percent: 0.0,
            started_at: chrono::Utc::now().timestamp(),
            estimated_remaining_secs: None,
        }
    }

    pub fn update_dimension(&mut self, dimension: AnalysisDimension, step: &str) {
        self.current_dimension = Some(dimension);
        self.current_step = step.to_string();
        self.percent = (self.completed_dimensions as f64 / self.total_dimensions as f64) * 100.0;
    }

    pub fn complete_dimension(&mut self) {
        self.completed_dimensions += 1;
        self.percent = (self.completed_dimensions as f64 / self.total_dimensions as f64) * 100.0;
    }
}