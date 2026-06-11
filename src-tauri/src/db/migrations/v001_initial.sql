-- v001: Initial Schema
-- 应用配置
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 自选股分组
CREATE TABLE watchlist_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 6),
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 自选股明细
CREATE TABLE watchlist_stocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL REFERENCES watchlist_groups(id) ON DELETE CASCADE,
    secid TEXT NOT NULL,
    name TEXT,
    note TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    UNIQUE(group_id, secid)
);

-- 持仓
CREATE TABLE positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL REFERENCES watchlist_groups(id) ON DELETE CASCADE,
    secid TEXT NOT NULL,
    name TEXT,
    cost_price TEXT NOT NULL,
    quantity TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(group_id, secid)
);

-- K线缓存
CREATE TABLE kline_cache (
    secid TEXT NOT NULL,
    period TEXT NOT NULL,
    adjust TEXT NOT NULL DEFAULT 'forward',
    data TEXT NOT NULL,
    cached_at INTEGER NOT NULL,
    ttl INTEGER NOT NULL,
    PRIMARY KEY (secid, period, adjust)
);

-- 交易日历
CREATE TABLE trade_calendar (
    trade_date TEXT NOT NULL,
    year INTEGER NOT NULL,
    month INTEGER NOT NULL,
    PRIMARY KEY (trade_date)
);
CREATE INDEX idx_trade_calendar_month ON trade_calendar(year, month);

-- 除权除息信息缓存
CREATE TABLE exrights_cache (
    secid TEXT NOT NULL,
    ex_date TEXT NOT NULL,
    bonus_share TEXT NOT NULL,
    allot_share TEXT NOT NULL,
    allot_price TEXT NOT NULL,
    dividend TEXT NOT NULL,
    PRIMARY KEY (secid, ex_date)
);

-- 预警规则
CREATE TABLE alert_rules (
    id TEXT PRIMARY KEY,
    secid TEXT NOT NULL,
    stock_name TEXT NOT NULL,
    rule_type TEXT NOT NULL,
    threshold TEXT NOT NULL DEFAULT '0',
    enabled INTEGER NOT NULL DEFAULT 1,
    triggered INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);
CREATE INDEX idx_alert_rules_secid ON alert_rules(secid);

-- 搜索历史
CREATE TABLE search_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    keyword TEXT NOT NULL,
    searched_at INTEGER NOT NULL
);

-- 插入默认分组
INSERT INTO watchlist_groups (name, sort_order, created_at, updated_at)
VALUES ('默认', 0, strftime('%s','now'), strftime('%s','now'));
