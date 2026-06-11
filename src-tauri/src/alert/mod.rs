pub mod rules;

use crate::db::DbPool;
use crate::market::StockQuote;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 预警规则类型（12 种变体，v2.0 完整版）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertRuleType {
    /// 价格上穿阈值
    PriceAbove,
    /// 价格下穿阈值
    PriceBelow,
    /// 涨幅超过阈值(%)
    ChangePercentAbove,
    /// 跌幅超过阈值(%)
    ChangePercentBelow,
    /// 量比超过阈值
    VolumeRatioAbove,
    /// 换手率超过阈值(%)
    TurnoverRateAbove,
    /// 涨停
    LimitUp,
    /// 跌停
    LimitDown,
    /// 异动拉升
    AnomalyRise,
    /// 异动下跌
    AnomalyFall,
    /// 临时停牌
    TempSuspend,
    /// 接近涨停（涨幅≥80%限制幅度）
    NearLimitUp,
}

impl AlertRuleType {
    /// 从字符串反序列化（兼容 SQLite 存储）
    pub fn from_str_ex(s: &str) -> Self {
        match s {
            "PriceAbove" => Self::PriceAbove,
            "PriceBelow" => Self::PriceBelow,
            "ChangePercentAbove" => Self::ChangePercentAbove,
            "ChangePercentBelow" => Self::ChangePercentBelow,
            "VolumeRatioAbove" => Self::VolumeRatioAbove,
            "TurnoverRateAbove" => Self::TurnoverRateAbove,
            "LimitUp" => Self::LimitUp,
            "LimitDown" => Self::LimitDown,
            "AnomalyRise" => Self::AnomalyRise,
            "AnomalyFall" => Self::AnomalyFall,
            "TempSuspend" => Self::TempSuspend,
            "NearLimitUp" => Self::NearLimitUp,
            // 兼容旧版
            "NewHigh" => Self::PriceAbove,
            "NewLow" => Self::PriceBelow,
            "Anomaly" => Self::AnomalyRise,
            _ => Self::PriceAbove,
        }
    }

    /// 转为字符串（SQLite 存储）
    pub fn to_str_ex(&self) -> &'static str {
        match self {
            Self::PriceAbove => "PriceAbove",
            Self::PriceBelow => "PriceBelow",
            Self::ChangePercentAbove => "ChangePercentAbove",
            Self::ChangePercentBelow => "ChangePercentBelow",
            Self::VolumeRatioAbove => "VolumeRatioAbove",
            Self::TurnoverRateAbove => "TurnoverRateAbove",
            Self::LimitUp => "LimitUp",
            Self::LimitDown => "LimitDown",
            Self::AnomalyRise => "AnomalyRise",
            Self::AnomalyFall => "AnomalyFall",
            Self::TempSuspend => "TempSuspend",
            Self::NearLimitUp => "NearLimitUp",
        }
    }

    /// 显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::PriceAbove => "价格上穿",
            Self::PriceBelow => "价格下穿",
            Self::ChangePercentAbove => "涨幅超限",
            Self::ChangePercentBelow => "跌幅超限",
            Self::VolumeRatioAbove => "量比超限",
            Self::TurnoverRateAbove => "换手率超限",
            Self::LimitUp => "涨停",
            Self::LimitDown => "跌停",
            Self::AnomalyRise => "异动拉升",
            Self::AnomalyFall => "异动下跌",
            Self::TempSuspend => "临时停牌",
            Self::NearLimitUp => "接近涨停",
        }
    }
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

/// 预警触发事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTriggered {
    pub secid: String,
    pub stock_name: String,
    pub rule_type: AlertRuleType,
    pub value: f64,
    pub message: String,
}

/// 预警管理器
pub struct AlertManager {
    db: Arc<DbPool>,
}

impl AlertManager {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    /// 检查预警并通知（12 变体全量实现）
    pub fn check_and_notify(&self, quotes: &[StockQuote], rules: &[AlertRule]) -> Vec<AlertTriggered> {
        let mut triggered_list = Vec::new();

        for rule in rules {
            // 跳过禁用或已触发的规则
            if !rule.enabled || rule.triggered {
                continue;
            }

            // 找到对应行情
            let quote = match quotes.iter().find(|q| q.secid == rule.secid) {
                Some(q) => q,
                None => continue,
            };

            // 跳过停牌股
            if quote.is_suspended {
                continue;
            }

            if let Some(alert) = self.evaluate_rule(rule, quote) {
                triggered_list.push(alert);
            }
        }

        triggered_list
    }

