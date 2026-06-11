use crate::db::{DbPool, Result};
use chrono::Utc;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: i64,
    pub group_id: i64,
    pub secid: String,
    pub name: Option<String>,
    pub cost_price: String,
    pub quantity: String,
    pub sort_order: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

// ==================== 持仓计算器（Decimal 精度） ====================

/// 市值 = 数量 × 现价
pub fn market_value(qty: Decimal, price: Decimal) -> Decimal {
    qty * price
}

/// 浮动盈亏 = 数量 × (现价 - 成本价)
pub fn float_pnl(qty: Decimal, price: Decimal, cost: Decimal) -> Decimal {
    qty * (price - cost)
}

/// 盈亏率 = (现价 - 成本价) / 成本价 × 100
/// cost = 0 时返回 None（防止除零）
pub fn pnl_rate(price: Decimal, cost: Decimal) -> Option<Decimal> {
    if cost.is_zero() {
        return None;
    }
    Some((price - cost) / cost * Decimal::from(100))
}

/// 当日盈亏 = 数量 × (现价 - 昨收价)
pub fn today_pnl(qty: Decimal, price: Decimal, prev_close: Decimal) -> Decimal {
    qty * (price - prev_close)
}

/// 仓位占比 = 个股市值 / 总市值 × 100
/// total = 0 时返回 None
pub fn position_ratio(market_value: Decimal, total_value: Decimal) -> Option<Decimal> {
    if total_value.is_zero() {
        return None;
    }
    Some(market_value / total_value * Decimal::from(100))
}

/// 持仓汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSummary {
    /// 总市值
    pub total_market_value: String,
    /// 总浮动盈亏
    pub total_float_pnl: String,
    /// 总盈亏率(%)
    pub total_pnl_rate: String,
    /// 当日盈亏
    pub total_today_pnl: String,
    /// 当日盈亏率(%)
    pub today_pnl_rate: String,
    /// 基准指数涨跌幅(%)
    pub benchmark_change: Option<String>,
    /// 超额收益(%) = 组合涨跌幅 - 基准涨跌幅
    pub excess_return: Option<String>,
}

/// 持仓行情快照（用于计算）
#[derive(Debug, Clone)]
pub struct PositionQuote {
    pub secid: String,
    pub cost_price: Decimal,
    pub quantity: Decimal,
    pub current_price: Decimal,
    pub prev_close: Decimal,
}

/// 汇总持仓
pub fn summarize(
    positions: &[PositionQuote],
    benchmark_change: Option<Decimal>,
) -> PortfolioSummary {
    let zero = Decimal::ZERO;
    let mut total_cost = zero;
    let mut total_market_val = zero;
    let mut total_float = zero;
    let mut total_today = zero;

    for p in positions {
        let mv = market_value(p.quantity, p.current_price);
        let cost_val = market_value(p.quantity, p.cost_price);
        total_market_val += mv;
        total_cost += cost_val;
        total_float += float_pnl(p.quantity, p.current_price, p.cost_price);
        total_today += today_pnl(p.quantity, p.current_price, p.prev_close);
    }

    let total_pnl_rate = if total_cost.is_zero() {
        Decimal::ZERO
    } else {
        (total_market_val - total_cost) / total_cost * Decimal::from(100)
    };

    // 当日盈亏率 = 当日盈亏 / (总市值 - 当日盈亏) × 100
    // 即当日盈亏相对昨日总市值
    let yesterday_total = total_market_val - total_today;
    let today_pnl_rate = if yesterday_total.is_zero() {
        Decimal::ZERO
    } else {
        total_today / yesterday_total * Decimal::from(100)
    };

    let excess_return = benchmark_change.map(|bc| total_pnl_rate - bc);

    // 保留2位小数输出
    let r2 = |d: Decimal| d.round_dp(2).to_string();
    let r4 = |d: Decimal| d.round_dp(4).to_string();

    PortfolioSummary {
        total_market_value: r2(total_market_val),
        total_float_pnl: r2(total_float),
        total_pnl_rate: r4(total_pnl_rate),
        total_today_pnl: r2(total_today),
        today_pnl_rate: r4(today_pnl_rate),
        benchmark_change: benchmark_change.map(|bc| r4(bc)),
        excess_return: excess_return.map(|er| r4(er)),
    }
}

// ==================== 数据库操作 ====================

