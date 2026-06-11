pub mod migrations;
pub mod position;
pub mod settings;
pub mod watchlist;

use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Lock poisoned")]
    LockPoisoned,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, Connection>>> for DbError {
    fn from(_: std::sync::PoisonError<std::sync::MutexGuard<'_, Connection>>) -> Self {
        DbError::LockPoisoned
    }
}

pub type Result<T> = std::result::Result<T, DbError>;

/// SQLite 连接池（单连接 + Mutex 模式）
/// SQLite 为文件级锁，多连接无收益
#[derive(Clone)]
pub struct DbPool {
    pub conn: Arc<Mutex<Connection>>,
    db_path: std::path::PathBuf,
}

impl DbPool {
    /// 打开数据库连接并配置 PRAGMA
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: path.to_path_buf(),
        })
    }

    /// 获取连接锁
    pub fn conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|_| DbError::LockPoisoned)
    }

    /// 获取数据库文件大小（字节）
    pub fn db_file_size(&self) -> Result<u64> {
        let metadata = std::fs::metadata(&self.db_path)?;
        Ok(metadata.len())
    }

    /// 内存数据库（仅供测试使用）
    #[cfg(test)]
    pub fn open_in_memory_for_test() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .unwrap();
        Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: std::path::PathBuf::from(":memory:"),
        }
    }
}
