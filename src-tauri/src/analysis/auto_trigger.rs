//! 自动触发分析功能
//! 盘后自动分析 + 异动触发分析 + 幂等性检查

use chrono::{Local, Timelike};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::db::DbPool;
use crate::market::trade_calendar::TradeCalendar;
use crate::market::MarketDataSource;

use super::engine::{AnalysisEngine, AnalysisResult};

/// 自动触发管理器
/// 负责：
/// 1. 盘后自动分析（交易日 15:30）
/// 2. 异动触发自动分析（封板/炸板/量比突增）
/// 3. 幂等性检查（避免重复分析浪费 token）
pub struct AutoTriggerManager {
    /// 数据库连接池
    db: Arc<DbPool>,
    /// 交易日历
    calendar: Arc<TradeCalendar>,
    /// 分析引擎
    engine: Arc<AnalysisEngine>,
    /// 上次触发日期（用于防止同一天多次触发）
    last_trigger_date: Mutex<Option<String>>,
}

/// 触发类型
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerType {
    /// 盘后自动分析（15:30）
    AutoPostMarket,
    /// 异动触发
    Anomaly {
        anomaly_type: String,
    },
    /// 手动触发
    Manual,
}

/// 异动类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum AnomalyType {
    /// 涨停封板
    LimitUpSealed,
    /// 跌停封板
    LimitDownSealed,
    /// 炸板（涨停打开）
    LimitUpBroken,
    /// 炸板（跌停打开）
    LimitDownBroken,
    /// 量比突增（>5）
    VolumeRatioSpike,
}

impl AnomalyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LimitUpSealed => "limit_up_sealed",
            Self::LimitDownSealed => "limit_down_sealed",
            Self::LimitUpBroken => "limit_up_broken",
            Self::LimitDownBroken => "limit_down_broken",
            Self::VolumeRatioSpike => "volume_ratio_spike",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "limit_up_sealed" => Some(Self::LimitUpSealed),
            "limit_down_sealed" => Some(Self::LimitDownSealed),
            "limit_up_broken" => Some(Self::LimitUpBroken),
            "limit_down_broken" => Some(Self::LimitDownBroken),
            "volume_ratio_spike" => Some(Self::VolumeRatioSpike),
            _ => None,
        }
    }

    /// 是否值得触发分析（涨停封板和量比突增值得分析）
    pub fn should_trigger_analysis(&self) -> bool {
        matches!(
            self,
            Self::LimitUpSealed | Self::VolumeRatioSpike
        )
    }
}

impl AutoTriggerManager {
    /// 创建自动触发管理器
    pub fn new(
        db: Arc<DbPool>,
        calendar: Arc<TradeCalendar>,
        engine: Arc<AnalysisEngine>,
    ) -> Self {
        Self {
            db,
            calendar,
            engine,
            last_trigger_date: Mutex::new(None),
        }
    }

    /// 检查是否应该触发盘后自动分析（15:30）
    /// 返回：触发成功时返回已分析的股票列表
    pub async fn check_and_trigger_after_hours(&self) -> Result<Vec<String>, String> {
        // 1. 检查是否为交易日
        if !self.calendar.is_trading_day().await {
            tracing::info!("非交易日，跳过盘后自动分析");
            return Ok(Vec::new());
        }

        // 2. 检查当前时间是否在 15:30 之后
        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute();

        // 15:30 之前不触发
        if hour < 15 || (hour == 15 && minute < 30) {
            tracing::debug!("当前时间 {}:{} 未到 15:30，跳过盘后自动分析", hour, minute);
            return Ok(Vec::new());
        }

        // 3. 幂等性检查：今天是否已触发过
        let today_str = now.format("%Y-%m-%d").to_string();
        {
            let last_date = self.last_trigger_date.lock().await;
            if let Some(last) = last_date.as_ref() {
                if last == &today_str {
                    tracing::info!("今日 {} 已触发过盘后自动分析，跳过", today_str);
                    return Ok(Vec::new());
                }
            }
        }

        // 4. 触发自选股分析
        let analyzed_secids = self.trigger_watchlist_analysis().await?;

        // 5. 更新上次触发日期
        {
            let mut last_date = self.last_trigger_date.lock().await;
            *last_date = Some(today_str.clone());
        }

        tracing::info!(
            "盘后自动分析完成，日期: {}, 分析股票数: {}",
            today_str,
            analyzed_secids.len()
        );

        Ok(analyzed_secids)
    }

