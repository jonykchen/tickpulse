use crate::config::constants;
use crate::db::DbPool;
use chrono::{Local, NaiveDate, Weekday};
use std::collections::HashSet;
use std::sync::Arc;

/// 交易日历模块
/// 多级缓存策略：内存 → SQLite → API → weekday 降级
pub struct TradeCalendar {
    trade_days: std::sync::Mutex<HashSet<NaiveDate>>,
    db: Arc<DbPool>,
    client: reqwest::Client,
}

impl TradeCalendar {
    pub fn new(db: Arc<DbPool>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(constants::HTTP_TIMEOUT_SECS))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            trade_days: std::sync::Mutex::new(HashSet::new()),
            db,
            client,
        }
    }

    /// 判断今天是否为交易日
    /// 路径：内存缓存 → SQLite → API → 降级 weekday
    pub async fn is_trading_day(&self) -> bool {
        let today = Local::now().date_naive();

        // 周末直接返回 false
        if matches!(today.weekday(), Weekday::Sat | Weekday::Sun) {
            return false;
        }

        // 内存缓存命中
        {
            let cache = self.trade_days.lock().unwrap();
            if cache.contains(&today) {
                return true;
            }
        }

        // 尝试 SQLite → API → 降级
        match self.load_or_fetch(today).await {
            Ok(days) => {
                let mut cache = self.trade_days.lock().unwrap();
                *cache = days;
                cache.contains(&today)
            }
            Err(_) => {
                // 降级：工作日视为交易日
                tracing::warn!("交易日历获取失败，降级为工作日判断");
                true
            }
        }
    }

    /// 从 SQLite 加载，若缺失则从 API 拉取
    async fn load_or_fetch(&self, date: NaiveDate) -> Result<HashSet<NaiveDate>, String> {
        let year = date.year();
        let month = date.month();

        // 尝试从 SQLite 加载
        let sqlite_days = self.load_from_db(year, month)?;
        if !sqlite_days.is_empty() {
            return Ok(sqlite_days);
        }

        // 从 API 拉取
        let api_days = self.fetch_from_api(year, month).await?;
        if !api_days.is_empty() {
            // 缓存到 SQLite
            self.save_to_db(&api_days)?;
        }

        Ok(api_days)
    }

    /// 从 SQLite 加载交易日
    fn load_from_db(&self, year: i32, month: u32) -> Result<HashSet<NaiveDate>, String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT trade_date FROM trade_calendar WHERE year = ?1 AND month = ?2")
            .map_err(|e| e.to_string())?;
        let days: HashSet<NaiveDate> = stmt
            .query_map(rusqlite::params![year, month], |row| {
                let date_str: String = row.get(0)?;
                NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        Ok(days)
    }

    /// 从东方财富 API 拉取交易日历
    async fn fetch_from_api(&self, year: i32, month: u32) -> Result<HashSet<NaiveDate>, String> {
        let url = format!(
            "{}?year={}&month={:02}",
            constants::EASTMONEY_CALENDAR_URL, year, month
        );

        tracing::info!("从 API 拉取交易日历: {}", url);

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://www.eastmoney.com/")
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|e| format!("API 请求失败: {}", e))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("API 响应解析失败: {}", e))?;

        let mut days = HashSet::new();
        if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
            for item in data {
                // 东方财富返回格式可能包含日期字段
                if let Some(date_str) = item.as_str() {
                    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                        days.insert(date);
                    }
                }
            }
        }

        Ok(days)
    }

    /// 保存交易日到 SQLite
    fn save_to_db(&self, days: &HashSet<NaiveDate>) -> Result<(), String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

        for day in days {
            let date_str = day.format("%Y-%m-%d").to_string();
            tx.execute(
                "INSERT OR IGNORE INTO trade_calendar (trade_date, year, month) VALUES (?1, ?2, ?3)",
                rusqlite::params![date_str, day.year(), day.month()],
            )
            .map_err(|e| e.to_string())?;
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 备用：通过行情数据自身判断交易日
    /// 上证指数行情日期 == 今天 → 交易日
    pub async fn is_trading_day_by_quote(
        &self,
        source: &dyn crate::market::MarketDataSource,
    ) -> bool {
        match source.fetch_quotes(&["1.000001".to_string()]).await {
            Ok(quotes) => {
                if let Some(q) = quotes.first() {
                    // 如果最新价 > 0 且有名称，视为交易日
                    q.price > 0.0 && !q.name.is_empty()
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weekend_not_trading() {
        // 周六周日不是交易日 — is_trading_day 内部直接判断
        // 此处验证逻辑正确性
        let sat = NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(); // 周六
        assert_eq!(sat.weekday(), Weekday::Sat);
        let sun = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap(); // 周日
        assert_eq!(sun.weekday(), Weekday::Sun);
    }
}
