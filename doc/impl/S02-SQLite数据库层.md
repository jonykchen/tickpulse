# S02 — SQLite 数据库层

> 阶段：Phase 1 | 前置依赖：S01 | 来源：v2.0 §5

## 1. 概述

本文档覆盖 SQLite 数据库层的完整设计，包括连接管理、Schema 定义（全部 CREATE TABLE + CREATE INDEX 语句）以及迁移框架设计。所有 SQL 语句均从 v2.0 §5 逐字抄录，不做简化或省略。

数据库文件随 Tauri 应用数据目录存放，通过 `rusqlite`（bundled 模式）操作，无需系统预装 SQLite。

## 2. 数据库连接管理

### 2.1 连接池设计

Rust 侧使用 `rusqlite` 单连接 + `Mutex` 模式（SQLite 为文件级锁，多连接无收益），通过 `tauri::Manager` state 注入全局。

```rust
use std::sync::Mutex;
use rusqlite::Connection;

pub struct DbPool {
    pub conn: Mutex<Connection>,
}

impl DbPool {
    pub fn open(path: &std::path::Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn: Mutex::new(conn) })
    }
}
```

### 2.2 初始化与注入

在 `lib.rs` 的 Tauri setup 钩子中初始化数据库并注册为 managed state：

```rust
tauri::Builder::default()
    .setup(|app| {
        let app_dir = app.path().app_data_dir()?;
        std::fs::create_dir_all(&app_dir)?;
        let db_path = app_dir.join("tickpulse.db");
        let pool = DbPool::open(&db_path)?;
        db::migrations::run(&pool)?;
        app.manage(pool);
        Ok(())
    })
```

### 2.3 PRAGMA 配置

| PRAGMA | 值 | 说明 |
|--------|-----|------|
| `journal_mode` | `WAL` | Write-Ahead Logging，读写并发友好 |
| `foreign_keys` | `ON` | 启用外键约束（如 `ON DELETE CASCADE`） |

## 3. Schema 定义

以下全部 CREATE TABLE 和 CREATE INDEX 语句均逐字抄录自 v2.0 §5。

### 3.1 settings — 应用配置

```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at INTEGER NOT NULL
);
```

### 3.2 watchlist_groups — 自选股分组

```sql
CREATE TABLE watchlist_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 6),
    sort_order INTEGER NOT NULL DEFAULT 0, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
);
```

### 3.3 watchlist_stocks — 自选股明细

```sql
CREATE TABLE watchlist_stocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL REFERENCES watchlist_groups(id) ON DELETE CASCADE,
    secid TEXT NOT NULL, name TEXT, note TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0, is_pinned INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL, UNIQUE(group_id, secid)
);
```

### 3.4 positions — 持仓

```sql
CREATE TABLE positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL REFERENCES watchlist_groups(id) ON DELETE CASCADE,
    secid TEXT NOT NULL, name TEXT,
    cost_price TEXT NOT NULL, quantity TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
    UNIQUE(group_id, secid)
);
```

### 3.5 kline_cache — K线缓存

```sql
CREATE TABLE kline_cache (
    secid TEXT NOT NULL, period TEXT NOT NULL, adjust TEXT NOT NULL DEFAULT 'forward',
    data TEXT NOT NULL, cached_at INTEGER NOT NULL, ttl INTEGER NOT NULL,
    PRIMARY KEY (secid, period, adjust)
);
```

### 3.6 trade_calendar — 交易日历

```sql
CREATE TABLE trade_calendar (
    trade_date TEXT NOT NULL, year INTEGER NOT NULL, month INTEGER NOT NULL,
    PRIMARY KEY (trade_date)
);
CREATE INDEX idx_trade_calendar_month ON trade_calendar(year, month);
```

### 3.7 exrights_cache — 除权除息信息缓存（v2.0）

