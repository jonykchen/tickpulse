-- v002: 扩展表结构
-- 大宗交易
CREATE TABLE IF NOT EXISTS block_trades (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    secid TEXT NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    trade_date TEXT NOT NULL,
    price TEXT NOT NULL,
    volume TEXT NOT NULL,
    amount TEXT NOT NULL,
    buyer TEXT,
    seller TEXT,
    premium_rate TEXT,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_block_trades_date ON block_trades(trade_date);
CREATE INDEX IF NOT EXISTS idx_block_trades_secid ON block_trades(secid);

-- 北向资金缓存
CREATE TABLE IF NOT EXISTS northbound_cache (
    trade_date TEXT PRIMARY KEY,
    sh_net_inflow TEXT NOT NULL,
    sz_net_inflow TEXT NOT NULL,
    total_net_inflow TEXT NOT NULL,
    cached_at INTEGER NOT NULL
);
