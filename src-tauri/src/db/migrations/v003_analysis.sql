-- v003: AI 分析引擎表
-- 分析结果
CREATE TABLE IF NOT EXISTS analysis_results (
    id TEXT PRIMARY KEY,
    secid TEXT NOT NULL,
    stock_name TEXT NOT NULL,
    overall_rating TEXT NOT NULL,
    overall_score REAL NOT NULL,
    bull_argument TEXT,
    bear_argument TEXT,
    verdict TEXT,
    quality_score REAL NOT NULL DEFAULT 0,
    dimensions_json TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_analysis_results_secid ON analysis_results(secid);
CREATE INDEX IF NOT EXISTS idx_analysis_results_created ON analysis_results(created_at);

-- PEG 缓存
CREATE TABLE IF NOT EXISTS peg_cache (
    secid TEXT NOT NULL,
    trade_date TEXT NOT NULL,
    pe_ttm REAL NOT NULL,
    cagr REAL NOT NULL,
    peg_value REAL NOT NULL,
    peg_rating TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (secid, trade_date)
);

-- 行业 PE 缓存
CREATE TABLE IF NOT EXISTS industry_pe_cache (
    industry_code TEXT NOT NULL,
    trade_date TEXT NOT NULL,
    pe_median REAL NOT NULL,
    pe_mean REAL NOT NULL,
    company_count INTEGER NOT NULL,
    cached_at INTEGER NOT NULL,
    PRIMARY KEY (industry_code, trade_date)
);

-- LLM 配置
CREATE TABLE IF NOT EXISTS llm_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    api_key_encrypted TEXT,
    base_url TEXT,
    mode TEXT NOT NULL DEFAULT 'cloud',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 决策记忆
CREATE TABLE IF NOT EXISTS decision_memory (
    id TEXT PRIMARY KEY,
    secid TEXT NOT NULL,
    decision_type TEXT NOT NULL,
    decision_text TEXT NOT NULL,
    context_json TEXT,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_decision_memory_secid ON decision_memory(secid);

-- 分析检查点（续传）
CREATE TABLE IF NOT EXISTS analysis_checkpoints (
    id TEXT PRIMARY KEY,
    secid TEXT NOT NULL,
    stock_name TEXT NOT NULL,
    analysis_date TEXT NOT NULL,
    completed_dimensions TEXT NOT NULL,  -- JSON array
    pending_dimensions TEXT NOT NULL,   -- JSON array
    partial_results TEXT NOT NULL,      -- JSON object
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_checkpoints_secid_date ON analysis_checkpoints(secid, analysis_date);
