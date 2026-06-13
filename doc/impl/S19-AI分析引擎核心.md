# S19 — AI 分析引擎核心

> 阶段：Phase 8-10 | 前置依赖：S18 | 来源：v3.0 §0-1, §5-9, §10.1-10.9, §12.1,12.5-12.7,12.9, §14.1-14.7, 附录E, §13, §15

## 1. 概述

本文档是 AI 分析引擎核心的完整实现规格，涵盖引擎架构、LLM 接口、七维度分析师、PEG 估值模块、数据层扩展、盘后自动分析、决策记忆闭环、质量门控、检查点续传、结构化输出降级、Token 预算控制、A股交易约束、分析师必采清单、多空辩论、分析预设、消息清理、语言策略、LLM 供应商兼容性、模型推荐、进度追踪、A股特化 Prompt、三方风险辩论、原子写入、成本预估等全部子模块。

## 2. 架构概述（§0）

### 2.1 三个开源项目分析结论

| 项目 | 核心价值 | 可借鉴思路 | 落地方案 |
|------|----------|-----------|---------|
| **TradingAgents-astock** | 多Agent辩论架构（7角色 + Bull/Bear辩论 + 三方风险辩论） | ① 多维度分析角色分工 ② 结构化输出（5级评级 Buy/Overweight/Hold/Underweight/Sell） ③ A股特化角色（政策/游资/解禁） ④ 双LLM设计（快速思考+深度思考） | AI分析引擎：7维度角色→结构化评级→综合报告 |
| **a-stock-data** | 7层27端点全栈数据（零第三方依赖） | ① 数据源优先级策略（mootdx/腾讯不封IP优先，东财仅用于独有数据） ② 东财统一限流入口 em_get() ③ 腾讯财经实时PE/PB/市值 ④ 概念板块归属+资金流向+股东户数+研报 | 新增数据源层：腾讯财经+东财限流+概念板块+资金面 |
| **astock-peg** | PEG估值分析（林奇投资法本地化） | ① PEG = 前瞻PE / CAGR 评级体系 ② PE消化年限计算 ③ AI自动生成7节结构化估值报告 ④ 行业PE横向对比 ⑤ 前端Next.js看板 | PEG看板+AI报告+行业PE对比+分析历史 |

### 2.2 v2.0 → v3.0 架构演进

```
v2.0 架构（行情监控）：
  数据源 → 行情调度 → 涨跌停/异动检测 → 实时展示

v3.0 架构（分析辅助决策）：
  数据源（+腾讯/mootdx/研报/解禁）→ 行情调度 → 实时展示
                                                      ↓
                                              AI 分析引擎（新增）
                                           ┌──┤ 7维度角色分析
                                           │  ├ PEG 估值评估
                                           │  ├ 行业横向对比
                                           │  ├ 结构化评级输出
                                           │  └ 综合分析报告
                                           ↓
                                      分析看板 + 历史归档
```

### 2.3 核心设计原则（源自开源项目经验）

1. **数据源优先级策略**（来自 a-stock-data）：能用 mootdx/腾讯拿到的数据不用东财，东财仅用于独有数据（龙虎榜/解禁/研报），所有东财请求走统一限流入口
2. **分析角色分工**（来自 TradingAgents-astock）：每个分析维度独立采集→独立出报告→综合辩论→结构化评级，而非一个大 prompt 万能
3. **PEG 评级体系**（来自 astock-peg）：PEG < 0.5 极度低估 / 0.5-1.0 低估 / 1.0-1.5 合理 / 1.5-2.0 偏贵 / > 2.0 高估
4. **零幻觉原则**（来自 astock-peg）：分析报告严禁估算或编造数据，所有数值必须引用真实字段，无法获取标注 [数据缺失]

## 3. AI引擎架构（§1）

### 3.1 引擎架构图

```
┌────────────────────────────────────────────────────────────────────┐
│                    AI Analysis Engine (Rust)                        │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  LLM Router（双模）                                          │  │
│  │  ┌────────────────┐  ┌────────────────┐                      │  │
│  │  │ 本地 LLM       │  │ 云端 API       │                      │  │
│  │  │ Ollama/llama   │  │ Claude/GPT/    │                      │  │
│  │  │ (隐私优先)     │  │ DeepSeek/Qwen  │                      │  │
│  │  └────────────────┘  └────────────────┘                      │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  7维度分析师（并行采集 + 独立出报告）                          │  │
│  │                                                              │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐       │  │
│  │  │ 📊技术面  │ │ 📰新闻面  │ │ 💰基本面  │ │ 🏛️政策面  │       │  │
│  │  │ K线形态   │ │ 公告/新闻 │ │ 财报/估值 │ │ 监管/产业 │       │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘       │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐                    │  │
│  │  │ 🔥游资追踪│ │ 🔓解禁监控│ │ 📈PEG估值 │                    │  │
│  │  │ 龙虎/资金 │ │ 减持/解禁 │ │ PE/增速   │                    │  │
│  │  └──────────┘ └──────────┘ └──────────┘                    │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  综合决策层                                                    │  │
│  │  1. 多维度报告汇总                                            │  │
│  │  2. 结构化评级（5级：Buy/Overweight/Hold/Underweight/Sell）    │  │
│  │  3. 综合分析报告生成（Markdown + PDF导出）                      │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  触发方式                                                      │  │
│  │  ① 手动：股票详情页「AI分析」按钮                               │  │
│  │  ② 自动：盘后15:30自动分析自选股（可配置）                       │  │
│  │  ③ 条件：异动触发自动分析（如封板/炸板/量比突增）                  │  │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
```

### 3.2 LLM 接口设计（双模 + 多供应商）

```rust
/// LLM 供应商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub mode: LlmMode,
    pub cloud_provider: CloudProvider,
    pub api_key: Option<String>,      // 云端模式必需
    pub base_url: Option<String>,     // 自定义端点（中转/Ollama）
    pub model: Option<String>,        // 模型名
    pub quick_model: Option<String>,  // 快速思考模型
    pub deep_model: Option<String>,   // 深度思考模型
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmMode {
    Local,   // Ollama 本地（隐私优先，延迟高）
    Cloud,   // 云端 API（速度快，需网络）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudProvider {
    Anthropic,
    OpenAI,
    DeepSeek,   // 兼容 OpenAI 格式
    Qwen,       // 兼容 OpenAI 格式
    GLM,        // 兼容 OpenAI 格式
    Custom,     // 任意 OpenAI 兼容端点
}

/// 双 LLM 设计（源自 TradingAgents-astock）
/// - quick_think: 数据采集、简单判断（速度快、成本低）
/// - deep_think: 综合决策、估值分析（质量高、推理深）
pub struct AnalysisEngine {
    quick_llm: Arc<dyn LlmClient>,
    deep_llm: Arc<dyn LlmClient>,
    data_collector: Arc<DataCollector>,
    report_store: Arc<ReportStore>,
}
```

### 3.3 七维度分析师定义

```rust
/// 分析维度（源自 TradingAgents-astock 的 7 个 Analyst 角色）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnalysisDimension {
    Technical,      // 技术面：K线形态、技术指标、量价分析
    News,           // 新闻面：行业新闻、公告、宏观事件
    Fundamentals,   // 基本面：财报三表、盈利能力、估值
    Policy,         // 政策面：监管政策、产业政策（A股特色）
    HotMoney,       // 游资追踪：龙虎榜、大单流向、主力资金（A股特色）
    Lockup,         // 解禁监控：限售解禁、大股东减持（A股特色）
    Peg,            // PEG估值：前瞻PE/CAGR/消化年限（新增）
}

/// 每个维度的分析报告
#[derive(Debug, Clone, Serialize)]
pub struct DimensionReport {
    pub dimension: AnalysisDimension,
    pub rating: DimensionRating,      // 利好/中性/利空
    pub confidence: f64,              // 0.0-1.0 置信度
    pub summary: String,              // 1-2句核心结论
    pub details: String,              // 完整分析文本（Markdown）
    pub data_sources: Vec<String>,    // 使用的数据源列表
    pub missing_data: Vec<String>,    // 缺失的数据点
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum DimensionRating {
    StrongPositive,  // 重大利好
    Positive,        // 利好
    Neutral,         // 中性
    Negative,        // 利空
    StrongNegative,  // 重大利空
}
```

### 3.4 PEG 估值分析模块（源自 astock-peg）

```rust
/// PEG 估值计算引擎
pub struct PegCalculator;

impl PegCalculator {
    /// PEG 评级（源自 astock-peg）
    #[derive(Debug, Clone, Copy, Serialize)]
    pub enum PegRating {
        ExtremelyUndervalued,  // PEG < 0.5
        Undervalued,           // 0.5 ≤ PEG < 1.0
        Fair,                  // 1.0 ≤ PEG < 1.5
        Expensive,             // 1.5 ≤ PEG < 2.0
        Overvalued,            // PEG ≥ 2.0
    }

    /// 计算 PEG 评级
    pub fn rate(peg: Decimal) -> PegRating {
        if peg < Decimal::from_str("0.5").unwrap() { PegRating::ExtremelyUndervalued }
        else if peg < Decimal::ONE { PegRating::Undervalued }
        else if peg < Decimal::from_str("1.5").unwrap() { PegRating::Fair }
        else if peg < Decimal::from_str("2.0").unwrap() { PegRating::Expensive }
        else { PegRating::Overvalued }
    }

    /// PE 消化年限 = ln(前瞻PE / 30) / ln(1 + CAGR)
    /// < 2年 = 成长性强，2-4年 = 正常，> 4年 = 需谨慎
    pub fn pe_digestion_years(forward_pe: Decimal, cagr: Decimal) -> Option<Decimal> {
        if forward_pe <= Decimal::ZERO || cagr <= Decimal::ZERO { return None; }
        let ratio = (forward_pe / Decimal::from(30)).to_f64()?;
        let growth = (Decimal::ONE + cagr).to_f64()?;
        if ratio <= 0.0 || growth <= 0.0 { return None; }
        Some(Decimal::from_f64_retain(ratio.ln() / growth.ln())?)
    }

    /// CAGR 计算：近3年净利润复合增速
    /// CAGR = (终值/初值)^(1/年数) - 1
    pub fn calc_cagr(beginning: Decimal, ending: Decimal, years: u32) -> Option<Decimal> {
        if beginning <= Decimal::ZERO { return None; }
        let ratio = (ending / beginning).to_f64()?;
        let cagr = ratio.powf(1.0 / years as f64) - 1.0;
        Some(Decimal::from_f64_retain(cagr)?)
    }
}
```

