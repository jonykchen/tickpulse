pub mod policy;

use crate::db::DbPool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 分层缓存管理
/// L1: 内存 LRU 缓存（热数据）
/// L2: SQLite kline_cache 表（冷数据）
pub struct CacheManager {
    db: Arc<DbPool>,
    /// 内存 LRU 缓存
    memory_cache: std::sync::Mutex<lru::LruCache<String, CachedKline>>,
}

const MEMORY_CACHE_SIZE: usize = 100;

/// K线缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedKline {
    pub secid: String,
    pub period: String,
    pub adjust: String,
    pub data: String,
    pub cached_at: i64,
    pub ttl: i64,
}

impl CacheManager {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self {
            db,
            memory_cache: std::sync::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(MEMORY_CACHE_SIZE).unwrap(),
            )),
        }
    }

    /// 获取K线缓存
    pub fn get(&self, secid: &str, period: &str, adjust: &str) -> Option<CachedKline> {
        let key = format!("{}:{}:{}", secid, period, adjust);

        // L1: 内存缓存
        {
            let mut cache = self.memory_cache.lock().unwrap();
            if let Some(cached) = cache.get(&key) {
                let now = chrono::Utc::now().timestamp();
                if now - cached.cached_at < cached.ttl {
                    return Some(cached.clone());
                }
                cache.pop(&key); // 过期移除
            }
        }

        // L2: SQLite 缓存
        let conn = self.db.conn().ok()?;
        let mut stmt = conn
            .prepare("SELECT secid, period, adjust, data, cached_at, ttl FROM kline_cache WHERE secid = ?1 AND period = ?2 AND adjust = ?3")
            .ok()?;
        let result = stmt
            .query_row(rusqlite::params![secid, period, adjust], |row| {
                Ok(CachedKline {
                    secid: row.get(0)?,
                    period: row.get(1)?,
                    adjust: row.get(2)?,
                    data: row.get(3)?,
                    cached_at: row.get(4)?,
                    ttl: row.get(5)?,
                })
            })
            .ok()?;

        let now = chrono::Utc::now().timestamp();
        if now - result.cached_at < result.ttl {
            // 写入内存缓存
            let mut cache = self.memory_cache.lock().unwrap();
            cache.put(key, result.clone());
            Some(result)
        } else {
            None
        }
    }

    /// 保存K线缓存
    pub fn put(&self, kline: CachedKline) {
        let key = format!("{}:{}:{}", kline.secid, kline.period, kline.adjust);

        // 写入内存缓存
        {
            let mut cache = self.memory_cache.lock().unwrap();
            cache.put(key, kline.clone());
        }

        // 写入 SQLite
        if let Ok(conn) = self.db.conn() {
            conn.execute(
                "INSERT OR REPLACE INTO kline_cache (secid, period, adjust, data, cached_at, ttl) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![kline.secid, kline.period, kline.adjust, kline.data, kline.cached_at, kline.ttl],
            ).ok();
        }
    }
}
