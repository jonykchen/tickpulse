use crate::db::{DbPool, Result};
use chrono::Utc;

/// 迁移定义
struct Migration {
    version: i64,
    description: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "initial schema",
        sql: include_str!("migrations/v001_initial.sql"),
    },
    // 后续迁移追加在此，版本号递增
];

/// 执行数据库迁移
/// 在单个事务内执行，失败则整体回滚
pub fn run(pool: &DbPool) -> Result<()> {
    let conn = pool.conn()?;

    // 确保 migrations 表存在
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL,
            description TEXT
        );"
    )?;

    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let tx = conn.unchecked_transaction()?;

    for m in MIGRATIONS {
        if m.version > current_version {
            tracing::info!("执行数据库迁移 v{}: {}", m.version, m.description);
            tx.execute_batch(m.sql)?;
            tx.execute(
                "INSERT INTO migrations (version, applied_at, description) VALUES (?1, ?2, ?3)",
                rusqlite::params![m.version, Utc::now().timestamp(), m.description],
            )?;
        }
    }

    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn test_pool() -> DbPool {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        DbPool::open(&db_path).unwrap()
    }

    #[test]
    fn test_migration_runs_once() {
        let pool = test_pool();
        run(&pool).unwrap();

        let conn = pool.conn().unwrap();
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 1);

        // 再次运行不应重复执行
        run(&pool).unwrap();
        let version2: i64 = conn
            .query_row("SELECT MAX(version) FROM migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version2, 1);
    }

    #[test]
    fn test_tables_created() {
        let pool = test_pool();
        run(&pool).unwrap();

        let conn = pool.conn().unwrap();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"settings".to_string()));
        assert!(tables.contains(&"watchlist_groups".to_string()));
        assert!(tables.contains(&"watchlist_stocks".to_string()));
        assert!(tables.contains(&"positions".to_string()));
        assert!(tables.contains(&"kline_cache".to_string()));
        assert!(tables.contains(&"trade_calendar".to_string()));
        assert!(tables.contains(&"exrights_cache".to_string()));
        assert!(tables.contains(&"alert_rules".to_string()));
        assert!(tables.contains(&"search_history".to_string()));
    }
}