### 3.5 综合决策输出（源自 TradingAgents-astock 结构化输出）

```rust
/// 综合分析结果
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisResult {
    pub id: String,                           // 分析ID
    pub secid: String,                        // 股票标识
    pub stock_name: String,                   // 股票名称
    pub analyzed_at: i64,                     // 分析时间戳
    pub trigger: AnalysisTrigger,             // 触发方式

    // 7维度报告
    pub dimensions: Vec<DimensionReport>,

    // 综合评级（5级，源自 TradingAgents-astock PortfolioRating）
    pub overall_rating: OverallRating,
    pub rating_rationale: String,             // 评级理由

    // PEG 估值（核心指标）
    pub peg_summary: Option<PegSummary>,

    // 风险提示
    pub risk_warnings: Vec<String>,

    // 完整报告 Markdown
    pub full_report: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum OverallRating {
    Buy,          // 强烈看多，建议建仓/加仓
    Overweight,   // 偏多，可逐步增仓
    Hold,         // 中性，维持持仓
    Underweight,  // 偏空，建议减仓
    Sell,         // 强烈看空，建议清仓
}

#[derive(Debug, Clone, Serialize)]
pub struct PegSummary {
    pub pe_ttm: Option<Decimal>,              // TTM市盈率
    pub forward_pe: Option<Decimal>,          // 前瞻PE（基于一致预期）
    pub cagr: Option<Decimal>,                // 净利润复合增速
    pub peg: Option<Decimal>,                 // PEG值
    pub peg_rating: Option<PegCalculator::PegRating>,
    pub digestion_years: Option<Decimal>,     // PE消化年限
    pub industry_avg_pe: Option<Decimal>,     // 行业平均PE
    pub pe_percentile: Option<f64>,           // PE在行业中百分位
}

#[derive(Debug, Clone, Serialize)]
pub enum AnalysisTrigger {
    Manual,                                    // 手动触发
    AutoPostMarket,                            // 盘后自动（15:30）
    Anomaly { anomaly_type: String },          // 异动触发
}
```

## 4. 数据层（§5-6）

### 4.1 SQLite Schema 扩展

```sql
-- AI 分析结果
CREATE TABLE analysis_results (
    id TEXT PRIMARY KEY,                    -- UUID
    secid TEXT NOT NULL,
    stock_name TEXT NOT NULL,
    analyzed_at INTEGER NOT NULL,           -- 时间戳
    trigger_type TEXT NOT NULL,             -- manual/auto_post_market/anomaly
    overall_rating TEXT NOT NULL,           -- Buy/Overweight/Hold/Underweight/Sell
    peg_value TEXT,                         -- PEG数值
    peg_rating TEXT,                        -- ExtremelyUndervalued/Undervalued/Fair/Expensive/Overvalued
    report_markdown TEXT NOT NULL,          -- 完整报告
    dimensions_json TEXT NOT NULL,          -- 7维度报告JSON
    risk_warnings_json TEXT,                -- 风险提示JSON
    created_at INTEGER NOT NULL
);
CREATE INDEX idx_analysis_secid ON analysis_results(secid);
CREATE INDEX idx_analysis_date ON analysis_results(analyzed_at);

-- PEG 看板缓存
CREATE TABLE peg_cache (
    secid TEXT PRIMARY KEY,
    pe_ttm TEXT,
    forward_pe TEXT,
    cagr TEXT,
    peg TEXT,
    peg_rating TEXT,
    digestion_years TEXT,
    industry_avg_pe TEXT,
    pe_percentile REAL,
    consensus_eps TEXT,                     -- 一致预期EPS
    updated_at INTEGER NOT NULL
);

-- 行业PE对比缓存
CREATE TABLE industry_pe_cache (
    industry TEXT NOT NULL,
    secid TEXT NOT NULL,
    pe_ttm TEXT,
    market_cap_yi REAL,
    rank_in_industry INTEGER,
    total_in_industry INTEGER,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (industry, secid)
);

-- 解禁日历缓存
CREATE TABLE lockup_cache (
    secid TEXT NOT NULL,
    unlock_date TEXT NOT NULL,
    unlock_volume TEXT,                     -- 解禁数量
    unlock_ratio TEXT,                      -- 占流通比
    unlock_type TEXT,                       -- 首发原股东/定增/股权激励/战略配售
    is_future INTEGER NOT NULL DEFAULT 0,   -- 0=已解禁, 1=待解禁
    PRIMARY KEY (secid, unlock_date)
);

-- 股东户数缓存
CREATE TABLE shareholder_count_cache (
    secid TEXT NOT NULL,
    report_date TEXT NOT NULL,
    shareholder_count INTEGER,
    qoq_change_pct REAL,                   -- 环比变化%
    avg_shares_per_holder REAL,             -- 户均持股
    PRIMARY KEY (secid, report_date)
);

-- LLM 配置
CREATE TABLE llm_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
-- 初始数据: mode, cloud_provider, api_key(加密), base_url, model, quick_model, deep_model
```

### 4.2 新增 Command

| Command | 参数 | 返回值 | 说明 |
|---------|------|--------|------|
| `trigger_analysis` | `secid, trigger` | `AnalysisResult` | 触发AI分析（7维度） |
| `get_analysis_result` | `secid` | `Option<AnalysisResult>` | 获取最新分析结果 |
| `get_analysis_history` | `secid, limit?` | `Vec<AnalysisRecord>` | 历史分析列表 |
| `export_analysis_pdf` | `analysis_id` | `path` | 导出PDF |
| `fetch_peg_board` | `secids` | `Vec<PegSummary>` | PEG看板批量数据 |
| `fetch_industry_comparison` | `secid` | `IndustryPegData` | 行业PE对比 |
| `fetch_concept_blocks` | `secid` | `Vec<ConceptBlock>` | 概念板块归属 |
| `fetch_lockup_calendar` | `secid` | `Vec<LockupEvent>` | 解禁日历 |
| `fetch_fund_flow` | `secid` | `FundFlowData` | 资金流向 |
| `fetch_shareholder_count` | `secid` | `Vec<ShareholderCount>` | 股东户数变化 |
| `fetch_research_reports` | `secid, limit?` | `Vec<ResearchReport>` | 研报列表 |
| `fetch_consensus_eps` | `secid` | `Vec<ConsensusEps>` | 一致预期EPS |
| `configure_llm` | `LlmConfig` | `()` | 配置LLM供应商 |
| `test_llm_connection` | — | `bool` | 测试LLM连通性 |

### 4.3 新增 Event

| Event | Payload | 触发时机 |
|-------|---------|----------|
| `analysis-progress` | `AnalysisProgress` | 分析维度完成时 |
| `analysis-completed` | `AnalysisResult` | 分析完成时 |
| `peg-updated` | `PegSummary` | PEG数据更新时 |

## 5. 盘后自动分析（§7-9）

### 5.1 盘后自动分析7步流程

```
15:30 收盘 → 触发自动分析
              │
              ├── 1. 判断是否交易日（复用 TradeCalendar）
              ├── 2. 遍历自选股列表
              ├── 3. 并行采集7维度数据（Python sidecar）
              │   ├── 技术面：K线+指标（mootdx，已有）
              │   ├── 新闻面：公告+新闻（东财，已有限流）
              │   ├── 基本面：财报+估值（mootdx+腾讯，已有限流）
              │   ├── 政策面：行业政策（东财新闻，已有限流）
              │   ├── 游资追踪：龙虎榜+资金流（东财，已有限流）
              │   ├── 解禁监控：解禁日历+减持（东财，已有限流）
              │   └── PEG估值：PE+EPS+CAGR（腾讯+同花顺，不限流）
              ├── 4. 7维度报告并行生成（quick_think LLM）
              ├── 5. 综合决策+评级（deep_think LLM）
              ├── 6. 存储分析结果（SQLite）
              └── 7. 推送通知「XX股分析完成：Hold（中性）PEG=1.2」
```

**防封策略**：7维度并行采集时，东财系数据走 em_get 串行限流，mootdx/腾讯可并行不限。

### 5.2 项目结构扩展

