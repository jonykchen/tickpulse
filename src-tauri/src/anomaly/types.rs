use serde::{Deserialize, Serialize};

/// 异动类型定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnomalyType {
    Surge,
    Dive,
    VolumeSpike,
    SealBoard,
    BreakBoard,
    TempSuspend,
}

/// 异动事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyEvent {
    pub id: String,
    pub secid: String,
    pub stock_name: String,
    pub anomaly_type: AnomalyType,
    pub value: f64,
    pub threshold: f64,
    pub timestamp: i64,
    pub detail: Option<String>,
}
