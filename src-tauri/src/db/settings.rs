use crate::db::{DbPool, Result};
use chrono::Utc;

impl DbPool {
    /// 获取配置值
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let result = stmt
            .query_row(rusqlite::params![key], |row| row.get(0))
            .ok();
        Ok(result)
    }

    /// 获取所有配置
    pub fn get_all_settings(&self) -> Result<Vec<(String, String)>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let settings = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(settings)
    }

    /// 更新或插入配置
    pub fn upsert_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn()?;
        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3",
            rusqlite::params![key, value, now],
        )?;
        Ok(())
    }

    /// 删除配置
    pub fn delete_setting(&self, key: &str) -> Result<()> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM settings WHERE key = ?1", rusqlite::params![key])?;
        Ok(())
    }
}