```
tickpulse/
├── src-tauri/src/
│   ├── analysis/                    # 🆕 AI分析引擎
│   │   ├── mod.rs                   # AnalysisEngine 主入口
│   │   ├── engine.rs                # 分析流程编排
│   │   ├── dimensions/              # 7维度分析师
│   │   │   ├── mod.rs
│   │   │   ├── technical.rs         # 技术面分析
│   │   │   ├── news.rs              # 新闻面分析
│   │   │   ├── fundamentals.rs      # 基本面分析
│   │   │   ├── policy.rs            # 政策面分析（A股特色）
│   │   │   ├── hot_money.rs         # 游资追踪（A股特色）
│   │   │   ├── lockup.rs            # 解禁监控（A股特色）
│   │   │   └── peg.rs               # PEG估值分析
│   │   ├── llm/                     # LLM 客户端
│   │   │   ├── mod.rs               # LlmRouter
│   │   │   ├── anthropic.rs         # Claude API
│   │   │   ├── openai_compat.rs     # OpenAI/DeepSeek/Qwen/GLM
│   │   │   └── ollama.rs            # 本地 LLM
│   │   ├── report.rs                # 报告生成+渲染
│   │   ├── schemas.rs               # 结构化输出 Schema
│   │   └── auto_trigger.rs          # 盘后/异动自动触发
│   ├── market/sources/
│   │   ├── tencent.rs               # 🆕 腾讯财经数据源
│   │   ├── mootdx.rs                # 🆕 mootdx bridge（via sidecar）
│   │   └── em_rate_limiter.rs       # 🆕 东财统一限流器
│   ├── sidecar/                     # 🆕 Python sidecar 管理
│   │   ├── mod.rs                   # SidecarManager
│   │   └── protocol.rs              # JSON-RPC 协议
│   └── db/
│       ├── analysis.rs              # 🆕 分析结果 CRUD
│       ├── peg_cache.rs             # 🆕 PEG缓存
│       └── lockup_cache.rs          # 🆕 解禁缓存
├── sidecar/                         # 🆕 Python 数据采集子进程
│   ├── main.py                      # 入口
│   ├── datafeeds.py                 # 数据源封装（复用 a-stock-data）
│   ├── collect_stock_data.py        # 个股全量采集（复用 astock-peg）
│   ├── detect_sector.py             # 行业检测（复用 astock-peg）
│   └── requirements.txt             # mootdx, requests, pandas, stockstats
├── src/                             # Vue 前端
│   ├── views/
│   │   ├── AnalysisDashboard.vue    # 🆕 AI分析主页
│   │   └── AnalysisHistory.vue      # 🆕 历史记录
│   ├── components/
│   │   ├── analysis/                # 🆕 分析组件
│   │   │   ├── DimensionCard.vue
│   │   │   ├── OverallRatingBar.vue
│   │   │   ├── PegRatingTag.vue
│   │   │   └── AnalysisTriggerConfig.vue
│   │   └── charts/
│   │       ├── IndustryPegChart.vue # 🆕 行业PE对比
│   │       └── PegTrendChart.vue    # 🆕 PEG趋势
│   └── stores/
│       └── analysis.ts              # 🆕 分析 Store
```

### 5.3 开发里程碑（Phase 7-10）

#### Phase 7 — 数据源扩展（1周）

- [ ] 腾讯财经 API 对接（PE/PB/市值/涨跌停价）
- [ ] 东财统一限流器 em_rate_limiter
- [ ] Python sidecar 架构搭建 + mootdx 集成
- [ ] 一致预期 EPS / 研报列表 / 概念板块 数据采集
- [ ] 解禁日历 / 股东户数 / 资金流向 数据采集

#### Phase 8 — AI 分析引擎（2周）

- [ ] LLM Router（Anthropic/OpenAI兼容/Ollama 双模）
- [ ] 7维度分析师 Prompt 工程 + 结构化输出
- [ ] PEG 估值计算引擎 + 评级体系
- [ ] 综合决策 + 5级评级输出
- [ ] 分析报告 Markdown 生成 + PDF 导出
- [ ] 盘后自动分析触发 + 异动触发

#### Phase 9 — 前端分析看板（1周）

- [ ] PEG 看板页面（表格+评级色条+消化年限）
- [ ] 分析报告页面（7维度卡片+综合评级+风险提示）
- [ ] 行业 PE 对比图（ECharts 箱线图）
- [ ] 分析历史记录页面
- [ ] 分析触发配置（手动/自动/异动）
- [ ] 自选股表新增 PEG/评级列

#### Phase 10 — 打磨 + 联调（1周）

- [ ] 分析结果 SQLite 持久化 + 迁移
- [ ] LLM 配置页面（供应商/Key/模型选择）
- [ ] 7维度数据采集容错（缺失数据标注 [数据缺失]）
- [ ] 分析性能优化（并行采集+增量缓存）
- [ ] 打包测试 + Python sidecar 分发

## 6. 决策记忆闭环（§10.1）

```rust
/// 🆕 决策记忆闭环
pub struct DecisionMemory {
    db: Arc<DbPool>,
    llm: Arc<dyn LlmClient>,
}

impl DecisionMemory {
    /// Phase A: 存储决策（分析完成时调用）
    pub async fn store_decision(
        &self,
        secid: &str,
        result: &AnalysisResult,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO decision_memory (secid, trade_date, rating, \
             peg_value, report_summary, status) \
             VALUES (?, ?, ?, ?, ?, 'pending')"
        )
        .bind(secid)
        .bind(result.analyzed_at)
        .bind(&result.overall_rating)
        .bind(&result.peg_summary.as_ref().and_then(|p| p.peg))
        .bind(&result.rating_rationale)
        .execute(&*self.db).await?;
        Ok(())
    }

    /// Phase B: 盘后复盘（T+1 检验昨天的评级是否正确）
    pub async fn reflect_on_decision(
        &self,
        secid: &str,
        trade_date: i64,
        actual_return: Decimal,     // 实际收益率
        alpha_return: Decimal,      // 超额收益（vs 沪深300）
        holding_days: u32,
    ) -> Result<String> {
        let decision = self.load_decision(secid, trade_date).await?;
        let prompt = format!(
            "你是交易分析师，回顾过去的决策。\
             实际收益率: {actual_return:+.1}%，超额收益(vs沪深300): {alpha_return:+.1}%，\
             持有天数: {holding_days}天。\
             原始决策: {rationale}\
             \
             请用2-4句话回答：\
             1. 方向判断是否正确？(引用收益率数据)\
             2. 哪个论点成立/失败了？\
             3. 一个下次类似分析应吸取的教训。",
            actual_return = actual_return,
            alpha_return = alpha_return,
            holding_days = holding_days,
            rationale = decision.rationale,
        );
        let reflection = self.llm.generate(&prompt).await?;
        self.update_with_outcome(secid, trade_date, actual_return, alpha_return, &reflection).await?;
        Ok(reflection)
    }

    /// 记忆注入：为分析 Prompt 添加历史上下文
    pub async fn get_past_context(
        &self,
        secid: &str,
        n_same: usize,    // 同股历史条数
        n_cross: usize,   // 跨股教训条数
    ) -> Result<String> {
        // 同股决策历史 + 跨股反思教训
        // 格式：[日期 | 评级 | 收益率 | 超额收益 | 持有天数] + 反思
        let same = self.load_same_ticker(secid, n_same).await?;
        let cross = self.load_cross_ticker_lessons(secid, n_cross).await?;
        Ok(format!("{same}\n\n{cross}"))
    }
}
```

**新增表**：
```sql
CREATE TABLE decision_memory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    secid TEXT NOT NULL,
    trade_date TEXT NOT NULL,
    rating TEXT NOT NULL,               -- Buy/Overweight/Hold/Underweight/Sell
    peg_value TEXT,
    report_summary TEXT NOT NULL,       -- 评级理由
    actual_return TEXT,                 -- NULL=pending, 实际收益率
    alpha_return TEXT,                  -- 超额收益(vs沪深300)
    holding_days INTEGER,
    reflection TEXT,                    -- LLM 反思文本
    status TEXT NOT NULL DEFAULT 'pending',  -- pending/resolved
    created_at INTEGER NOT NULL,
    resolved_at INTEGER,
    UNIQUE(secid, trade_date)
);
CREATE INDEX idx_decision_memory_secid ON decision_memory(secid);
CREATE INDEX idx_decision_memory_status ON decision_memory(status);
```

## 7. 数据质量门控（§10.2）

```rust
/// 🆕 数据质量门控
pub struct QualityGate {
    llm: Arc<dyn LlmClient>,
}

impl QualityGate {
    /// 硬检查（代码级，无 LLM 调用）
    fn hard_check(&self, dimension: AnalysisDimension, report: &DimensionReport) -> QualityGrade {
        // 1. 报告为空 → F
        if report.details.trim().is_empty() { return QualityGrade::F; }

        // 2. 报告过短 → D
        if report.details.len() < 200 { return QualityGrade::D; }

        // 3. 失败标记检测
        let failure_markers = ["无法获取", "I cannot retrieve", "unable to fetch", "工具调用失败"];
        let failure_count = failure_markers.iter()
            .filter(|m| report.details.contains(*m)).count();
        let stripped: String = report.details.chars()
            .filter(|c| !failure_markers.iter().any(|m| m.contains(*c)))
            .collect();
        if failure_count > 0 && stripped.trim().len() < 200 {
            return QualityGrade::D;
        }

        // 4. 数据缺失计数
        let missing = report.details.matches("[数据缺失").count();
        if missing >= 3 { return QualityGrade::C; }
        if missing > 0 { return QualityGrade::B; }

        // 5. 检查汇总表格
        let has_table = report.details.contains('|') && report.details.contains("---");
        if !has_table { return QualityGrade::B; }

        QualityGrade::A
    }

    /// 综合质量评估
    pub async fn evaluate(&self, reports: &[DimensionReport]) -> QualitySummary {
        let mut grades = HashMap::new();
        let mut fail_count = 0;
        for r in reports {
            let grade = self.hard_check(r.dimension, r);
            if matches!(grade, QualityGrade::D | QualityGrade::F) { fail_count += 1; }
            grades.insert(r.dimension, grade);
        }

        // 如果 ≤3 个维度失败，用 LLM 复审
        let llm_review = if fail_count < 4 {
            Some(self.llm_review(reports, &grades).await)
        } else {
            None  // 多数失败，跳过复审
        };

        QualitySummary {
            grades,
            llm_review,
            overall_confidence: self.calc_confidence(&grades),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum QualityGrade { A, B, C, D, F }

#[derive(Debug, Clone, Serialize)]
pub struct QualitySummary {
    pub grades: HashMap<AnalysisDimension, QualityGrade>,
    pub llm_review: Option<String>,
    pub overall_confidence: f64,  // 0.0-1.0
}
```