    /// 评估单条规则
    fn evaluate_rule(&self, rule: &AlertRule, quote: &StockQuote) -> Option<AlertTriggered> {
        let matched = match rule.rule_type {
            AlertRuleType::PriceAbove => quote.price > rule.threshold,
            AlertRuleType::PriceBelow => quote.price < rule.threshold,
            AlertRuleType::ChangePercentAbove => quote.change_percent > rule.threshold,
            AlertRuleType::ChangePercentBelow => quote.change_percent < -rule.threshold,
            AlertRuleType::VolumeRatioAbove => quote.volume_ratio > rule.threshold,
            AlertRuleType::TurnoverRateAbove => quote.turnover_rate > rule.threshold,
            AlertRuleType::LimitUp => quote.is_limit_up,
            AlertRuleType::LimitDown => quote.is_limit_down,
            AlertRuleType::AnomalyRise => quote.change_speed > rule.threshold && quote.change_percent > 0.0,
            AlertRuleType::AnomalyFall => quote.change_speed < -rule.threshold && quote.change_percent < 0.0,
            AlertRuleType::TempSuspend => quote.is_temp_suspended,
            AlertRuleType::NearLimitUp => quote.is_near_limit_up,
        };

        if matched {
            let value = match rule.rule_type {
                AlertRuleType::LimitUp | AlertRuleType::LimitDown => quote.change_percent,
                AlertRuleType::TempSuspend => 1.0,
                AlertRuleType::NearLimitUp => quote.change_percent,
                _ => match rule.rule_type {
                    AlertRuleType::PriceAbove | AlertRuleType::PriceBelow => quote.price,
                    AlertRuleType::ChangePercentAbove | AlertRuleType::ChangePercentBelow => quote.change_percent,
                    AlertRuleType::VolumeRatioAbove => quote.volume_ratio,
                    AlertRuleType::TurnoverRateAbove => quote.turnover_rate,
                    AlertRuleType::AnomalyRise | AlertRuleType::AnomalyFall => quote.change_speed,
                    _ => 0.0,
                },
            };

            let message = format!(
                "{} {}",
                quote.name,
                match rule.rule_type {
                    AlertRuleType::PriceAbove => format!("价格上穿 {:.2}", quote.price),
                    AlertRuleType::PriceBelow => format!("价格下穿 {:.2}", quote.price),
                    AlertRuleType::ChangePercentAbove => format!("涨幅 {:.2}%", quote.change_percent),
                    AlertRuleType::ChangePercentBelow => format!("跌幅 {:.2}%", quote.change_percent),
                    AlertRuleType::VolumeRatioAbove => format!("量比 {:.2}", quote.volume_ratio),
                    AlertRuleType::TurnoverRateAbove => format!("换手率 {:.2}%", quote.turnover_rate),
                    AlertRuleType::LimitUp => "涨停".to_string(),
                    AlertRuleType::LimitDown => "跌停".to_string(),
                    AlertRuleType::AnomalyRise => format!("异动拉升 涨速{:.2}", quote.change_speed),
                    AlertRuleType::AnomalyFall => format!("异动下跌 涨速{:.2}", quote.change_speed),
                    AlertRuleType::TempSuspend => "临时停牌".to_string(),
                    AlertRuleType::NearLimitUp => format!("接近涨停 {:.2}%", quote.change_percent),
                }
            );

            Some(AlertTriggered {
                secid: quote.secid.clone(),
                stock_name: quote.name.clone(),
                rule_type: rule.rule_type.clone(),
                value,
                message,
            })
        } else {
            None
        }
    }

    /// 重置所有规则触发状态（每日开盘前调用）
    pub fn reset_daily(&self) -> Result<(), String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;
        conn.execute("UPDATE alert_rules SET triggered = 0", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

// ==================== SQLite 持久化 ====================

impl DbPool {
    /// 获取所有预警规则
    pub fn get_alert_rules(&self) -> Result<Vec<AlertRule>, String> {
        let conn = self.conn().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, secid, stock_name, rule_type, threshold, enabled, triggered, created_at
             FROM alert_rules ORDER BY created_at DESC"
        ).map_err(|e| e.to_string())?;
        let rules = stmt
            .query_map([], |row| {
                let rule_type_str: String = row.get(3)?;
                let threshold_str: String = row.get(4)?;
                Ok(AlertRule {
                    id: row.get(0)?,
                    secid: row.get(1)?,
                    stock_name: row.get(2)?,
                    rule_type: AlertRuleType::from_str_ex(&rule_type_str),
                    threshold: threshold_str.parse().unwrap_or(0.0),
                    enabled: row.get::<_, i32>(5)? != 0,
                    triggered: row.get::<_, i32>(6)? != 0,
                    created_at: row.get(7)?,
                })
            }).map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rules)
    }

