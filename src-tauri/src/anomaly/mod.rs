pub mod types;

use crate::market::StockQuote;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 异动检测引擎
pub struct AnomalyDetector {
    /// 异动阈值配置
    config: AnomalyConfig,
}

/// 异动检测配置
#[derive(Debug, Clone)]
pub struct AnomalyConfig {
    /// 快速拉升阈值（3分钟涨幅%）
    pub surge_threshold: f64,
    /// 快速下跌阈值（3分钟跌幅%）
    pub dive_threshold: f64,
    /// 量比突增阈值
    pub volume_spike_threshold: f64,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            surge_threshold: 3.0,
            dive_threshold: -3.0,
            volume_spike_threshold: 3.0,
        }
    }
}

/// 异动类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnomalyType {
    /// 快速拉升
    Surge,
    /// 快速下跌
    Dive,
    /// 量比突增
    VolumeSpike,
    /// 封板
    SealBoard,
    /// 炸板
    BreakBoard,
    /// 临时停牌
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

impl AnomalyDetector {
    pub fn new(config: AnomalyConfig) -> Self {
        Self { config }
    }

    /// 检测异动
    pub fn detect(
        &self,
        current: &[StockQuote],
        prev: &DashMap<String, StockQuote>,
    ) -> Vec<AnomalyEvent> {
        let mut anomalies = Vec::new();
        let now = chrono::Utc::now().timestamp();

        for quote in current {
            // 量比突增检测
            if quote.volume_ratio >= self.config.volume_spike_threshold {
                anomalies.push(AnomalyEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    secid: quote.secid.clone(),
                    stock_name: quote.name.clone(),
                    anomaly_type: AnomalyType::VolumeSpike,
                    value: quote.volume_ratio,
                    threshold: self.config.volume_spike_threshold,
                    timestamp: now,
                    detail: Some(format!("量比: {:.2}", quote.volume_ratio)),
                });
            }

            // 与前一次行情对比检测快速拉升/下跌
            if let Some(prev_quote) = prev.get(&quote.secid) {
                let change_diff = quote.change_percent - prev_quote.change_percent;

                // 快速拉升
                if change_diff >= self.config.surge_threshold {
                    anomalies.push(AnomalyEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        secid: quote.secid.clone(),
                        stock_name: quote.name.clone(),
                        anomaly_type: AnomalyType::Surge,
                        value: change_diff,
                        threshold: self.config.surge_threshold,
                        timestamp: now,
                        detail: Some(format!(
                            "涨幅从{:.2}%升至{:.2}%",
                            prev_quote.change_percent, quote.change_percent
                        )),
                    });
                }

                // 快速下跌
                if change_diff <= self.config.dive_threshold {
                    anomalies.push(AnomalyEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        secid: quote.secid.clone(),
                        stock_name: quote.name.clone(),
                        anomaly_type: AnomalyType::Dive,
                        value: change_diff,
                        threshold: self.config.dive_threshold,
                        timestamp: now,
                        detail: Some(format!(
                            "跌幅从{:.2}%降至{:.2}%",
                            prev_quote.change_percent, quote.change_percent
                        )),
                    });
                }
            }
        }

        anomalies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AnomalyConfig::default();
        assert_eq!(config.surge_threshold, 3.0);
        assert_eq!(config.dive_threshold, -3.0);
        assert_eq!(config.volume_spike_threshold, 3.0);
    }

    #[test]
    fn test_volume_spike_detection() {
        let detector = AnomalyDetector::new(AnomalyConfig::default());
        let prev = DashMap::new();

        let quote = StockQuote {
            secid: "0.000001".to_string(),
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            price: 10.0,
            change: 0.1,
            change_percent: 1.0,
            volume: 10000,
            amount: 100000.0,
            high: 10.2,
            low: 9.8,
            open: 9.9,
            pre_close: 9.9,
            total_market_cap: 0.0,
            main_net_inflow: 0.0,
            market: 0,
            turnover_rate: 1.0,
            total_turnover_rate: None,
            pe_ttm: 0.0,
            pe_dynamic: None,
            pe_static: 0.0,
            pb: 0.0,
            volume_ratio: 5.0, // 超过阈值
            volume_ratio_note: None,
            change_speed: 0.0,
            ytd_change: 0.0,
            board_type: crate::limit::board_type::BoardType::MainBoardSZ,
            stock_status: crate::limit::board_type::StockStatus::Normal,
            is_limit_up: false,
            is_limit_down: false,
            is_near_limit_up: false,
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
        };

        let anomalies = detector.detect(&[quote], &prev);
        assert!(anomalies.iter().any(|a| a.anomaly_type == AnomalyType::VolumeSpike));
    }
}