**质量门控注入到综合决策 Prompt**：
```rust
// 综合决策时，附加质量门控摘要
let quality_prompt = format!(
    "⚠️ 数据质量门控结果：\
     技术面: {tech_grade} | 基本面: {fund_grade} | 游资: {hot_grade} | ...\
     整体数据可信度: {confidence}\
     注意：评级为 D/F 的维度数据不可靠，决策时降低权重。",
    // ...
);
```

## 8. 分析检查点续传（§10.3）

```rust
/// 🆕 分析检查点管理器
pub struct AnalysisCheckpoint {
    db: Arc<DbPool>,
}

impl AnalysisCheckpoint {
    /// 生成确定性分析 ID（同只股票同一天 = 同一 ID）
    pub fn analysis_id(secid: &str, date: NaiveDate) -> String {
        let input = format!("{}:{}", secid, date.format("%Y-%m-%d"));
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    /// 保存当前分析进度
    pub async fn save_progress(
        &self,
        analysis_id: &str,
        completed_dimensions: &[AnalysisDimension],
        pending_dimensions: &[AnalysisDimension],
        collected_data: &serde_json::Value,  // 已采集的原始数据
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO analysis_checkpoints \
             (analysis_id, completed, pending, data, updated_at) \
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(analysis_id)
        .bind(serde_json::to_string(completed_dimensions)?)
        .bind(serde_json::to_string(pending_dimensions)?)
        .bind(serde_json::to_string(collected_data)?)
        .bind(chrono::Utc::now().timestamp())
        .execute(&*self.db).await?;
        Ok(())
    }

    /// 检查是否有可恢复的检查点
    pub async fn has_checkpoint(&self, analysis_id: &str) -> bool { /* ... */ }

    /// 从检查点恢复分析
    pub async fn resume_from_checkpoint(
        &self,
        analysis_id: &str,
    ) -> Result<Option<PartialAnalysis>> { /* ... */ }
}
```

**新增表**：
```sql
CREATE TABLE analysis_checkpoints (
    analysis_id TEXT PRIMARY KEY,
    secid TEXT NOT NULL,
    trade_date TEXT NOT NULL,
    completed TEXT NOT NULL,      -- JSON: 已完成的维度列表
    pending TEXT NOT NULL,        -- JSON: 待完成的维度列表
    data TEXT NOT NULL,           -- JSON: 已采集的原始数据
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

## 9. 结构化输出降级（§10.4）

```rust
/// 🆕 结构化输出降级策略
pub struct StructuredOutput<T: DeserializeOwned> {
    schema: Value,             // JSON Schema 定义
    render_fn: fn(&T) -> String,  // 结构化→Markdown 渲染
    parse_fn: fn(&str) -> Option<T>, // 自由文本→结构化解析
}

impl<T: DeserializeOwned> StructuredOutput<T> {
    /// 调用 LLM 并尝试结构化输出，失败则降级
    pub async fn invoke(
        &self,
        llm: &dyn LlmClient,
        prompt: &str,
        agent_name: &str,
    ) -> Result<String> {
        // 1. 尝试结构化输出
        match llm.invoke_structured(prompt, &self.schema).await {
            Ok(structured) => {
                // 渲染为 Markdown（下游统一消费格式）
                return Ok((self.render_fn)(&structured));
            }
            Err(e) => {
                tracing::warn!(
                    "{}: 结构化输出失败 ({}), 降级为自由文本",
                    agent_name, e
                );
            }
        }

        // 2. 降级：自由文本生成
        let response = llm.invoke(prompt).await?;

        // 3. 尝试从自由文本中解析结构化数据
        if let Some(parsed) = (self.parse_fn)(&response) {
            Ok((self.render_fn)(&parsed))
        } else {
            // 4. 解析也失败，返回原始自由文本（不阻塞流程）
            tracing::warn!("{}: 自由文本解析也失败，返回原始输出", agent_name);
            Ok(response)
        }
    }
}

/// 评级启发式解析器（源自 TradingAgents-astock rating.py）
/// 两遍扫描：先找 "Rating: X" 标签，再全文搜索单词
pub fn parse_rating(text: &str) -> OverallRating {
    let ratings = ["Buy", "Overweight", "Hold", "Underweight", "Sell"];
    let rating_set: HashSet<&str> = ratings.iter().copied().collect();

    // Pass 1: 找 "Rating: X" / "评级: X" 标签
    let label_re = Regex::new(r"(?i)(rating|评级)[\s:：\-]*\*{0,2}(\w+)").unwrap();
    for line in text.lines() {
        if let Some(caps) = label_re.captures(line) {
            if let Some(m) = caps.get(2) {
                let word = m.as_str().to_lowercase();
                if rating_set.contains(word.as_str()) {
                    return match word.as_str() {
                        "buy" => OverallRating::Buy,
                        "overweight" => OverallRating::Overweight,
                        "hold" => OverallRating::Hold,
                        "underweight" => OverallRating::Underweight,
                        "sell" => OverallRating::Sell,
                        _ => OverallRating::Hold,
                    };
                }
            }
        }
    }

    // Pass 2: 全文搜索单词
    for line in text.lines() {
        for word in line.split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphabetic());
            if rating_set.contains(clean.to_lowercase().as_str()) {
                return match clean.to_lowercase().as_str() {
                    "buy" => OverallRating::Buy,
                    "overweight" => OverallRating::Overweight,
                    "hold" => OverallRating::Hold,
                    "underweight" => OverallRating::Underweight,
                    "sell" => OverallRating::Sell,
                    _ => OverallRating::Hold,
                };
            }
        }
    }

    OverallRating::Hold  // 默认中性
}
```

## 10. 分析去重与幂等性（§10.6）

```rust
/// 🆕 分析去重 + 幂等性
impl AnalysisEngine {
    pub async fn trigger_analysis(&self, secid: &str, trigger: AnalysisTrigger) -> Result<AnalysisResult> {
        let today = Local::now().date_naive();
        let analysis_id = AnalysisCheckpoint::analysis_id(secid, today);

        // 1. 检查是否已在分析中
        if let Some(existing) = self.get_analysis_result(secid).await? {
            if existing.analyzed_at > today.and_hms_opt(0,0,0).unwrap().timestamp() {
                match trigger {
                    AnalysisTrigger::Manual => {
                        // 手动触发：允许重新分析（覆盖）
                        tracing::info!("手动触发重新分析: {} (上次: {})", secid, existing.overall_rating);
                    }
                    _ => {
                        // 自动触发：跳过（已分析过）
                        tracing::info!("今日已分析: {} (评级: {}), 跳过", secid, existing.overall_rating);
                        return Ok(existing);
                    }
                }
            }
        }

        // 2. 检查是否有可恢复的检查点
        if let Some(partial) = self.checkpoint.resume_from_checkpoint(&analysis_id).await? {
            tracing::info!("从检查点恢复分析: {} (已完成 {} 个维度)",
                secid, partial.completed_dimensions.len());
            return self.resume_analysis(partial).await;
        }

        // 3. 正常分析流程
        self.run_full_analysis(secid, &analysis_id, trigger).await
    }
}
```

## 11. Token预算控制（§10.7）

```rust
/// 🆕 Token 预算控制
pub struct TokenBudget {
    pub max_raw_data_chars: usize,      // 默认 30000
    pub max_dimension_report_chars: usize, // 默认 3000
    pub max_total_prompt_chars: usize,   // 默认 60000（含所有维度+系统提示）
}

impl TokenBudget {
    /// 截断原始数据
    pub fn truncate_raw_data(&self, raw: &str) -> String {
        if raw.len() > self.max_raw_data_chars {
            format!("{}\n...(数据已截断)", &raw[..self.max_raw_data_chars])
        } else {
            raw.to_string()
        }
    }

