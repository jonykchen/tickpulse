pub mod data_vendor_router;
pub mod exchange;
pub mod field_registry;
pub mod northbound_cache;
pub mod scheduler;
pub mod sources;
pub mod ticker_normalizer;
pub mod trade_calendar;
pub mod types;

// 重导出常用类型
pub use data_vendor_router::{DataCapability, DataVendorRouter};

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

use crate::limit::board_type::BoardType;
use crate::limit::board_type::StockStatus;

/// 量比衰减提示（v2.0）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VolumeRatioNote {
    /// 开盘30分钟内，量比偏高，参考价值有限
    Early,
}

// ==================== 通用数据类型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    // ── 基础字段 ──
    pub secid: String,
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: i64,
    pub amount: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub pre_close: f64,
    pub total_market_cap: f64,
    pub main_net_inflow: f64,
    pub market: i64,

    // ── 换手率（v2.0: 区分流通/总） ──
    pub turnover_rate: f64,              // 流通换手率（东方财富 f8，常用）
    pub total_turnover_rate: Option<f64>, // 总换手率（v2.0 新增）

    // ── 市盈率（v2.0: 三种 PE） ──
    pub pe_ttm: f64,                     // TTM 市盈率（最常用，默认展示）
    pub pe_dynamic: Option<f64>,         // 动态市盈率（东方财富 f9，实测为 TTM）
    pub pe_static: f64,                  // 静态市盈率（上年度 EPS）
    pub pb: f64,                         // 市净率

    // ── 量比（v2.0: 衰减标记） ──
    pub volume_ratio: f64,
    pub volume_ratio_note: Option<VolumeRatioNote>, // 衰减提示

    // ── 其他指标 ──
    pub change_speed: f64,               // 涨速
    pub ytd_change: f64,                 // 年初至今涨幅

    // ── 涨跌停/停牌 ──
    pub board_type: BoardType,           // 板块类型
    pub stock_status: StockStatus,       // 股票状态
    pub is_limit_up: bool,               // 是否涨停
    pub is_limit_down: bool,             // 是否跌停
    pub is_near_limit_up: bool,          // 是否接近涨停（涨幅≥8%）
    pub limit_up_price: Option<f64>,     // 涨停价
    pub limit_down_price: Option<f64>,   // 跌停价
    pub is_suspended: bool,              // 是否停牌

    // ── 新股临停（v2.0） ──
    pub is_temp_suspended: bool,         // 是否临时停牌
    pub temp_suspend_reason: Option<String>, // 临停原因
    pub temp_suspend_resume_time: Option<i64>, // 预计恢复时间

    // ── 封板信息 ──
    pub seal_strength: Option<f64>,      // 封板强度 0.0-1.0
    pub seal_break_count: u32,           // 炸板次数

    // ── 融资融券（v2.0） ──
    pub is_margin_target: bool,          // 是否为两融标的
    pub margin_balance: Option<f64>,     // 融资余额（亿元）
    pub short_volume: Option<f64>,       // 融券余量（万股）
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
        let mut limit_up = 0i32;
        let mut limit_down = 0i32;
        for q in quotes {
            if q.is_suspended {
                continue;
            }
            if q.is_limit_up {
                limit_up += 1;
            } else if q.is_limit_down {
                limit_down += 1;
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
            limit_up_count: limit_up,
            limit_down_count: limit_down,
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

/// 大宗交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTrade {
    pub secid: String,
    pub code: String,
    pub name: String,
    pub trade_date: String,
    pub price: f64,
    pub volume: f64,
    pub amount: f64,
    pub buyer: Option<String>,
    pub seller: Option<String>,
    pub premium_rate: Option<f64>,
}