    /// 异动触发自动分析
    /// 当检测到封板/炸板/量比突增时调用
    pub async fn on_anomaly_detected(
        &self,
        secid: &str,
        anomaly_type: &str,
    ) -> Result<(), String> {
        // 解析异动类型
        let anomaly = AnomalyType::from_str(anomaly_type);
        if anomaly.is_none() {
            tracing::warn!("未知异动类型: {}, 跳过分析", anomaly_type);
            return Ok(());
        }

        let anomaly = anomaly.unwrap();

        // 判断是否值得触发分析
        if !anomaly.should_trigger_analysis() {
            tracing::debug!(
                "异动类型 {} 不触发分析（仅记录）",
                anomaly_type
            );
            return Ok(());
        }

        // 幂等性检查：今天是否已分析过该股票
        if self.has_analyzed_today(secid).await? {
            tracing::info!(
                "股票 {} 今日已有分析记录，异动触发跳过（避免浪费 token）",
                secid
            );
            return Ok(());
        }

        // 获取股票名称（从数据库或行情数据）
        let stock_name = self.get_stock_name(secid).await?;

        // 获取双 LLM 配置
        let dual_config = self.engine.get_dual_llm_config()?;

        // 执行分析
        tracing::info!(
            "异动触发分析: {} ({}), 异动类型: {}",
            secid,
            stock_name,
            anomaly_type
        );

        match self.engine.run(secid, &stock_name, &dual_config).await {
            Ok(result) => {
                tracing::info!(
                    "异动分析完成: {} -> {}",
                    secid,
                    result.overall_rating.display()
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!("异动分析失败: {} - {}", secid, e);
                Err(e)
            }
        }
    }

    /// 批量触发自选股分析
    /// 遍历所有自选股，跳过已分析的，分析未分析的
    pub async fn trigger_watchlist_analysis(&self) -> Result<Vec<String>, String> {
        // 1. 获取所有自选股 secid
        let secids = self.db.get_all_watchlist_secids().map_err(|e| e.to_string())?;

        if secids.is_empty() {
            tracing::info!("自选股列表为空，跳过分析");
            return Ok(Vec::new());
        }

        tracing::info!("开始分析 {} 只自选股", secids.len());

        // 2. 获取双 LLM 配置
        let dual_config = self.engine.get_dual_llm_config()?;

        // 3. 并行分析（但有幂等性检查）
        let mut analyzed = Vec::new();
        let mut skipped = Vec::new();

        for secid in &secids {
            // 幂等性检查
            if self.has_analyzed_today(secid).await? {
                skipped.push(secid.clone());
                continue;
            }

            // 获取股票名称
            let stock_name = self.get_stock_name(secid).await?;

            // 执行分析
            match self.engine.run(secid, &stock_name, &dual_config).await {
                Ok(result) => {
                    tracing::info!(
                        "分析完成: {} -> {}",
                        secid,
                        result.overall_rating.display()
                    );
                    analyzed.push(secid.clone());
                }
                Err(e) => {
                    tracing::error!("分析失败: {} - {}", secid, e);
                    // 失败的也计入尝试过，避免反复重试同一只股票
                    analyzed.push(secid.clone());
                }
            }
        }

        tracing::info!(
            "自选股分析完成: 成功 {} 只, 跳过 {} 只",
            analyzed.len(),
            skipped.len()
        );

        Ok(analyzed)
    }

    /// 幂等性检查：今天是否已分析过该股票
    /// 检查 analysis_results 表当天是否有该股票的分析记录
    async fn has_analyzed_today(&self, secid: &str) -> Result<bool, String> {
        let today = Local::now().date_naive();
        let today_start = today
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp();
        let today_end = today
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp();

        let conn = self.db.conn().map_err(|e| e.to_string())?;

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM analysis_results WHERE secid = ?1 AND created_at >= ?2 AND created_at <= ?3",
                rusqlite::params![secid, today_start, today_end],
                |row| row.get(0),
            )
            .map_err(|e| format!("查询分析记录失败: {}", e))?;

        Ok(count > 0)
    }

    /// 获取股票名称
    /// 优先从自选股表获取，若无则从行情数据获取
    async fn get_stock_name(&self, secid: &str) -> Result<String, String> {
        // 1. 从自选股表获取
        let conn = self.db.conn().map_err(|e| e.to_string())?;
        let name_from_db: Option<String> = conn
            .query_row(
                "SELECT name FROM watchlist_stocks WHERE secid = ?1 LIMIT 1",
                rusqlite::params![secid],
                |row| row.get(0),
            )
            .ok();

        if let Some(name) = name_from_db {
            if !name.is_empty() {
                return Ok(name);
            }
        }

        // 2. 从行情数据获取
        let source = crate::market::sources::eastmoney::EastMoneySource::new();
        match source.fetch_quotes(&[secid.to_string()]).await {
            Ok(quotes) => {
                if let Some(q) = quotes.first() {
                    if !q.name.is_empty() {
                        // 更新数据库中的名称
                        self.update_stock_name(secid, &q.name)?;
                        return Ok(q.name.clone());
                    }
                }
            }
            Err(e) => {
                tracing::warn!("获取股票名称失败: {} - {}", secid, e);
            }
        }

        // 3. 使用 secid 作为默认名称
        Ok(secid.to_string())
    }

