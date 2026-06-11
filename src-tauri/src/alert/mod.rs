pub mod rules;

use crate::db::DbPool;
use crate::market::StockQuote;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 预警规则类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertRuleType {
    PriceAbove,
    PriceBelow,
    ChangePercentAbove,
    ChangePercentBelow,
    VolumeRatioAbove,
    NewHigh,
    NewLow,
    LimitUp,
    LimitDown,
    Anomaly,
    TempSuspend,
}

/// 预警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub secid: String,
    pub stock_name: String,
    pub rule_type: AlertRuleType,
    pub threshold: f64,
    pub enabled: bool,
    pub triggered: bool,
    pub created_at: i64,
}

/// 预警管理器
pub struct AlertManager {
    db: Arc<DbPool>,
}

impl AlertManager {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    /// 检查预警并通知
    pub fn check_and_notify(&self, quotes: &[StockQuote]) -> Vec<AlertTriggered> {
        let mut triggered = Vec::new();

        // 简化实现：检查涨跌停预警
        for quote in quotes {
            // 涨停检测
            if quote.change_percent >= 9.9 {
                triggered.push(AlertTriggered {
                    secid: quote.secid.clone(),
                    stock_name: quote.name.clone(),
                    rule_type: AlertRuleType::LimitUp,
                    value: quote.change_percent,
                    message: format!("{} 涨停 {:.2}%", quote.name, quote.change_percent),
                });
            }

            // 跌停检测
            if quote.change_percent <= -9.9 {
                triggered.push(AlertTriggered {
                    secid: quote.secid.clone(),
                    stock_name: quote.name.clone(),
                    rule_type: AlertRuleType::LimitDown,
                    value: quote.change_percent,
                    message: format!("{} 跌停 {:.2}%", quote.name, quote.change_percent),
                });
            }
        }

        triggered
    }
}

/// 预警触发事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTriggered {
    pub secid: String,
    pub stock_name: String,
    pub rule_type: AlertRuleType,
    pub value: f64,
    pub message: String,
}
