pub mod exchange;
pub mod scheduler;
pub mod sources;
pub mod trade_calendar;
pub mod types;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::result::Result;

/// 行情数据源统一接口
#[async_trait]
pub trait MarketDataSource: Send + Sync {
    /// 批量获取实时行情
    async fn fetch_quotes(&self, secids: &[String]) -> Result<Vec<StockQuote>, String>;

    /// 获取K线数据
    async fn fetch_kline(
        &self,
        secid: &str,
        period: KlinePeriod,
        limit: u32,
        adjust: AdjustType,
    ) -> Result<Vec<KlineBar>, String>;

    /// 获取分时走势数据
    async fn fetch_timeline(&self, secid: &str) -> Result<TimelineData, String>;

    /// 搜索股票
    async fn search(&self, keyword: &str) -> Result<Vec<SearchResult>, String>;

    /// 获取除权除息信息
    async fn fetch_exrights(&self, secid: &str) -> Result<Vec<ExRightInfo>, String>;

    /// 数据源名称
    fn name(&self) -> &'static str;

    /// 优先级（越小越高）
    fn priority(&self) -> u8;
}

// ==================== 通用数据类型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    pub secid: String,
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: i64,
    pub amount: f64,
    pub turnover_rate: f64,
    pub volume_ratio: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub pre_close: f64,
    pub total_market_cap: f64,
    pub pe_ttm: f64,
    pub pe_static: f64,
    pub pb: f64,
    pub change_speed: f64,
    pub ytd_change: f64,
    pub main_net_inflow: f64,
    pub market: i64,
    pub is_suspended: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum KlinePeriod {
    Min1,
    Min5,
    Min15,
    Min30,
    Min60,
    Daily,
    Weekly,
    Monthly,
}

impl KlinePeriod {
    pub fn to_eastmoney_code(&self) -> &'static str {
        match self {
            Self::Min1 => "1",
            Self::Min5 => "5",
            Self::Min15 => "15",
            Self::Min30 => "30",
            Self::Min60 => "60",
            Self::Daily => "101",
            Self::Weekly => "102",
            Self::Monthly => "103",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AdjustType {
    None,
    Forward,
    Backward,
}

impl AdjustType {
    pub fn to_fqt(&self) -> &'static str {
        match self {
            Self::None => "0",
            Self::Forward => "1",
            Self::Backward => "2",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineBar {
    pub time: i64,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: i64,
    pub amount: f64,
    pub change_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    pub time: String,
    pub price: f64,
    pub avg_price: f64,
    pub volume: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineData {
    pub secid: String,
    pub name: String,
    pub pre_close: f64,
    pub points: Vec<TimelinePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub secid: String,
    pub code: String,
    pub name: String,
    pub market: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExRightInfo {
    pub secid: String,
    pub ex_date: String,
    pub bonus_share: f64,
    pub allot_share: f64,
    pub allot_price: f64,
    pub dividend: f64,
}

/// 涨跌家数摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSummary {
    pub up_count: i32,
    pub down_count: i32,
    pub flat_count: i32,
    pub limit_up_count: i32,
    pub limit_down_count: i32,
}

impl MarketSummary {
    pub fn from_quotes(quotes: &[StockQuote]) -> Self {
        let mut up = 0i32;
        let mut down = 0i32;
        let mut flat = 0i32;
        for q in quotes {
            if q.is_suspended {
                continue;
            }
            if q.change_percent > 0.0 {
                up += 1;
            } else if q.change_percent < 0.0 {
                down += 1;
            } else {
                flat += 1;
            }
        }
        Self {
            up_count: up,
            down_count: down,
            flat_count: flat,
            limit_up_count: 0,
            limit_down_count: 0,
        }
    }
}

/// 调度器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatus {
    pub phase: String,
    pub interval_secs: u64,
    pub is_trading_day: bool,
}