    /// 截断单维度报告（质量门控审查时使用）
    pub fn truncate_dimension_report(&self, report: &str) -> String {
        if report.len() > self.max_dimension_report_chars {
            format!("{}\n... (truncated for review)", &report[..self.max_dimension_report_chars])
        } else {
            report.to_string()
        }
    }
}
```

## 12. A股交易约束Prompt（§10.8）

```rust
/// 🆕 A股交易约束 Prompt 段（注入综合决策和每个分析师的系统提示）
pub const A_STOCK_CONSTRAINTS_PROMPT: &str = r#"
**A股交易约束**（必须在决策中考虑）：
- **T+1 结算锁**：今天买入的股票明天才能卖出。如果隔夜出现利空（政策/外盘暴跌），损失被锁定无法当日止损。这是 A 股最核心的结构性风险。
- **涨跌停陷阱**：如果股票跌停（主板-10%/科创创业-20%），卖单无法成交——你被困住了。连续跌停可能造成灾难性损失且无法退出。
- **ST/退市风险**：连续亏损被 ST 触发 ±5% 涨跌停 + 机构强制卖出，形成恶性循环。
- **游资撤退风险**：游资双向移动都快。今天的涨停明星可能是明天的跌停牺牲品。散户是最后知道游资撤退的人。
- **最小交易单位**：主板 100 股（1手），科创板/创业板 200 股。
- **交易时段**：09:30-11:30, 13:00-15:00（北京时间），午间休市无法交易。
"#;
```

## 13. 分析师必采清单（§10.9）

```rust
/// 🆕 各维度必采清单（嵌入分析师 Prompt）
pub fn get_mandatory_checklist(dimension: AnalysisDimension) -> &'static str {
    match dimension {
        AnalysisDimension::Technical => r#"
📋 必采清单 — 无法获取时标注 [数据缺失: xxx]：
1. 近 5 日成交量变化趋势（放量/缩量/平稳）
2. 当前价格与5日/10日/20日均线的关系（上方/下方/交叉）
3. MACD 金叉/死叉状态
4. 近期支撑位和压力位
5. 技术面总体判断（看多/看空/中性）"#,

        AnalysisDimension::HotMoney => r#"
📋 必采清单 — 无法获取时标注 [数据缺失: xxx]：
1. 近 5 日成交量变化趋势（放量/缩量/平稳）
2. 当日北向资金净流入金额（沪股通 + 深股通）
3. 个股主力资金净流入（超大单 + 大单）
4. 所属概念板块及当日板块涨幅
5. 当日是否上榜热门股及题材归因
6. 资金面总体判断（主力流入/主力流出/资金博弈/无明显信号）"#,

        AnalysisDimension::Lockup => r#"
📋 必采清单 — 无法获取时标注 [数据缺失: xxx]：
1. 近 6 个月内部人/大股东交易记录（增持/减持/无变动）
2. 前十大股东持股变化趋势
3. 解禁/减持相关新闻及公告
4. 减持压力评级（重大压力/中等压力/轻微压力/无明显压力）
5. 未来 3 个月潜在减持风险评估"#,

        AnalysisDimension::Peg => r#"
📋 必采清单 — 无法获取时标注 [数据缺失: xxx]：
1. 当前 PE(TTM) 与行业平均对比
2. 一致预期 EPS → 前瞻 PE
3. 近 3 年净利润 CAGR
4. PEG = 前瞻PE / (CAGR × 100)
5. PEG 评级（极度低估/低估/合理/偏贵/高估）
6. PE 消化年限"#,

        // ... 其他维度类似
        _ => "",
    }
}
```

## 14. 多空辩论机制（§12.1）

```rust
/// 🆕 多空辩论引擎
pub struct DebateEngine {
    quick_llm: Arc<dyn LlmClient>,
    deep_llm: Arc<dyn LlmClient>,
    max_debate_rounds: u32,   // 默认1（即Bull一轮+Bear一轮）
}

impl DebateEngine {
    pub async fn run_debate(
        &self,
        reports: &[DimensionReport],
        quality_summary: &QualitySummary,
    ) -> Result<DebateResult> {
        let mut history = String::new();
        let mut count = 0u32;

        // Bull 开场
        let bull_arg = self.generate_bull_argument(reports, quality_summary, &history).await?;
        history.push_str(&bull_arg);
        count += 1;

        // Bear 反驳
        let bear_arg = self.generate_bear_argument(reports, quality_summary, &history).await?;
        history.push_str(&bear_arg);
        count += 1;

        // 条件性继续辩论（如果 max_debate_rounds > 1）
        while count < 2 * self.max_debate_rounds {
            let bull_arg = self.generate_bull_argument(reports, quality_summary, &history).await?;
            history.push_str(&bull_arg);
            count += 1;
            if count >= 2 * self.max_debate_rounds { break; }

            let bear_arg = self.generate_bear_argument(reports, quality_summary, &history).await?;
            history.push_str(&bear_arg);
            count += 1;
        }

        // Research Manager 裁决（用 deep_think_llm）
        let decision = self.research_manager_judge(&history, reports).await?;

        Ok(DebateResult {
            bull_arguments: history.clone(),
            bear_arguments: history.clone(),
            judge_decision: decision,
        })
    }
}

/// A股利多框架（注入 Bull Prompt）
pub const BULL_FRAMEWORK_PROMPT: &str = r#"
A股利多框架 — 优先强调以下看多催化剂：
- **政策利好**：政府补贴/产业扶持政策（如"专精特新"/国家战略行业）/监管利好信号
- **北向资金持续净流入**：外资看多信号
- **游资接力**：连续涨停+量价配合+题材归因明确+板块轮动刚开始
- **PEG低估**：前瞻PE/PEG/PE消化年限论证当前估值有增长支撑
- **解禁压力清除**：主要解禁期已过/大股东未减持
"#;

/// A股利空框架（注入 Bear Prompt）
pub const BEAR_FRAMEWORK_PROMPT: &str = r#"
A股利空框架 — 优先强调以下看空风险：
- **政策打压**：突然的行业整顿/反垄断/证监会窗口指导/行业性交易限制
- **解禁减持**：即将到来的大规模解禁/控股股东预披露减持/股权质押平仓风险
- **游资撤退**：涨停后放量滞涨/连板断裂/板块轮动转离此题材
- **估值泡沫**：PE远超30x增长锚且EPS无法在3年内消化/PEG>2/散户投机溢价
- **T+1陷阱**：大涨后买入者次日才能卖出——隔夜情绪逆转/跳空低开损失被锁定
- **北向撤退**：沪深股通净流出信号外资减仓
"#;
```

## 15. 分析维度可配置选股（§12.5）

```rust
/// 🆕 分析维度选择器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisProfile {
    pub name: String,
    pub description: String,
    pub dimensions: Vec<AnalysisDimension>,
}

impl AnalysisProfile {
    /// 预设：短线快评（3维度，~15秒）
    pub fn short_term() -> Self {
        Self {
            name: "短线快评".into(),
            description: "技术面+游资+资金流，适合短线交易参考".into(),
            dimensions: vec![AnalysisDimension::Technical, AnalysisDimension::HotMoney, AnalysisDimension::Peg],
        }
    }

    /// 预设：基本面深研（4维度，~30秒）
    pub fn fundamental_deep() -> Self {
        Self {
            name: "基本面深研".into(),
            description: "基本面+PEG+政策+解禁，适合中长线投资研究".into(),
            dimensions: vec![
                AnalysisDimension::Fundamentals,
                AnalysisDimension::Peg,
                AnalysisDimension::Policy,
                AnalysisDimension::Lockup,
            ],
        }
    }

    /// 预设：全面分析（7维度，~60秒）
    pub fn full() -> Self {
        Self {
            name: "全面分析".into(),
            description: "7维度全面分析+多空辩论+综合评级".into(),
            dimensions: vec![
                AnalysisDimension::Technical,
                AnalysisDimension::News,
                AnalysisDimension::Fundamentals,
                AnalysisDimension::Policy,
                AnalysisDimension::HotMoney,
                AnalysisDimension::Lockup,
                AnalysisDimension::Peg,
            ],
        }
    }

    /// 预设：异动快查（2维度，~10秒，异动触发时自动使用）
    pub fn anomaly_quick() -> Self {
        Self {
            name: "异动快查".into(),
            description: "技术面+游资，异动触发时快速响应".into(),
            dimensions: vec![AnalysisDimension::Technical, AnalysisDimension::HotMoney],
        }
    }
}
```

## 16. 分析流程消息清理（§12.6）

```rust
/// 🆕 分析流程消息清理
impl AnalysisEngine {
    pub async fn run_analysis(&self, secid: &str, profile: &AnalysisProfile) -> Result<AnalysisResult> {
        let mut dimension_reports = Vec::new();

        for dimension in &profile.dimensions {
            // 1. 采集数据（Python sidecar）
            let raw_data = self.data_router.fetch(&dimension_to_capability(dimension), &params).await?;

            // 2. 单维度分析（不传入其他维度的对话历史）
            let report = self.analyze_dimension(dimension, &raw_data).await?;

            // 3. 只保留结构化结果，不保留中间对话
            dimension_reports.push(report);

            // 4. 保存检查点
            self.checkpoint.save_progress(&analysis_id, &dimension_reports, &pending).await?;
        }

        // 5. 质量门控（基于结构化报告，不含对话历史）
        let quality = self.quality_gate.evaluate(&dimension_reports).await?;

        // 6. 多空辩论 + 综合决策（只看结构化报告+质量摘要）
        let debate = self.debate_engine.run_debate(&dimension_reports, &quality).await?;
        let final_result = self.final_decision(&dimension_reports, &quality, &debate).await?;

        Ok(final_result)
    }
}
```

## 17. 输出语言策略（§12.7）

```rust
/// 🆕 输出语言策略
pub fn get_language_instruction(output_lang: &str) -> String {
    match output_lang {
        "Chinese" => "\n\n请用中文输出最终报告。内部推理可以使用英文，但面向用户的结论和表格必须用中文。",
        "English" => "\n\nPlease output the final report in English.",
        _ => String::new(),
    }
}
```

## 18. LLM供应商兼容性（§14.1）

| 供应商 | 坑点 | 处理方式 |
|--------|------|---------|
| **DeepSeek** | thinking模式返回`reasoning_content`，**下次调用必须原样回传**，否则 HTTP 400 | `_get_request_payload` 回传 + `_create_chat_result` 捕获 |
| **DeepSeek Reasoner** | 不支持 `tool_choice`，结构化输出不可用 | `with_structured_output` 抛 `NotImplementedError`，触发 `structured.py` 降级 |
| **Anthropic** | extended thinking / tool use 返回 content 为 typed blocks 列表，不是纯 string | `normalize_content()` 统一转 string |
| **OpenAI Responses API** | 返回 content 为 list of typed blocks (reasoning, text...) | `NormalizedChatOpenAI.invoke()` 归一化 |
| **OpenAI** | `with_structured_output` 默认走 Responses API parse path，产生大量 Pydantic 序列化警告 | 强制 `method="function_calling"` |
| **Ollama / OpenRouter** | 任意模型名都接受，无法预验证 | `validate_model` 对这两个供应商直接返回 true |

```rust
/// 🆕 LLM 供应商特定适配器
pub enum LlmAdapter {
    Anthropic {
        // Claude: content 可能是 typed blocks，需归一化
        // extended thinking: reasoning_content 需回传
    },
    OpenAiCompatible {
        provider: String,  // "openai", "deepseek", "qwen", "glm", "minimax"
        // DeepSeek: reasoning_content 回传
        // DeepSeek Reasoner: 无 tool_choice，降级为自由文本
        // OpenAI: with_structured_output 用 function_calling 模式
    },
    Ollama {
        // 本地模型：任意模型名，结构化输出可能不支持
    },
}

