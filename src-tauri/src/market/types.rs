use crate::market::exchange::Exchange;
use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 交易阶段（14阶段精细化调度）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketPhase {
    Holiday,              // 非交易日
    PreMarket,            // 00:00-09:14
    AuctionCancelable,    // 09:15-09:20（可撤单）
    AuctionUncancelable,  // 09:20-09:25（不可撤单）
    PreOpen,              // 09:25-09:30（静默期）
    MorningVolatile,      // 09:30-10:30（开盘首小时）
    MorningStable,        // 10:31-11:30（平稳期）
    LunchBreak,           // 11:31-12:59（午间休市）
    AfternoonOpen,        // 13:00-13:30（午盘开盘）
    AfternoonStable,      // 13:31-14:56（平稳期）
    ClosingAuction,       // 14:57-15:00（深交所/科创板收盘竞价）
    ContinuousClosing,    // 14:57-15:00（沪市主板连续竞价收盘）
    AfterHours,           // 15:01-15:04
    PostMarketTrading,    // 15:05-15:30（科创板/创业板盘后固定价格交易）
    PostMarketEnd,        // 15:31-23:59
}

impl MarketPhase {
    /// 根据当前时间和交易所判断交易阶段
    pub fn current(is_trading_day: bool, exchange: Exchange) -> Self {
        if !is_trading_day {
            return Self::Holiday;
        }
        let now = chrono::Local::now();
        let (h, m) = (now.hour(), now.minute());
        match (h, m) {
            (0..=8, _) | (9, 0..=14) => Self::PreMarket,
            (9, 15..=19) => Self::AuctionCancelable,
            (9, 20..=24) => Self::AuctionUncancelable,
            (9, 25..=29) => Self::PreOpen,
            (9, 30..=59) | (10, 0..=30) => Self::MorningVolatile,
            (10, 31..=59) | (11, 0..=30) => Self::MorningStable,
            (11, 31..=59) | (12, _) => Self::LunchBreak,
            (13, 0..=30) => Self::AfternoonOpen,
            (13, 31..=59) | (14, 0..=56) => Self::AfternoonStable,
            (14, 57..=59) => {
                match exchange {
                    Exchange::SZSE => Self::ClosingAuction,
                    Exchange::SSE => Self::ContinuousClosing,
                    Exchange::BSE => Self::ClosingAuction,
                    _ => Self::ClosingAuction,
                }
            }
            (15, 0..=4) => Self::AfterHours,
            (15, 5..=30) => {
                match exchange {
                    Exchange::SSE | Exchange::SZSE => Self::PostMarketTrading,
                    _ => Self::PostMarketEnd,
                }
            }
            _ => Self::PostMarketEnd,
        }
    }

    /// 各阶段刷新间隔
    pub fn refresh_interval(&self) -> Duration {
        match self {
            Self::Holiday => Duration::from_secs(3600),
            Self::PreMarket => Duration::from_secs(300),
            Self::AuctionCancelable => Duration::from_secs(5),
            Self::AuctionUncancelable => Duration::from_secs(3),
            Self::PreOpen => Duration::from_secs(30),
            Self::MorningVolatile => Duration::from_secs(6),
            Self::MorningStable => Duration::from_secs(10),
            Self::LunchBreak => Duration::from_secs(60),
            Self::AfternoonOpen => Duration::from_secs(6),
            Self::AfternoonStable => Duration::from_secs(10),
            Self::ClosingAuction => Duration::from_secs(5),
            Self::ContinuousClosing => Duration::from_secs(5),
            Self::AfterHours => Duration::from_secs(300),
            Self::PostMarketTrading => Duration::from_secs(15),
            Self::PostMarketEnd => Duration::from_secs(300),
        }
    }

    /// 阶段显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Holiday => "休市",
            Self::PreMarket => "盘前",
            Self::AuctionCancelable | Self::AuctionUncancelable => "集合竞价",
            Self::PreOpen => "即将开盘",
            Self::MorningVolatile | Self::MorningStable => "交易中",
            Self::LunchBreak => "午间休市",
            Self::AfternoonOpen | Self::AfternoonStable => "交易中",
            Self::ClosingAuction => "收盘竞价",
            Self::ContinuousClosing => "收盘竞价(沪)",
            Self::AfterHours => "已收盘",
            Self::PostMarketTrading => "盘后交易",
            Self::PostMarketEnd => "已收盘",
        }
    }

    /// 是否有实时数据
    pub fn has_realtime_data(&self) -> bool {
        !matches!(
            self,
            Self::Holiday | Self::PreMarket | Self::AfterHours | Self::PostMarketEnd
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_holiday() {
        let phase = MarketPhase::current(false, Exchange::SZSE);
        assert_eq!(phase, MarketPhase::Holiday);
    }

    #[test]
    fn test_refresh_intervals() {
        assert_eq!(MarketPhase::AuctionUncancelable.refresh_interval(), Duration::from_secs(3));
        assert_eq!(MarketPhase::MorningVolatile.refresh_interval(), Duration::from_secs(6));
        assert_eq!(MarketPhase::LunchBreak.refresh_interval(), Duration::from_secs(60));
        assert_eq!(MarketPhase::ClosingAuction.refresh_interval(), Duration::from_secs(5));
    }

    #[test]
    fn test_has_realtime_data() {
        assert!(!MarketPhase::Holiday.has_realtime_data());
        assert!(!MarketPhase::PreMarket.has_realtime_data());
        assert!(MarketPhase::MorningVolatile.has_realtime_data());
        assert!(MarketPhase::ClosingAuction.has_realtime_data());
        assert!(!MarketPhase::AfterHours.has_realtime_data());
    }

    #[test]
    fn test_display_names() {
        assert_eq!(MarketPhase::Holiday.display_name(), "休市");
        assert_eq!(MarketPhase::ClosingAuction.display_name(), "收盘竞价");
        assert_eq!(MarketPhase::ContinuousClosing.display_name(), "收盘竞价(沪)");
        assert_eq!(MarketPhase::PostMarketTrading.display_name(), "盘后交易");
    }
}