    /// 更新自选股表中的股票名称
    fn update_stock_name(&self, secid: &str, name: &str) -> Result<(), String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE watchlist_stocks SET name = ?1 WHERE secid = ?2",
            rusqlite::params![name, secid],
        )
        .map_err(|e| format!("更新股票名称失败: {}", e))?;
        Ok(())
    }

    /// 强制触发分析（忽略幂等性检查）
    /// 用于手动触发或用户明确要求重新分析
    pub async fn force_analyze(
        &self,
        secid: &str,
        stock_name: &str,
    ) -> Result<AnalysisResult, String> {
        let dual_config = self.engine.get_dual_llm_config()?;
        self.engine.run(secid, stock_name, &dual_config).await
    }

    /// 获取今天已分析的股票列表
    pub async fn get_today_analyzed(&self) -> Result<Vec<String>, String> {
        let today = Local::now().date_naive();
        let today_start = today
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp();
        let today_end = today
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp();

        let conn = self.db.conn().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT secid FROM analysis_results WHERE created_at >= ?1 AND created_at <= ?2",
            )
            .map_err(|e| format!("查询分析记录失败: {}", e))?;

        let secids: Vec<String> = stmt
            .query_map(rusqlite::params![today_start, today_end], |row| row.get(0))
            .map_err(|e| format!("查询分析记录失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(secids)
    }

    /// 检查是否应该触发分析（综合判断）
    /// 用于调度器定期调用
    pub async fn should_trigger(&self) -> bool {
        // 1. 是否为交易日
        if !self.calendar.is_trading_day().await {
            return false;
        }

        // 2. 时间检查（15:30-18:00 窗口）
        let now = Local::now();
        let hour = now.hour();
        let minute = now.minute();

        // 只在 15:30 到 18:00 之间触发
        if hour < 15 || hour >= 18 {
            return false;
        }
        if hour == 15 && minute < 30 {
            return false;
        }

        // 3. 幂等性检查
        let today_str = now.format("%Y-%m-%d").to_string();
        {
            let last_date = self.last_trigger_date.lock().await;
            if let Some(last) = last_date.as_ref() {
                if last == &today_str {
                    return false;
                }
            }
        }

        true
    }
}

/// 分析触发配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AutoTriggerConfig {
    /// 是否启用盘后自动分析
    pub enable_post_market: bool,
    /// 盘后分析触发时间（小时，默认 15）
    pub post_market_hour: u32,
    /// 盘后分析触发时间（分钟，默认 30）
    pub post_market_minute: u32,
    /// 是否启用异动触发分析
    pub enable_anomaly_trigger: bool,
    /// 触发异动分析的异动类型列表
    pub anomaly_types: Vec<String>,
    /// 单日最大分析次数（防止 token 消耗过多）
    pub max_daily_analysis: u32,
}

impl Default for AutoTriggerConfig {
    fn default() -> Self {
        Self {
            enable_post_market: true,
            post_market_hour: 15,
            post_market_minute: 30,
            enable_anomaly_trigger: true,
            anomaly_types: vec![
                AnomalyType::LimitUpSealed.as_str().to_string(),
                AnomalyType::VolumeRatioSpike.as_str().to_string(),
            ],
            max_daily_analysis: 50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anomaly_type_should_trigger() {
        assert!(AnomalyType::LimitUpSealed.should_trigger_analysis());
        assert!(AnomalyType::VolumeRatioSpike.should_trigger_analysis());
        assert!(!AnomalyType::LimitDownSealed.should_trigger_analysis());
        assert!(!AnomalyType::LimitUpBroken.should_trigger_analysis());
    }

    #[test]
    fn test_anomaly_type_from_str() {
        assert_eq!(
            AnomalyType::from_str("limit_up_sealed"),
            Some(AnomalyType::LimitUpSealed)
        );
        assert_eq!(
            AnomalyType::from_str("volume_ratio_spike"),
            Some(AnomalyType::VolumeRatioSpike)
        );
        assert_eq!(AnomalyType::from_str("unknown"), None);
    }

    #[test]
    fn test_default_config() {
        let config = AutoTriggerConfig::default();
        assert!(config.enable_post_market);
        assert!(config.enable_anomaly_trigger);
        assert_eq!(config.post_market_hour, 15);
        assert_eq!(config.post_market_minute, 30);
        assert_eq!(config.max_daily_analysis, 50);
    }
}