    /// 添加预警规则
    pub fn add_alert_rule(
        &self,
        id: &str,
        secid: &str,
        stock_name: &str,
        rule_type: &AlertRuleType,
        threshold: f64,
    ) -> Result<(), String> {
        let conn = self.conn().map_err(|e| e.to_string())?;
        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT INTO alert_rules (id, secid, stock_name, rule_type, threshold, enabled, triggered, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, 0, ?6)",
            rusqlite::params![id, secid, stock_name, rule_type.to_str_ex(), threshold.to_string(), now],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 删除预警规则
    pub fn remove_alert_rule(&self, id: &str) -> Result<(), String> {
        let conn = self.conn().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM alert_rules WHERE id = ?1", rusqlite::params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 切换预警规则启用/禁用
    pub fn toggle_alert_rule(&self, id: &str, enabled: bool) -> Result<(), String> {
        let conn = self.conn().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE alert_rules SET enabled = ?1 WHERE id = ?2",
            rusqlite::params![enabled as i32, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 标记规则已触发
    pub fn mark_alert_triggered(&self, id: &str) -> Result<(), String> {
        let conn = self.conn().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE alert_rules SET triggered = 1 WHERE id = ?1",
            rusqlite::params![id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limit::board_type::{BoardType, StockStatus};

    fn make_quote(secid: &str, price: f64, change_pct: f64, volume_ratio: f64) -> StockQuote {
        StockQuote {
            secid: secid.to_string(),
            code: "600519".to_string(),
            name: "贵州茅台".to_string(),
            price,
            change: 0.0,
            change_percent: change_pct,
            volume: 1000,
            amount: 100000.0,
            high: price,
            low: price,
            open: price,
            pre_close: price,
            total_market_cap: 0.0,
            main_net_inflow: 0.0,
            market: 1,
            turnover_rate: 1.5,
            total_turnover_rate: None,
            pe_ttm: 30.0,
            pe_dynamic: None,
            pe_static: 28.0,
            pb: 10.0,
            volume_ratio,
            volume_ratio_note: None,
            change_speed: 0.5,
            ytd_change: 0.0,
            board_type: BoardType::MainBoardSH,
            stock_status: StockStatus::Normal,
            is_limit_up: change_pct >= 9.9,
            is_limit_down: change_pct <= -9.9,
            is_near_limit_up: change_pct >= 8.0,
            limit_up_price: None,
            limit_down_price: None,
            is_suspended: false,
            is_temp_suspended: false,
            temp_suspend_reason: None,
            temp_suspend_resume_time: None,
            seal_strength: None,
            seal_break_count: 0,
            is_margin_target: false,
            margin_balance: None,
            short_volume: None,
        }
    }

    #[test]
    fn test_price_above_alert() {
        let rule = AlertRule {
            id: "1".to_string(),
            secid: "1.600519".to_string(),
            stock_name: "贵州茅台".to_string(),
            rule_type: AlertRuleType::PriceAbove,
            threshold: 1800.0,
            enabled: true,
            triggered: false,
            created_at: 0,
        };
        let quote = make_quote("1.600519", 1850.0, 2.0, 1.0);

        let db = crate::db::DbPool::open_in_memory_for_test();
        let manager = AlertManager::new(Arc::new(db));
        let result = manager.check_and_notify(&[quote], &[rule]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].rule_type, AlertRuleType::PriceAbove);
    }

    #[test]
    fn test_limit_up_alert() {
        let rule = AlertRule {
            id: "2".to_string(),
            secid: "1.600519".to_string(),
            stock_name: "贵州茅台".to_string(),
            rule_type: AlertRuleType::LimitUp,
            threshold: 0.0,
            enabled: true,
            triggered: false,
            created_at: 0,
        };
        let quote = make_quote("1.600519", 1850.0, 10.0, 1.0);

        let db = crate::db::DbPool::open_in_memory_for_test();
        let manager = AlertManager::new(Arc::new(db));
        let result = manager.check_and_notify(&[quote], &[rule]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_triggered_rule_skipped() {
        let rule = AlertRule {
            id: "3".to_string(),
            secid: "1.600519".to_string(),
            stock_name: "贵州茅台".to_string(),
            rule_type: AlertRuleType::LimitUp,
            threshold: 0.0,
            enabled: true,
            triggered: true, // 已触发
            created_at: 0,
        };
        let quote = make_quote("1.600519", 1850.0, 10.0, 1.0);

        let db = crate::db::DbPool::open_in_memory_for_test();
        let manager = AlertManager::new(Arc::new(db));
        let result = manager.check_and_notify(&[quote], &[rule]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_alert_rule_type_roundtrip() {
        let types = vec![
            AlertRuleType::PriceAbove,
            AlertRuleType::PriceBelow,
            AlertRuleType::ChangePercentAbove,
            AlertRuleType::ChangePercentBelow,
            AlertRuleType::VolumeRatioAbove,
            AlertRuleType::TurnoverRateAbove,
            AlertRuleType::LimitUp,
            AlertRuleType::LimitDown,
            AlertRuleType::AnomalyRise,
            AlertRuleType::AnomalyFall,
            AlertRuleType::TempSuspend,
            AlertRuleType::NearLimitUp,
        ];
        for t in types {
            let s = t.to_str_ex();
            let t2 = AlertRuleType::from_str_ex(s);
            assert_eq!(t, t2);
        }
    }
}
