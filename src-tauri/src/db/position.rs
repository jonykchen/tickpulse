use crate::db::{DbPool, Result};
use chrono::Utc;
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
}