impl LlmAdapter {
    /// 发送请求前的适配处理
    pub fn adapt_request(&self, messages: &mut Vec<LlmMessage>) {
        match self {
            Self::OpenAiCompatible { provider, .. } if provider == "deepseek" => {
                // DeepSeek: 回传上一轮的 reasoning_content
                for msg in messages.iter_mut().rev() {
                    if msg.role == "assistant" {
                        if let Some(reasoning) = msg.additional_kwargs.get("reasoning_content") {
                            // 确保下次请求携带此字段
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// 收到响应后的适配处理
    pub fn adapt_response(&self, response: &mut LlmResponse) {
        // 所有供应商: 归一化 content（typed blocks → string）
        response.normalize_content();

        // DeepSeek: 捕获 reasoning_content 供下次回传
        if let Self::OpenAiCompatible { provider, .. } = self {
            if provider == "deepseek" {
                response.capture_reasoning_content();
            }
        }
    }

    /// 检查是否支持结构化输出
    pub fn supports_structured_output(&self, model: &str) -> bool {
        match self {
            Self::OpenAiCompatible { provider, .. } => {
                // DeepSeek Reasoner 不支持
                if provider == "deepseek" && model.contains("reasoner") {
                    return false;
                }
                true
            }
            Self::Ollama => false,  // 本地模型通常不支持
            _ => true,
        }
    }
}
```

## 19. 双LLM模型推荐（§14.2）

```rust
/// 🆕 双LLM模型推荐组合
pub struct ModelRecommendation {
    pub provider: String,
    pub quick_model: String,    // 数据采集/简单判断（速度快/成本低）
    pub deep_model: String,     // 综合决策/估值分析（质量高/推理深）
    pub estimated_cost_per_analysis: String,  // 单次分析预估成本
}

impl ModelRecommendation {
    pub fn recommendations() -> Vec<Self> {
        vec![
            // ── 国内直连（推荐）────────────────────────
            Self {
                provider: "deepseek".into(),
                quick_model: "deepseek-v4-flash".into(),
                deep_model: "deepseek-v4-pro".into(),
                estimated_cost_per_analysis: "¥0.02-0.05".into(),
            },
            Self {
                provider: "qwen".into(),
                quick_model: "qwen3.5-flash".into(),
                deep_model: "qwen3.6-plus".into(),
                estimated_cost_per_analysis: "¥0.03-0.08".into(),
            },
            Self {
                provider: "glm".into(),
                quick_model: "glm-4.7".into(),
                deep_model: "glm-5.1".into(),
                estimated_cost_per_analysis: "¥0.02-0.06".into(),
            },
            Self {
                provider: "minimax".into(),
                quick_model: "MiniMax-M2.7-highspeed".into(),
                deep_model: "MiniMax-M2.7".into(),
                estimated_cost_per_analysis: "¥0.01-0.03".into(),
            },
            // ── 海外（需网络）──────────────────────────
            Self {
                provider: "anthropic".into(),
                quick_model: "claude-sonnet-4-6".into(),
                deep_model: "claude-opus-4-6".into(),
                estimated_cost_per_analysis: "$0.15-0.40".into(),
            },
            Self {
                provider: "openai".into(),
                quick_model: "gpt-5.4-mini".into(),
                deep_model: "gpt-5.4".into(),
                estimated_cost_per_analysis: "$0.10-0.30".into(),
            },
            // ── 本地（隐私优先）────────────────────────
            Self {
                provider: "ollama".into(),
                quick_model: "qwen3:latest".into(),
                deep_model: "glm-4.7-flash:latest".into(),
                estimated_cost_per_analysis: "免费(本地GPU)".into(),
            },
        ]
    }
}
```

## 20. 分析进度追踪（§14.3）

```rust
/// 🆕 分析进度追踪器
pub struct AnalysisProgressTracker {
    stages: Vec<StageInfo>,
    current_stage: Option<String>,
    start_time: Instant,
    stats: AnalysisStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageInfo {
    pub id: String,
    pub name: String,
    pub icon: String,       // emoji
    pub status: StageStatus,
    pub report: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum StageStatus { Pending, Active, Done, Skipped, Failed }

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisStats {
    pub llm_calls: u32,
    pub tool_calls: u32,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub elapsed_secs: f64,
}

impl AnalysisProgressTracker {
    /// v3.0 完整阶段列表（12阶段，对应 TradingAgents 的12阶段）
    pub fn default_stages() -> Vec<StageInfo> {
        vec![
            StageInfo { id: "technical".into(),    name: "技术分析".into(),   icon: "📊".into(), status: StageStatus::Pending, report: None },
            StageInfo { id: "news".into(),         name: "新闻舆情".into(),   icon: "📰".into(), status: StageStatus::Pending, report: None },
            StageInfo { id: "fundamentals".into(),  name: "基本面".into(),     icon: "📋".into(), status: StageStatus::Pending, report: None },
            StageInfo { id: "policy".into(),        name: "政策分析".into(),   icon: "🏛️".into(), status: StageStatus::Pending, report: None },
            StageInfo { id: "hot_money".into(),     name: "游资追踪".into(),   icon: "🔥".into(), status: StageStatus::Pending, report: None },
            StageInfo { id: "lockup".into(),        name: "解禁监控".into(),   icon: "🔒".into(), status: StageStatus::Pending, report: None },
            StageInfo { id: "peg".into(),           name: "PEG估值".into(),    icon: "📈".into(), status: StageStatus::Pending, report: None },
            StageInfo { id: "quality_gate".into(),  name: "质量门控".into(),   icon: "✅".into(), status: StageStatus::Pending, report: None },
            StageInfo { id: "debate".into(),        name: "多空辩论".into(),   icon: "⚔️".into(), status: StageStatus::Pending, report: None },
            StageInfo { id: "trader".into(),        name: "交易决策".into(),   icon: "💹".into(), status: StageStatus::Pending, report: None },
            StageInfo { id: "risk".into(),          name: "风控评估".into(),   icon: "🛡️".into(), status: StageStatus::Pending, report: None },
            StageInfo { id: "pm".into(),            name: "最终决策".into(),   icon: "👔".into(), status: StageStatus::Pending, report: None },
        ]
    }

    /// 阶段完成时：更新状态 + emit 前端事件
    pub fn mark_stage_done(&mut self, stage_id: &str, app: &AppHandle) {
        if let Some(stage) = self.stages.iter_mut().find(|s| s.id == stage_id) {
            stage.status = StageStatus::Done;
        }
        // 推送进度到前端
        app.emit("analysis-progress", &self.current_progress()).ok();
    }

    fn current_progress(&self) -> serde_json::Value {
        serde_json::json!({
            "stages": self.stages,
            "current_stage": self.current_stage,
            "stats": self.stats,
            "elapsed_secs": self.start_time.elapsed().as_secs_f64(),
        })
    }
}
```

**前端进度面板**：
```
┌─────────────────────────────────────────────────────────┐
│ AI 分析进度 — 贵州茅台(600519)                    42s   │
│                                                         │
│  ✅ 📊 技术分析    3s                                   │
│  ✅ 📰 新闻舆情    5s                                   │
│  ✅ 📋 基本面      8s                                   │
│  ✅ 🏛️ 政策分析    4s                                   │
│  ⏳ 🔥 游资追踪    进行中...                            │
│  ⬜ 🔒 解禁监控                                         │
│  ⬜ 📈 PEG估值                                          │
│  ⬜ ✅ 质量门控                                         │
│  ⬜ ⚔️ 多空辩论                                         │
│  ⬜ 💹 交易决策                                         │
│  ⬜ 🛡️ 风控评估                                         │
│  ⬜ 👔 最终决策                                         │
│                                                         │
│  LLM调用: 12次  工具调用: 8次  Token: 15.2k入/4.8k出   │
└─────────────────────────────────────────────────────────┘
```

## 21. A股特化Prompt设计（§14.4）

| 维度 | A股特化Prompt要点 | 为什么重要 |
|------|------------------|-----------|
| **技术面** | ①涨跌停制度→触及后技术指标失真 ②T+1→短线策略可执行性受限 ③北向资金→领先趋势转折 ④换手率→散户占比高是核心指标 ⑤量价关系→"量在价先"规律显著 | 不加这些，LLM会用美股技术分析逻辑分析A股，结论偏误 |
| **情绪面** | ①散户占比>60%→情绪影响远大于成熟市场 ②舆论阵地→东财股吧/雪球/同花顺社区 ③反向指标→散户一致看多是阶段顶部 ④时间维度→区分短期波动(1-3天)和中期趋势(1-4周) | A股是情绪市，不分析情绪等于瞎子摸象 |
| **基本面** | ①中国会计准则CAS vs IFRS差异 ②A股PE中位数30-50x为常态，不能照搬美股15-25x ③财报披露节奏(一季报4月/半年报8月/三季报10月/年报次年4月) ④特殊风险→商誉减值/股权质押/关联交易 | 不加这些，LLM会用美股估值标准判断A股 |
| **政策面** | ①政策力度分级→指导意见(弱)<部委通知(中)<国务院文件(强)<法律法规(最强) ②影响时间窗口→短期脉冲(1-2周)/中期趋势(1-3月)/长期结构性(半年+) ③受益/受损逻辑链→政策→行业→公司业务→财务影响 | A股是政策市，政策分析权重应最高 |
| **游资追踪** | ①量价异动→放量(>20日均量2倍)/换手率>10%异常 ②龙虎榜信号→知名游资席位买入是强势信号 ③连板分析→首板放量vs缩量含义不同/三板以上进"妖股"模式 ④板块资金轮动→资金从一个板块撤出流入另一个 | 游资是A股短线定价核心力量 |
| **解禁监控** | ①限售股类型→首发原股东/定增/股权激励/战略配售 ②解禁规模→占流通市值>20%为重大压力 ③减持新规→大股东每90天集中竞价≤1%/大宗交易≤2% ④减持动力→当前股价vs解禁成本溢价倍数 | 解禁是A股特有供给冲击 |

## 22. 三方风险辩论（§14.5）

| 角色 | A股特化视角 | 核心论点 |
|------|-----------|---------|
| **激进派** | ①涨停板效应→T+1反而帮助多日连板 ②政策底→政府扶持的板块有"政策底" ③游资共识→知名席位+题材归因=短期爆发力 ④北向验证→内外资共振是强信号 ⑤PE扩张期→A股牛市题材龙头PE可到50-100x ⑥散户顺风→80%散户，情绪转正时羊群效应放大收益 | "错过也是风险" |
| **保守派** | ①T+1锁定→隔夜跳空无法止损 ②跌停陷阱→卖不出，连续跌停灾难性 ③ST/退市→±5%+机构强制卖出恶性循环 ④游资撤退→今天涨停明天跌停 ⑤估值纪律→PE>50x+PEG>2是投机区 ⑥政策反复→政府给的可一夜收回 | "保护本金第一" |
| **中立派** | ①T+1双刃剑→锁死亏损但也防恐慌卖出 ②政策分级→国务院指令(高)>地方激励(低)>市场传闻(噪音) ③北向聪明钱→确认信号非主信号 ④估值区间法→不是"PE>30就贵"而是"什么PE区间有增长支撑" ⑤解禁时机→不恐慌但逐步减仓 ⑥板块轮动→2-4周周期，判断在轮动哪个阶段 ⑦仓位管理>方向判断→±10-20%日限制下仓位比方向重要 | "适度仓位捕捉上行" |

**中立派的"仓位管理>方向判断"** 是 A 股最重要的实践智慧——在 ±10-20% 日限制 + T+1 的市场结构下，**仓位大小比看多看空更重要**。

## 23. 原子文件写入（§14.6）

```rust
/// 🆕 原子文件写入
pub fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;  // rename 在 POSIX 上是原子操作
    Ok(())
}
```

## 24. 分析成本预估（§14.7）

```rust
/// 🆕 分析成本预估
pub struct CostEstimator;

impl CostEstimator {
    /// 各供应商每百万 token 价格（美元）
    pub fn price_per_million_tokens(provider: &str, model: &str) -> (f64, f64) {
        // 返回 (input_price, output_price) 每100万token
        match provider {
            "deepseek" => (0.14, 0.28),       // DeepSeek V4
            "qwen" => (0.30, 0.60),           // Qwen Plus
            "anthropic" if model.contains("opus") => (15.0, 75.0),
            "anthropic" if model.contains("sonnet") => (3.0, 15.0),
            "anthropic" if model.contains("haiku") => (0.80, 4.0),
            "openai" if model.contains("gpt-5.4") => (10.0, 30.0),
            "openai" if model.contains("mini") => (0.15, 0.60),
            "ollama" => (0.0, 0.0),           // 本地免费
            _ => (1.0, 3.0),                  // 未知供应商默认
        }
    }

    /// 计算当前分析成本
    pub fn estimate_cost(
        provider: &str,
        model: &str,
        tokens_in: u64,
        tokens_out: u64,
    ) -> f64 {
        let (in_price, out_price) = Self::price_per_million_tokens(provider, model);
        (tokens_in as f64 / 1_000_000.0 * in_price) + (tokens_out as f64 / 1_000_000.0 * out_price)
    }
}
```

## 25. 遗漏项对照汇总

### 附录E.1 架构级新增（v3.0 草案原有）

| 新增项 | 来源 | 优先级 | 说明 |
|--------|------|--------|------|
| AI 分析引擎 | TradingAgents-astock | 🔴 P0 | 核心新增，从监控→分析 |
| PEG 估值看板 | astock-peg | 🔴 P0 | 估值辅助决策核心指标 |
| 5级结构化评级 | TradingAgents-astock | 🔴 P0 | Buy/Overweight/Hold/Underweight/Sell |
| 腾讯财经数据源 | astock-peg / a-stock-data | 🟡 P1 | 不封IP的PE/PB/市值来源 |
| 东财统一限流 | a-stock-data | 🟡 P1 | 防封核心策略 |
| 概念板块归属 | a-stock-data | 🟡 P1 | 游资分析+板块轮动 |
| 解禁日历 | TradingAgents-astock | 🟡 P1 | A股特有供给冲击 |
| 股东户数+筹码 | a-stock-data | 🟡 P1 | 筹码集中度判断 |
| 研报+一致预期 | astock-peg | 🟡 P1 | 前瞻PE计算基础 |
| 行业PE对比 | astock-peg | 🟢 P2 | 横向估值参考 |
| Python sidecar | a-stock-data / astock-peg | 🟡 P1 | 复用成熟数据采集代码 |
| 盘后自动分析 | TradingAgents-astock | 🟢 P2 | 15:30自动触发 |
| 分析历史+PDF | astock-peg | 🟢 P2 | 报告归档 |
| 游资追踪维度 | TradingAgents-astock | 🟡 P1 | 龙虎榜+资金流+板块轮动 |
| 政策面维度 | TradingAgents-astock | 🟡 P1 | A股政策市特色 |
| 解禁监控维度 | TradingAgents-astock | 🟡 P1 | 减持压力评估 |

### 附录E.2 深度审查后补充的遗漏项（§10.x）

| 遗漏项 | 来源 | 优先级 | 影响 | 章节 |
|--------|------|--------|------|------|
| 决策反思与记忆闭环 | TradingAgents reflection+memory | 🔴 P0 | AI无法学习，反复犯同样错误 | §10.1 |
| 数据质量门控 | TradingAgents quality_gate | 🔴 P0 | 可能基于空/失败数据做判断 | §10.2 |
| 分析检查点与断点续传 | TradingAgents checkpointer | 🟡 P1 | 崩溃丢失所有分析进度 | §10.3 |
| 结构化输出降级策略 | TradingAgents structured+rating | 🟡 P1 | 弱模型/Ollama无法工作 | §10.4 |
| Ticker格式归一化 | a-stock-data + astock-peg | 🔴 P0 | 多数据源格式不统一到处报错 | §10.5 |
| 分析去重与幂等性 | astock-peg route.ts | 🟡 P1 | 重复点击浪费token+时间 | §10.6 |
| 原始数据截断与Token控制 | astock-peg analysis.ts | 🟡 P1 | 原始数据可能超token限制 | §10.7 |
| A股交易约束嵌入Prompt | TradingAgents portfolio_manager | 🔴 P0 | LLM可能给T+0等错误建议 | §10.8 |
| 分析师必采清单 | TradingAgents 各分析师📋 | 🟡 P1 | LLM可能跳过关键数据直接结论 | §10.9 |

### §13 第二轮遗漏项对照（补充 §12.x）

| 遗漏项 | 来源 | 优先级 | 影响 | 章节 |
|--------|------|--------|------|------|
| 多空辩论机制 | TradingAgents setup.py + bull/bear/research_manager | 🔴 P0 | 单一综合决策偏向乐观，缺乏对抗性验证 | §12.1 |
| 数据供应商路由与降级链 | TradingAgents interface.py | 🟡 P1 | 单数据源挂了全挂，无降级 | §12.2 |
| 数据层12个已知坑点 | a-stock-data SKILL.md 3版迭代踩坑 | 🔴 P0 | 不提前知道会反复踩坑浪费数天 | §12.3 |
| 北向数据断供+本地自缓存 | a-stock-data hsgt_realtime | 🔴 P0 | 东财北向数据2024-08后断供，面板永远显示0 | §12.4 |
| 分析维度可配置（分析预设） | TradingAgents selected_analysts | 🟡 P1 | 全跑7维度浪费token，短线只需3个 | §12.5 |
| 分析流程消息清理 | TradingAgents create_msg_delete | 🟡 P1 | 中间对话历史无限膨胀，超token限制 | §12.6 |
| 输出语言策略 | TradingAgents get_language_instruction | 🟢 P2 | 中文推理质量低，需内部英文+输出中文 | §12.7 |
| 字段映射集中管理 | a-stock-data SKILL.md 字段映射表 | 🟡 P1 | 字段映射散落各处，新增数据源易遗漏 | §12.8 |
| ST股龙虎榜特殊触发规则 | a-stock-data SKILL.md | 🟢 P2 | ST股上榜频率高是正常现象，非异常 | §12.9 |

### §15 第三轮遗漏项对照（补充 §14.x）

| 遗漏项 | 来源 | 优先级 | 影响 | 章节 |
|--------|------|--------|------|------|
| LLM供应商特定兼容性坑点 | TradingAgents llm_clients/ 全套 | 🔴 P0 | DeepSeek reasoning_content不回传→400, Reasoner无tool_choice→崩溃 | §14.1 |
| 双LLM模型推荐组合 | TradingAgents model_catalog.py | 🟡 P1 | 用户不知道选什么模型配什么模型 | §14.2 |
| 分析进度实时追踪与可视化 | TradingAgents progress.py + runner.py | 🔴 P0 | 1-3分钟分析无进度→用户焦虑→放弃使用 | §14.3 |
| A股特化Prompt精妙设计 | TradingAgents 各分析师system_message | 🔴 P0 | 不加A股框架→LLM用美股逻辑分析A股→结论偏误 | §14.4 |
| 三方风险辩论A股特化视角 | TradingAgents 三个Debator | 🟡 P1 | 只有Bull/Bear缺Neutral→缺少"仓位管理>方向判断"智慧 | §14.5 |
| 原子文件写入 | astock-peg writePortfolio | 🟢 P2 | 写入中途崩溃→文件损坏 | §14.6 |
| 分析成本预估与Token预算 | TradingAgents StatsCallbackHandler | 🟡 P1 | 用户不知道一次分析花多少钱 | §14.7 |

## 26. 测试要点

| 测试类别 | 测试项 | 优先级 |
|----------|--------|--------|
| PEG计算 | `PegCalculator::rate` 五级边界值（0.49/0.5/0.99/1.0/1.49/1.5/1.99/2.0） | P0 |
| PEG计算 | `pe_digestion_years` 边界（forward_pe=0, cagr=0, forward_pe=30） | P0 |
| PEG计算 | `calc_cagr` 边界（beginning=0, years=0, 正常3年） | P0 |
| 结构化输出降级 | `parse_rating` 两遍扫描（含 Rating: Buy 标签 / 无标签全文搜索 / 无匹配默认 Hold） | P0 |
| 结构化输出降级 | `StructuredOutput::invoke` 三步降级（结构化成功→自由文本解析→原始返回） | P0 |
| 质量门控 | `hard_check` 五层规则（空→F, 短→D, 失败标记→D, 缺失≥3→C, 缺失>0→B, 无表→B, A） | P0 |
| 分析去重 | 手动触发允许覆盖，自动触发跳过已有结果 | P1 |
| 检查点续传 | 崩溃后恢复，从已完成维度继续 | P1 |
| Token预算 | `truncate_raw_data` 30000字符截断 / `truncate_dimension_report` 3000字符截断 | P1 |
| 多空辩论 | `run_debate` 条件循环（max_debate_rounds=1 → Bull+Bear各1轮） | P1 |
| 多空辩论 | `run_debate` 条件循环（max_debate_rounds=2 → Bull+Bear各2轮） | P1 |
| LLM适配 | `LlmAdapter::supports_structured_output` DeepSeek Reasoner返回false, Ollama返回false | P1 |
| LLM适配 | `LlmAdapter::adapt_request` DeepSeek reasoning_content回传 | P1 |
| LLM适配 | `LlmAdapter::adapt_response` content归一化 + reasoning_content捕获 | P1 |
| 进度追踪 | `mark_stage_done` 正确更新状态 + emit事件 | P1 |
| 成本预估 | `CostEstimator::estimate_cost` 各供应商价格计算 | P2 |
| 原子写入 | `atomic_write` 中途崩溃不损坏目标文件 | P2 |
| 语言策略 | `get_language_instruction` Chinese/English/default | P2 |
| 分析预设 | `AnalysisProfile` 四种预设维度配置正确 | P1 |

## 27. 文件清单

| 文件路径 | 职责 |
|----------|------|
| `src-tauri/src/analysis/mod.rs` | AnalysisEngine 主入口 |
| `src-tauri/src/analysis/engine.rs` | 分析流程编排（run_analysis, trigger_analysis） |
| `src-tauri/src/analysis/dimensions/mod.rs` | 维度分派 |
| `src-tauri/src/analysis/dimensions/technical.rs` | 技术面分析 |
| `src-tauri/src/analysis/dimensions/news.rs` | 新闻面分析 |
| `src-tauri/src/analysis/dimensions/fundamentals.rs` | 基本面分析 |
| `src-tauri/src/analysis/dimensions/policy.rs` | 政策面分析 |
| `src-tauri/src/analysis/dimensions/hot_money.rs` | 游资追踪 |
| `src-tauri/src/analysis/dimensions/lockup.rs` | 解禁监控 |
| `src-tauri/src/analysis/dimensions/peg.rs` | PEG估值分析 |
| `src-tauri/src/analysis/llm/mod.rs` | LlmRouter + LlmAdapter |
| `src-tauri/src/analysis/llm/anthropic.rs` | Claude API 适配 |
| `src-tauri/src/analysis/llm/openai_compat.rs` | OpenAI/DeepSeek/Qwen/GLM 适配 |
| `src-tauri/src/analysis/llm/ollama.rs` | 本地 LLM 适配 |
| `src-tauri/src/analysis/report.rs` | 报告生成+渲染 |
| `src-tauri/src/analysis/schemas.rs` | 结构化输出 Schema + StructuredOutput<T> |
| `src-tauri/src/analysis/auto_trigger.rs` | 盘后/异动自动触发 |
| `src-tauri/src/analysis/decision_memory.rs` | 决策记忆闭环（store_decision/reflect_on_decision/get_past_context） |
| `src-tauri/src/analysis/quality_gate.rs` | 数据质量门控（hard_check 5层/evaluate/QualityGrade A-F） |
| `src-tauri/src/analysis/checkpoint.rs` | 分析检查点续传（analysis_id/save_progress/has_checkpoint/resume_from_checkpoint） |
| `src-tauri/src/analysis/debate.rs` | 多空辩论引擎（run_debate/Bull/Bear框架Prompt） |
| `src-tauri/src/analysis/token_budget.rs` | Token预算控制（truncate_raw_data/truncate_dimension_report） |
| `src-tauri/src/analysis/cost_estimator.rs` | 分析成本预估（price_per_million_tokens/estimate_cost） |
| `src-tauri/src/analysis/progress.rs` | 分析进度追踪（12阶段/StageInfo/StageStatus/mark_stage_done+emit） |
| `src-tauri/src/analysis/profiles.rs` | 分析预设配置（AnalysisProfile 4种预设） |
| `src-tauri/src/analysis/prompts.rs` | Prompt常量（A_STOCK_CONSTRAINTS_PROMPT/BULL_FRAMEWORK_PROMPT/BEAR_FRAMEWORK_PROMPT/DRAGON_TIGER_RULES/get_mandatory_checklist/get_language_instruction） |
| `src-tauri/src/analysis/atomic_write.rs` | 原子文件写入（tmp+rename POSIX） |
| `src-tauri/src/market/sources/tencent.rs` | 腾讯财经数据源 |
| `src-tauri/src/market/sources/mootdx.rs` | mootdx bridge（via sidecar） |
| `src-tauri/src/market/sources/em_rate_limiter.rs` | 东财统一限流器 |
| `src-tauri/src/sidecar/mod.rs` | SidecarManager |
| `src-tauri/src/sidecar/protocol.rs` | JSON-RPC 协议 |
| `src-tauri/src/db/analysis.rs` | 分析结果 CRUD |
| `src-tauri/src/db/peg_cache.rs` | PEG缓存 |
| `src-tauri/src/db/lockup_cache.rs` | 解禁缓存 |
| `sidecar/main.py` | Python 数据采集入口 |
| `sidecar/datafeeds.py` | 数据源封装（复用 a-stock-data） |
| `sidecar/collect_stock_data.py` | 个股全量采集（复用 astock-peg） |
| `sidecar/detect_sector.py` | 行业检测（复用 astock-peg） |
| `sidecar/requirements.txt` | mootdx, requests, pandas, stockstats |
| `src/views/AnalysisDashboard.vue` | AI分析主页 |
| `src/views/AnalysisHistory.vue` | 历史记录 |
| `src/components/analysis/DimensionCard.vue` | 单维度分析卡片 |
| `src/components/analysis/OverallRatingBar.vue` | 综合评级条 |
| `src/components/analysis/PegRatingTag.vue` | PEG评级标签 |
| `src/components/analysis/AnalysisTriggerConfig.vue` | 分析触发配置 |
| `src/components/charts/IndustryPegChart.vue` | 行业PE对比 |
| `src/components/charts/PegTrendChart.vue` | PEG趋势 |
| `src/stores/analysis.ts` | 分析 Store |