```sql
-- v2.0: 除权除息信息缓存
CREATE TABLE exrights_cache (
    secid TEXT NOT NULL, ex_date TEXT NOT NULL,
    bonus_share TEXT NOT NULL,    -- 每股送转股
    allot_share TEXT NOT NULL,    -- 每股配股
    allot_price TEXT NOT NULL,    -- 配股价
    dividend TEXT NOT NULL,       -- 每股派息
    PRIMARY KEY (secid, ex_date)
);
```

### 3.8 alert_rules — 预警规则

```sql
CREATE TABLE alert_rules (
    id TEXT PRIMARY KEY, secid TEXT NOT NULL, stock_name TEXT NOT NULL,
    rule_type TEXT NOT NULL, enabled INTEGER NOT NULL DEFAULT 1,
    triggered INTEGER NOT NULL DEFAULT 0, created_at INTEGER NOT NULL
);
CREATE INDEX idx_alert_rules_secid ON alert_rules(secid);
```

### 3.9 search_history — 搜索历史

```sql
CREATE TABLE search_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT, keyword TEXT NOT NULL, searched_at INTEGER NOT NULL
);
```

### 3.10 migrations — 迁移版本记录

```sql
CREATE TABLE migrations (
    version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL, description TEXT
);
```

## 4. 迁移框架设计

### 4.1 核心思路

- `migrations` 表记录已执行的迁移版本号。
- 每次应用启动时，对比代码中定义的迁移列表与 `migrations` 表记录，按版本号顺序执行未应用的迁移。
- 迁移在单个事务内执行，失败则整体回滚，应用无法启动（避免半迁移状态）。

### 4.2 迁移定义

```rust
// src-tauri/src/db/migrations.rs

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
```

### 4.3 迁移执行逻辑

```rust
pub fn run(pool: &DbPool) -> Result<()> {
    let conn = pool.conn.lock().unwrap();

    // 确保 migrations 表存在
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS migrations (
            version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL, description TEXT
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
            tx.execute_batch(m.sql)?;
            tx.execute(
                "INSERT INTO migrations (version, applied_at, description) VALUES (?1, ?2, ?3)",
                rusqlite::params![m.version, chrono::Utc::now().timestamp(), m.description],
            )?;
        }
    }

    tx.commit()?;
    Ok(())
}
```

### 4.4 迁移文件组织

```
src-tauri/src/db/
├── mod.rs                # DbPool + 公共函数
├── migrations.rs         # 迁移执行逻辑 + MIGRATIONS 列表
├── migrations/           # SQL 文件目录
│   └── v001_initial.sql  # 包含全部初始 CREATE TABLE + CREATE INDEX
├── watchlist.rs          # 自选股 CRUD
├── position.rs           # 持仓 CRUD
└── settings.rs           # 配置读写
```

### 4.5 迁移编写规范

| 规则 | 说明 |
|------|------|
| 版本号递增 | 每次新增迁移版本号严格递增，不可复用已用版本号 |
| 仅增不改 | 迁移只允许 `ALTER TABLE ADD COLUMN` / `CREATE TABLE` / `CREATE INDEX`，禁止修改或删除已有列 |
| 向后兼容 | 新列必须有 `DEFAULT` 值或允许 `NULL`，确保旧代码不崩溃 |
| 事务保护 | 迁移在单事务内执行，失败整体回滚 |
| 测试覆盖 | 每个迁移需编写从上一版本升级的集成测试 |

## 5. 文件清单

| 文件路径 | 职责 |
|----------|------|
| `src-tauri/src/db/mod.rs` | `DbPool` 定义、连接打开、PRAGMA 配置、公共查询函数 |
| `src-tauri/src/db/migrations.rs` | 迁移执行逻辑、`MIGRATIONS` 列表定义 |
| `src-tauri/src/db/migrations/v001_initial.sql` | 初始 Schema（全部 CREATE TABLE + CREATE INDEX） |
| `src-tauri/src/db/watchlist.rs` | 自选股分组 + 股票 CRUD |
| `src-tauri/src/db/position.rs` | 持仓 CRUD |
| `src-tauri/src/db/settings.rs` | 配置读写 |