impl DbPool {
    /// 获取分组下的持仓
    pub fn get_positions(&self, group_id: i64) -> Result<Vec<Position>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, group_id, secid, name, cost_price, quantity, sort_order, created_at, updated_at
             FROM positions WHERE group_id = ?1 ORDER BY sort_order"
        )?;
        let positions = stmt
            .query_map(rusqlite::params![group_id], |row| {
                Ok(Position {
                    id: row.get(0)?,
                    group_id: row.get(1)?,
                    secid: row.get(2)?,
                    name: row.get(3)?,
                    cost_price: row.get(4)?,
                    quantity: row.get(5)?,
                    sort_order: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(positions)
    }

    /// 添加持仓
    pub fn add_position(
        &self,
        group_id: i64,
        secid: &str,
        name: Option<&str>,
        cost_price: &str,
        quantity: &str,
    ) -> Result<i64> {
        let conn = self.conn()?;
        let now = Utc::now().timestamp();
        let max_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM positions WHERE group_id = ?1",
                rusqlite::params![group_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        conn.execute(
            "INSERT INTO positions (group_id, secid, name, cost_price, quantity, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![group_id, secid, name, cost_price, quantity, max_order, now, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 更新持仓
    pub fn update_position(
        &self,
        id: i64,
        cost_price: &str,
        quantity: &str,
    ) -> Result<()> {
        let conn = self.conn()?;
        let now = Utc::now().timestamp();
        conn.execute(
            "UPDATE positions SET cost_price = ?1, quantity = ?2, updated_at = ?3 WHERE id = ?4",
            rusqlite::params![cost_price, quantity, now, id],
        )?;
        Ok(())
    }

    /// 删除持仓
    pub fn delete_position(&self, id: i64) -> Result<()> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM positions WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    /// 获取所有持仓（跨分组）
    pub fn get_all_positions(&self) -> Result<Vec<Position>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, group_id, secid, name, cost_price, quantity, sort_order, created_at, updated_at
             FROM positions ORDER BY group_id, sort_order"
        )?;
        let positions = stmt
            .query_map([], |row| {
                Ok(Position {
                    id: row.get(0)?,
                    group_id: row.get(1)?,
                    secid: row.get(2)?,
                    name: row.get(3)?,
                    cost_price: row.get(4)?,
                    quantity: row.get(5)?,
                    sort_order: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(positions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_value() {
        let qty = Decimal::from(1000);
        let price = Decimal::from_str("25.50").unwrap();
        let mv = market_value(qty, price);
        assert_eq!(mv, Decimal::from_str("25500").unwrap());
    }

    #[test]
    fn test_float_pnl() {
        let qty = Decimal::from(500);
        let price = Decimal::from_str("30.00").unwrap();
        let cost = Decimal::from_str("25.00").unwrap();
        let pnl = float_pnl(qty, price, cost);
        assert_eq!(pnl, Decimal::from_str("2500").unwrap());
    }

    #[test]
    fn test_pnl_rate() {
        let price = Decimal::from_str("30.00").unwrap();
        let cost = Decimal::from_str("25.00").unwrap();
        let rate = pnl_rate(price, cost).unwrap();
        assert_eq!(rate, Decimal::from_str("20.0000").unwrap());
    }

    #[test]
    fn test_pnl_rate_zero_cost() {
        let price = Decimal::from_str("30.00").unwrap();
        let cost = Decimal::ZERO;
        assert!(pnl_rate(price, cost).is_none());
    }

    #[test]
    fn test_today_pnl() {
        let qty = Decimal::from(1000);
        let price = Decimal::from_str("10.50").unwrap();
        let prev_close = Decimal::from_str("10.00").unwrap();
        let pnl = today_pnl(qty, price, prev_close);
        assert_eq!(pnl, Decimal::from_str("500").unwrap());
    }

    #[test]
    fn test_position_ratio() {
        let mv = Decimal::from_str("25000").unwrap();
        let total = Decimal::from_str("100000").unwrap();
        let ratio = position_ratio(mv, total).unwrap();
        assert_eq!(ratio, Decimal::from_str("25.0000").unwrap());
    }

    #[test]
    fn test_summarize() {
        let positions = vec![
            PositionQuote {
                secid: "1.600519".to_string(),
                cost_price: Decimal::from_str("1800.00").unwrap(),
                quantity: Decimal::from(10),
                current_price: Decimal::from_str("1850.00").unwrap(),
                prev_close: Decimal::from_str("1840.00").unwrap(),
            },
            PositionQuote {
                secid: "0.000001".to_string(),
                cost_price: Decimal::from_str("12.00").unwrap(),
                quantity: Decimal::from(1000),
                current_price: Decimal::from_str("12.50").unwrap(),
                prev_close: Decimal::from_str("12.30").unwrap(),
            },
        ];
        let summary = summarize(&positions, Some(Decimal::from_str("1.5").unwrap()));
        // 茅台市值 18500 + 平安市值 12500 = 31000
        assert_eq!(summary.total_market_value, "31000.00");
        // 浮动盈亏: (1850-1800)*10 + (12.5-12)*1000 = 500+500 = 1000
        assert_eq!(summary.total_float_pnl, "1000.00");
        // 有超额收益
        assert!(summary.excess_return.is_some());
    }
}
