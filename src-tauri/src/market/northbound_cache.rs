//! 北向资金本地缓存
//! SQLite northbound_cache 表 + 同花顺 hsgtApi 替代

use crate::db::{DbPool, Result};
use serde::{Deserialize, Serialize};

/// 北向资金记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NorthboundFlow {
    pub trade_date: String,
    pub sh_net_inflow: f64,
    pub sz_net_inflow: f64,
    pub total_net_inflow: f64,
}

impl DbPool {
    /// 保存北向资金数据
    pub fn save_northbound(&self, data: &NorthboundFlow) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO northbound_cache (trade_date, sh_net_inflow, sz_net_inflow, total_net_inflow, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                data.trade_date,
                data.sh_net_inflow.to_string(),
                data.sz_net_inflow.to_string(),
                data.total_net_inflow.to_string(),
                chrono::Utc::now().timestamp(),
            ],
        )?;
        Ok(())
    }

    /// 获取北向资金最近 N 天数据
    pub fn get_northbound_recent(&self, days: u32) -> Result<Vec<NorthboundFlow>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT trade_date, sh_net_inflow, sz_net_inflow, total_net_inflow
             FROM northbound_cache ORDER BY trade_date DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(rusqlite::params![days], |row| {
            let sh_str: String = row.get(1)?;
            let sz_str: String = row.get(2)?;
            let total_str: String = row.get(3)?;
            Ok(NorthboundFlow {
                trade_date: row.get(0)?,
                sh_net_inflow: sh_str.parse().unwrap_or(0.0),
                sz_net_inflow: sz_str.parse().unwrap_or(0.0),
                total_net_inflow: total_str.parse().unwrap_or(0.0),
            })
        })?;
        rows.filter_map(|r| r.ok()).collect::<Vec<_>>().pipe(Ok)
    }
}

/// Iterator pipe extension — 让链式调用更简洁
trait Pipe<T> {
    fn pipe<U>(self, f: impl FnOnce(T) -> U) -> U;
}

impl<T> Pipe<T> for T {
    fn pipe<U>(self, f: impl FnOnce(T) -> U) -> U {
        f(self)
    }
}
