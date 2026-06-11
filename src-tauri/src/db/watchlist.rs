use crate::db::{DbPool, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistGroup {
    pub id: i64,
    pub name: String,
    pub sort_order: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistStock {
    pub id: i64,
    pub group_id: i64,
    pub secid: String,
    pub name: Option<String>,
    pub note: Option<String>,
    pub sort_order: i64,
    pub is_pinned: bool,
    pub created_at: i64,
}

impl DbPool {
    /// 获取所有自选股分组
    pub fn get_watchlist_groups(&self) -> Result<Vec<WatchlistGroup>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, sort_order, created_at, updated_at FROM watchlist_groups ORDER BY sort_order"
        )?;
        let groups = stmt
            .query_map([], |row| {
                Ok(WatchlistGroup {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    sort_order: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(groups)
    }

    /// 创建自选股分组
    pub fn create_watchlist_group(&self, name: &str) -> Result<i64> {
        let conn = self.conn()?;
        let now = Utc::now().timestamp();
        let max_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM watchlist_groups",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        conn.execute(
            "INSERT INTO watchlist_groups (name, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![name, max_order, now, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 删除自选股分组（级联删除组内股票）
    pub fn delete_watchlist_group(&self, id: i64) -> Result<()> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM watchlist_groups WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    /// 重命名分组
    pub fn rename_watchlist_group(&self, id: i64, name: &str) -> Result<()> {
        let conn = self.conn()?;
        let now = Utc::now().timestamp();
        conn.execute(
            "UPDATE watchlist_groups SET name = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![name, now, id],
        )?;
        Ok(())
    }

    /// 获取分组下的自选股
    pub fn get_watchlist_stocks(&self, group_id: i64) -> Result<Vec<WatchlistStock>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, group_id, secid, name, note, sort_order, is_pinned, created_at
             FROM watchlist_stocks
             WHERE group_id = ?1
             ORDER BY is_pinned DESC, sort_order"
        )?;
        let stocks = stmt
            .query_map(rusqlite::params![group_id], |row| {
                Ok(WatchlistStock {
                    id: row.get(0)?,
                    group_id: row.get(1)?,
                    secid: row.get(2)?,
                    name: row.get(3)?,
                    note: row.get(4)?,
                    sort_order: row.get(5)?,
                    is_pinned: row.get::<_, i64>(6)? != 0,
                    created_at: row.get(7)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(stocks)
    }

    /// 添加自选股
    pub fn add_watchlist_stock(
        &self,
        group_id: i64,
        secid: &str,
        name: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn()?;
        let now = Utc::now().timestamp();
        let max_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM watchlist_stocks WHERE group_id = ?1",
                rusqlite::params![group_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        conn.execute(
            "INSERT INTO watchlist_stocks (group_id, secid, name, sort_order, is_pinned, created_at)
             VALUES (?1, ?2, ?3, ?4, 0, ?5)",
            rusqlite::params![group_id, secid, name, max_order, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 删除自选股
    pub fn remove_watchlist_stock(&self, id: i64) -> Result<()> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM watchlist_stocks WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    /// 置顶/取消置顶自选股
    pub fn toggle_pin_watchlist_stock(&self, id: i64, pinned: bool) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE watchlist_stocks SET is_pinned = ?1 WHERE id = ?2",
            rusqlite::params![pinned as i64, id],
        )?;
        Ok(())
    }

    /// 获取所有自选股的 secid（跨分组去重）
    pub fn get_all_watchlist_secids(&self) -> Result<Vec<String>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT DISTINCT secid FROM watchlist_stocks")?;
        let secids = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(secids)
    }

    /// 更新自选股备注
    pub fn update_watchlist_note(&self, id: i64, note: Option<&str>) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE watchlist_stocks SET note = ?1 WHERE id = ?2",
            rusqlite::params![note, id],
        )?;
        Ok(())
    }
}
