-- v004: 决策记忆闭环表
-- 存储历史决策及其检验结果
CREATE TABLE IF NOT EXISTS decision_memory_v2 (
    id TEXT PRIMARY KEY,
    secid TEXT NOT NULL,
    stock_name TEXT NOT NULL,
    decision_date INTEGER NOT NULL,
    rating TEXT NOT NULL,               -- StrongBuy/Buy/Hold/Sell/StrongSell
    target_price REAL,
    stop_loss REAL,
    reasoning_summary TEXT NOT NULL,    -- 评级理由
    peg_value REAL,
    actual_return REAL,                 -- NULL=pending, T+1实际涨跌幅
    alpha_return REAL,                  -- 超额收益(vs沪深300)
    reflection TEXT,                    -- LLM 反思文本
    status TEXT NOT NULL DEFAULT 'pending',  -- pending/reflected
    created_at INTEGER NOT NULL,
    reflected_at INTEGER,
    UNIQUE(secid, decision_date)
);
CREATE INDEX IF NOT EXISTS idx_decision_memory_v2_secid ON decision_memory_v2(secid);
CREATE INDEX IF NOT EXISTS idx_decision_memory_v2_date ON decision_memory_v2(decision_date);
CREATE INDEX IF NOT EXISTS idx_decision_memory_v2_status ON decision_memory_v2(status);
