# TickPulse - 快速上手文档

> A股桌面行情监控应用，基于 Tauri 2 + Vue 3 构建

---

## 一、项目概述

**TickPulse** 是一款专为A股投资者设计的桌面行情监控应用，提供实时行情、持仓管理、异动检测、AI分析等功能。

### 核心特性

| 功能模块 | 描述 |
|---------|------|
| 📊 **实时行情** | 自选股实时行情刷新，支持涨跌停判断、封板强度监控 |
| 📈 **持仓管理** | 持仓盈亏计算、组合收益汇总、基准对比 |
| 🔔 **预警系统** | 价格突破、涨跌幅、量比等多维度预警规则 |
| 🤖 **AI分析引擎** | 7维度分析 + PEG估值 + 多空辩论 + 质量门控 |
| 📉 **K线图表** | 分时走势、日K、周K、月K + MACD/BOLL指标 |
| 💹 **市场数据** | 北向资金、大宗交易、龙虎榜、涨跌停统计 |
| ⚡ **系统功能** | 系统托盘、开机自启、全局快捷键、悬浮窗 |

---

## 二、技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                      TickPulse 应用架构                      │
├─────────────────────────────────────────────────────────────┤
│  前端 (Vue 3 + TypeScript)                                  │
│  ├── 视图层: 12 个路由视图 (Home/Position/AnalysisDashboard/...) │
│  ├── 组件层: 50+ 个 Vue 组件 (Charts/Stock/Layout/Analysis/...) │
│  ├── 状态管理: 7 个 Pinia Stores (market/alert/anomaly/analysis/...) │
│  └── IPC 封装: src/lib/tauri.ts (50+ API 函数)                 │
├─────────────────────────────────────────────────────────────┤
│  Tauri Bridge (IPC)                                         │
│  └── invoke() / listen() 双向通信                           │
├─────────────────────────────────────────────────────────────┤
│  后端 (Rust)                                                │
│  ├── 行情模块: market/ (14阶段调度、6数据源容灾)           │
│  ├── 分析引擎: analysis/ (7维度分析、LLM集成)               │
│  ├── 数据库: db/ (SQLite + WAL模式、16张表)                 │
│  ├── 预警系统: alert/ (规则引擎、触发检测)                  │
│  ├── 异动检测: anomaly/ (快速拉升/下跌/量比突增)           │
│  ├── 涨跌停: limit/ (6板块差异化、向上向下取整)             │
│  └── 系统模块: system/ (托盘、快捷键、自启)                 │
├─────────────────────────────────────────────────────────────┤
│  Python Sidecar (可选)                                      │
│  └── 16个数据抓取 handler + stdin/stdout JSON IPC           │
└─────────────────────────────────────────────────────────────┘

---

## 三、关键特性详解

### 3.1 14阶段精细化调度

行情调度器根据交易时段自动调整刷新策略：

| 阶段 | 时间范围 | 刷新间隔 | 行为 |
|-----|---------|---------|------|
| PreMarket | 09:15 前 | 60s | 盘前准备 |
| CallAuction | 09:15-09:25 | 3s | 集合竞价 |
| PreOpen | 09:25-09:30 | 5s | 静默期 |
| MorningOpen | 09:30-09:35 | 3s | 开盘波动 |
| MorningActive | 09:35-10:00 | 5s | 早盘活跃 |
| MorningStable | 10:00-11:30 | 10s | 上午稳定期 |
| LunchBreak | 11:30-13:00 | 60s | 午休 |
| AfternoonOpen | 13:00-13:05 | 3s | 下午开盘 |
| AfternoonActive | 13:05-14:00 | 10s | 下午活跃 |
| AfternoonStable | 14:00-14:30 | 10s | 下午稳定 |
| LateTrading | 14:30-14:45 | 5s | 尾盘交易 |
| CallAuctionClose | 14:45-15:00 | 3s | 收盘集合竞价 |
| AfterHours | 15:00-15:30 | 30s | 盘后 |
| Holiday | 非交易日 | 300s | 节假日 |

### 3.2 量比衰减提示

开盘 30 分钟内 (09:30-10:00)，量比数值偏高，参考价值有限。系统自动标记 `VolumeRatioNote.Early`：

```rust
// 开盘30分钟内标记量比衰减
let is_early_market = minutes_since_open < 30;
if is_early_market && quote.volume_ratio > 0.0 {
    quote.volume_ratio_note = Some(VolumeRatioNote::Early);
}


### 3.3 封板强度计算

封板强度 = 有效封板时间 / 总交易时间 (排除午休)，炸板每次惩罚 5%：

```rust
/// 计算有效交易分钟数（排除午休11:30-13:00）
fn calc_seal_strength(first_seal_time, current_time, break_count) -> f64 {
    let minutes_sealed = calc_effective_minutes(first_seal_time, current_time);
    let strength = minutes_sealed / 240.0;  // 总交易时长 240 分钟
    let penalty = break_count as f64 * 0.05;  // 炸板惩罚
    (strength - penalty).max(0.0).min(1.0)
}
```

### 3.4 6板块差异化涨跌停

| 板块 | 涨跌停幅度 | 特殊规则 |
|-----|-----------|---------|
| 主板 (沪/深) | 10% | - |
| 创业板 | 20% | 2020.8.24 前为 10% |
| 科创板 | 20% | - |
| 北交所 | 30% | - |
| ST 股 | 5% | - |
| 新股首日 | 不设涨跌停 | 触发临停机制 |

**涨停价向上取整、跌停价向下取整到 0.01 元**
```

---

## 四、环境准备

### 系统要求

- **操作系统**: Windows 10/11、macOS 10.15+、Linux
- **Node.js**: v18+
- **Rust**: 1.77+
- **Python**: 3.10+ (可选，用于 Sidecar)

### 安装依赖

```bash
# 1. 安装 Node.js 依赖
npm install

# 2. 安装 Rust (如未安装)
# Windows: https://rustup.rs
# macOS/Linux:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. 验证环境
node -v    # v18+
npm -v
rustc -V   # 1.77+
cargo -V

# 4. (可选) 安装 Python Sidecar 依赖
pip install -r sidecar/requirements.txt
```

---

## 五、开发运行

### 启动开发模式

```bash
# 启动前端 + Tauri 后端 (热重载)
npm run tauri dev
```

首次启动会编译 Rust 后端，可能需要几分钟。

### 访问地址

- **前端**: http://localhost:5173 (Vite 开发服务器)
- **Tauri DevTools**: `Ctrl+Shift+I` / `Cmd+Option+I`

### 开发模式特性

- ✅ 前端热重载 (HMR)
- ✅ Rust 后端自动重新编译
- ✅ DevTools 调试支持

---

## 六、项目结构

```
tickpulse/
├── src/                          # 前端源码 (Vue 3 + TS)
│   ├── views/                    # 路由视图 (12个)
│   │   ├── Home.vue              # 首页 - 自选股行情
│   │   ├── Position.vue          # 持仓管理
│   │   ├── StockMarket.vue       # 市场总览
│   │   ├── Settings.vue          # 设置页面
│   │   ├── AnalysisDashboard.vue # AI分析看板
│   │   ├── AnalysisReport.vue    # 分析报告详情
│   │   ├── AnalysisHistory.vue   # 分析历史
│   │   ├── PegBoard.vue          # PEG看板
│   │   ├── IndustryPegComparison.vue # 行业PEG对比
│   │   ├── AnomalyLog.vue        # 异动日志
│   │   └── Suspend.vue           # 悬浮窗
│   │
│   ├── components/               # Vue 组件 (58个)
│   │   ├── charts/               # 图表组件 (K线/分时/MACD/北向资金)
│   │   ├── stock/                # 股票相关 (表格/标签/搜索)
│   │   ├── layout/               # 布局组件
│   │   ├── market/               # 市场数据组件 (龙虎榜/板块/北向资金)
│   │   ├── analysis/             # AI分析组件 (维度卡片/评级标签)
│   │   ├── drawer/               # 抽屉组件 (K线/ETF/基金)
│   │   ├── position/             # 持仓组件
│   │   └── common/               # 通用组件 (对话框/颜色选择器)
│   │
│   ├── stores/                   # Pinia 状态管理 (6个)
│   │   ├── market.ts             # 行情数据
│   │   ├── alert.ts              # 预警状态
│   │   ├── anomaly.ts            # 异动事件 (FIFO队列, 最大200条)
│   │   ├── analysis.ts           # 分析结果
│   │   ├── config.ts             # 应用配置
│   │   └── page.ts               # 页面状态
│   │
│   ├── types/                    # TypeScript 类型定义
│   │   ├── stock.ts              # 股票/行情类型
│   │   ├── analysis.ts           # AI分析类型
│   │   ├── alert.ts              # 预警类型
│   │   └── config.ts             # 配置类型
│   │
│   ├── composables/              # Vue Composables
│   │   ├── useAlert.ts           # 预警逻辑
│   │   ├── useChartData.ts       # 图表数据处理
│   │   ├── useRefresh.ts         # 刷新逻辑
│   │   └── useTheme.ts           # 主题切换
│   │
│   ├── lib/                      # 工具库
│   │   ├── tauri.ts              # Tauri IPC 封装 (核心!)
│   │   ├── constants.ts          # 常量定义
│   │   └── format.ts             # 格式化工具
│   │
│   ├── router/index.ts           # 路由配置
│   └── main.ts                   # 应用入口
│
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── lib.rs                # Tauri 命令定义 (41个命令)
│   │   ├── main.rs               # 应用入口
│   │   │
│   │   ├── market/               # 行情模块 (核心)
│   │   │   ├── mod.rs            # MarketDataSource trait
│   │   │   ├── scheduler.rs      # 14阶段精细化调度器
│   │   │   ├── sources/          # 数据源实现 (6个)
│   │   │   │   ├── eastmoney.rs  # 东方财富 (优先级0, 主数据源)
│   │   │   │   ├── tencent.rs    # 腾讯财经 (优先级2)
│   │   │   │   ├── ths.rs        # 同花顺 (优先级3)
│   │   │   │   ├── sina.rs       # 新浪财经 (优先级4)
│   │   │   │   ├── fund.rs       # 基金数据源 (优先级5)
│   │   │   │   └── dzh.rs        # 大智慧 (优先级6)
│   │   │   ├── trade_calendar.rs # 交易日历
│   │   │   ├── northbound_cache.rs # 北向资金缓存
│   │   │   └── exchange.rs       # 交易所判断
│   │   │
│   │   ├── analysis/             # AI 分析引擎
│   │   │   ├── engine.rs         # 分析引擎核心
│   │   │   ├── dimensions/       # 7维度分析器
│   │   │   ├── llm/              # LLM 客户端 (Anthropic/OpenAI/Ollama)
│   │   │   ├── debate.rs         # 多空辩论引擎
│   │   │   ├── peg.rs            # PEG 计算器
│   │   │   └── quality_gate.rs   # 质量门控
│   │   │
│   │   ├── db/                   # 数据库层
│   │   │   ├── mod.rs            # DbPool (SQLite + Mutex)
│   │   │   ├── migrations.rs     # 迁移管理
│   │   │   ├── watchlist.rs      # 自选股
│   │   │   ├── position.rs       # 持仓
│   │   │   └── settings.rs       # 配置存储
│   │   │
│   │   ├── alert/                # 预警系统
│   │   │   ├── mod.rs            # 预警管理器
│   │   │   └── rules.rs          # 规则定义
│   │   │
│   │   ├── limit/                # 涨跌停判断 (6板块差异化引擎)
│   │   │   ├── mod.rs            # 涨跌停计算 (向上/向下取整)
│   │   │   └── board_type.rs     # 板块类型 (主板/创业板/科创板/北交所/ST)
│   │   │
│   │   ├── anomaly/              # 异动检测
│   │   │   ├── mod.rs            # 异动引擎
│   │   │   └── types.rs          # 异动类型
│   │   │
│   │   ├── system/               # 系统功能
│   │   │   ├── tray.rs           # 系统托盘
│   │   │   ├── hotkey.rs         # 全局快捷键
│   │   │   ├── autostart.rs      # 开机自启
│   │   │   └── window.rs         # 窗口管理
│   │   │
│   │   ├── indicator/            # 技术指标
│   │   │   ├── ma.rs             # 均线 MA
│   │   │   ├── ema.rs            # 指数均线 EMA
│   │   │   ├── macd.rs           # MACD
│   │   │   └── boll.rs           # 布林带
│   │   │
│   │   └── config.rs             # 配置常量
│   │
│   ├── db/migrations/            # SQL 迁移脚本
│   │   ├── v001_initial.sql      # 初始表结构
│   │   ├── v002_block_trades.sql # 大宗交易表
│   │   └── v003_analysis.sql     # AI分析表
│   │
│   ├── Cargo.toml                # Rust 依赖配置
│   └── tauri.conf.json           # Tauri 应用配置
│
├── sidecar/                      # Python Sidecar (可选)
│   ├── main.py                   # IPC 主循环 + 16个 handler
│   └── requirements.txt          # Python 依赖
│
├── doc/                          # 文档
│   ├── impl/                     # 实现文档 (S01-S20)
│   └── QUICKSTART.md              # 快速上手
│
├── package.json                  # Node.js 配置
├── vite.config.ts                # Vite 配置
└── tsconfig.json                 # TypeScript 配置
```

---

## 七、核心模块详解

### 7.1 前端 IPC 通信

前端通过 `src/lib/tauri.ts` 与 Rust 后端通信：

```typescript
// 获取行情数据
import { fetchQuotes } from '@/lib/tauri';
const quotes = await fetchQuotes(['1.600519', '0.000001']);

// 监听行情推送事件
import { onStockUpdate } from '@/lib/tauri';
const unsubscribe = onStockUpdate((quotes) => {
  console.log('行情更新:', quotes);
});

// 执行 AI 分析
import { analyzeStock } from '@/lib/tauri';
const result = await analyzeStock('1.600519', '贵州茅台');
```

### 7.2 后端 Tauri 命令

在 `src-tauri/src/lib.rs` 中定义了 **41 个** Tauri 命令，按功能分组：

| 分类 | 命令数量 | 主要命令 |
|-----|---------|---------|
| 自选股管理 | 8 | `get_watchlist_groups`, `add_watchlist_stock`, `toggle_pin` |
| 持仓管理 | 5 | `get_positions`, `add_position`, `get_portfolio_summary` |
| 配置管理 | 2 | `get_settings`, `update_setting` |
| 行情数据 | 6 | `fetch_quotes`, `fetch_kline`, `fetch_timeline`, `search_stock` |
| 预警系统 | 4 | `get_alert_rules`, `add_alert_rule`, `toggle_alert_rule` |
| AI 分析 | 9 | `analyze_stock`, `get_analysis_history`, `calc_peg`, `calc_cagr` |
| 系统功能 | 5 | `is_autostart_enabled`, `set_autostart`, `open_suspend_window` |
| 市场数据 | 2 | `fetch_block_trades`, `get_northbound_cache` |

```rust
// 行情相关
#[tauri::command]
async fn fetch_quotes(secids: Vec<String>) -> Result<Vec<StockQuote>, String>;

// 持仓汇总（含盈亏计算）
#[tauri::command]
async fn get_portfolio_summary(group_id: i64, benchmark_change: Option<f64>) -> Result<PortfolioSummary, String>;

// AI 分析（7维度 + 多空辩论 + 质量门控）
#[tauri::command]
async fn analyze_stock(secid: String, stock_name: String) -> Result<AnalysisResult, String>;
```

### 7.3 数据库架构

应用使用 SQLite 数据库，采用 WAL 模式，共 **16 张表**：

| 表名 | 用途 | 迁移版本 |
|-----|------|---------|
| `watchlist_groups` | 自选股分组 | v001 |
| `watchlist_stocks` | 自选股明细 | v001 |
| `positions` | 持仓记录 | v001 |
| `alert_rules` | 预警规则 | v001 |
| `kline_cache` | K线缓存 | v001 |
| `trade_calendar` | 交易日历 | v001 |
| `exrights_cache` | 除权除息缓存 | v001 |
| `settings` | 应用配置 | v001 |
| `search_history` | 搜索历史 | v001 |
| `block_trades` | 大宗交易记录 | v002 |
| `northbound_cache` | 北向资金缓存 | v002 |
| `analysis_results` | AI分析结果 | v003 |
| `peg_cache` | PEG缓存 | v003 |
| `industry_pe_cache` | 行业PE缓存 | v003 |
| `llm_config` | LLM配置 | v003 |
| `decision_memory` | 决策记忆 | v003 |

### 7.4 数据源系统

支持 **6 个数据源** 的自动切换和熔断容灾：

```rust
pub trait MarketDataSource: Send + Sync {
    async fn fetch_quotes(&self, secids: &[String]) -> Result<Vec<StockQuote>, String>;
    async fn fetch_kline(&self, secid: &str, period: KlinePeriod, ...) -> Result<Vec<KlineBar>, String>;
    fn name(&self) -> &'static str;
    fn priority(&self) -> u8;  // 优先级，越小越高
}
```

| 数据源 | 优先级 | 功能 | 备注 |
|-------|-------|------|------|
| **EastMoney** | 0 | 行情、K线、分时、搜索 | 主数据源 |
| **Tencent** | 2 | 行情 | 备用 |
| **THS** | 3 | 行情 | 同花顺 |
| **Sina** | 4 | 行情 | 备用 |
| **Fund** | 5 | 基金数据 | ETF/LOF |
| **Dzh** | 6 | 行情 | 大智慧 |

**熔断机制**: 连续失败 10 次自动切换到下一优先级数据源

### 7.5 AI 分析引擎

7维度分析架构：

```
┌─────────────────────────────────────────────────────────────┐
│                    AI 分析引擎流程                           │
├─────────────────────────────────────────────────────────────┤
│  1. 数据收集: 获取实时行情 + K线数据                         │
│                          ↓                                  │
│  2. 7维度分析 (规则引擎 + LLM增强):                         │
│     ├── IndustryTrend      行业趋势                         │
│     ├── CompetitivePosition 竞争格局                        │
│     ├── FinancialHealth    财务健康                         │
│     ├── ManagementQuality  管理层质量                       │
│     ├── GrowthPotential    成长性评估                       │
│     ├── Valuation          估值分析 (含PEG)                 │
│     └── TechnicalSignals   技术面信号                       │
│                          ↓                                  │
│  3. 多空辩论: 生成多方/空方观点 + 裁决                       │
│                          ↓                                  │
│  4. 质量门控: 置信度评估 + 质量评分                         │
│                          ↓                                  │
│  5. 报告生成: 可读报告 + 结构化数据                          │
└─────────────────────────────────────────────────────────────┘
```

支持的 LLM 供应商：
- **Anthropic**: Claude 系列
- **OpenAI**: GPT 系列
- **DeepSeek**: 国产大模型
- **Ollama**: 本地部署模型

### 7.6 异动检测引擎

实时检测 6 种异动类型：

| 异动类型 | 触发条件 | 说明 |
|---------|---------|------|
| **Surge** | 3分钟涨幅 ≥ 3% | 快速拉升 |
| **Dive** | 3分钟跌幅 ≥ 3% | 快速下跌 |
| **VolumeSpike** | 量比 ≥ 3 | 量比突增 |
| **SealBoard** | 触及涨停价 | 封板 |
| **BreakBoard** | 涨停后回落 | 炸板 |
| **TempSuspend** | 新股/异常波动 | 临时停牌 |

异动事件通过 `anomaly-detected` 事件推送到前端，前端使用 **FIFO 队列** 存储，最大 200 条。

### 7.7 前端事件系统

前端通过 `listen()` 监听后端推送的事件：

| 事件名 | 数据类型 | 触发时机 |
|-------|---------|---------|
| `stock-update` | `StockQuote[]` | 行情增量变化 |
| `scheduler-status` | `SchedulerStatus` | 阶段切换 |
| `anomaly-detected` | `AnomalyEvent[]` | 异动检测 |
| `alert-triggered` | `AlertTriggered[]` | 预警触发 |
| `market-summary` | `MarketSummary` | 涨跌家数更新 |

### 7.8 Python Sidecar

Python Sidecar 提供 16 个数据抓取 Handler，通过 stdin/stdout JSON IPC 与 Rust 主进程通信：

| Action | 功能 | 数据源 |
|--------|------|--------|
| `fetch_financial_report` | 三大财务报表 | 东方财富 datacenter |
| `fetch_shareholder_count` | 股东户数变化 | 东方财富 |
| `fetch_lockup_schedule` | 解禁时间表 | 东方财富 |
| `fetch_industry_pe` | 行业PE中位数 | 东方财富 |
| `fetch_dragon_tiger` | 龙虎榜数据 | 东方财富 datacenter |
| `fetch_northbound_flow` | 北向资金流向 | 东方财富 kamt API |
| `fetch_margin_data` | 融资融券数据 | 东方财富 |
| `fetch_stock_rank` | 股票排行榜 | 东方财富 |
| `fetch_plate_list` | 板块列表 | 东方财富 |
| `fetch_news` | 新闻快讯 | 东方财富 7x24 |
| `fetch_announcement` | 公告信息 | 巨潮资讯 |
| `fetch_research_report` | 研报数据 | 东方财富 |
| `fetch_institutional_holdings` | 机构持仓 | 东方财富 |
| `fetch_main_force_flow` | 主力资金流向 | 东方财富 |
| `fetch_sector_rotation` | 板块轮动 | 东方财富 |
| `fetch_market_sentiment` | 市场情绪 | 东方财富 |

**已知坑点**：
- 东财 datacenter 需要 `Referer: https://data.eastmoney.com/` 头
- 巨潮资讯 PDF 需特殊 `User-Agent`
- 新浪接口偶尔返回空数据
- 腾讯接口返回 GBK 编码

---

## 八、构建发布

### 构建应用

```bash
# 构建生产版本
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`：
- **Windows**: `.msi` / `.exe` 安装包
- **macOS**: `.dmg` / `.app`
- **Linux**: `.deb` / `.AppImage`

### 构建配置

编辑 `src-tauri/tauri.conf.json`：

```json
{
  "productName": "TickPulse",
  "version": "0.1.0",
  "identifier": "com.tickpulse.app",
  "bundle": {
    "active": true,
    "targets": "all"
  }
}
```

---

## 九、配置说明

### 9.1 LLM 配置

在设置页面配置 AI 分析引擎：

```typescript
interface LlmConfig {
  provider: "anthropic" | "openai" | "deepseek" | "ollama";
  model: string;           // 如 "claude-sonnet-4-6"
  apiKey: string | null;   // API 密钥
  baseUrl: string | null;  // 自定义端点 (Ollama 用)
  mode: "cloud" | "local"; // 云端/本地
}
```

### 9.2 刷新间隔

默认 10 秒刷新一次行情，可在设置中调整。

### 9.3 数据目录

数据库和配置存储位置：
- **Windows**: `%APPDATA%/com.tickpulse.app/`
- **macOS**: `~/Library/Application Support/com.tickpulse.app/`
- **Linux**: `~/.config/com.tickpulse.app/`

---

## 十、开发指南

### 10.1 添加新的 Tauri 命令

1. 在 `src-tauri/src/lib.rs` 中定义命令：

```rust
#[tauri::command]
fn my_new_command(param: &str) -> Result<String, String> {
    Ok(format!("Hello, {}", param))
}
```

2. 注册命令：

```rust
.invoke_handler(tauri::generate_handler![
    // ... 其他命令
    my_new_command,
])
```

3. 在前端调用：

```typescript
// src/lib/tauri.ts
export async function myNewCommand(param: string): Promise<string> {
  return invoke("my_new_command", { param });
}
```

### 10.2 添加新的数据库迁移

1. 创建迁移文件 `src-tauri/src/db/migrations/v004_xxx.sql`
2. 在 `migrations.rs` 中注册：

```rust
Migration {
    version: 4,
    description: "xxx",
    sql: include_str!("migrations/v004_xxx.sql"),
},
```

### 10.3 添加新的分析维度

1. 在 `src-tauri/src/analysis/dimensions/` 创建模块
2. 实现 `analyze_xxx()` 函数
3. 在 `engine.rs` 中注册维度

---

## 十一、常见问题

### Q: 首次编译很慢？

A: Rust 编译需要时间，首次编译后会缓存。建议使用 `cargo sccache` 加速。

### Q: 如何调试 Rust 后端？

A: 使用 `tracing` 日志：
```rust
tracing::info!("调试信息: {:?}", data);
```
日志会在控制台输出。

### Q: 数据源请求失败？

A: 检查网络连接，应用有熔断机制，会自动切换备用数据源。

### Q: 如何启用 Python Sidecar？

A: 安装依赖后在设置中启用，应用会通过 stdin/stdout JSON IPC 与 Python 进程通信。

---

## 十二、相关资源

- **Tauri 文档**: https://tauri.app/v2/guide/
- **Vue 3 文档**: https://vuejs.org/
- **Pinia 文档**: https://pinia.vuejs.org/
- **Naive UI 文档**: https://www.naiveui.com/
- **ECharts 文档**: https://echarts.apache.org/

---

## 十三、更新日志

| 版本 | 日期 | 更新内容 |
|------|------|---------|
| 0.1.0 | 2026-06 | 初始版本，S01-S09 核心功能实现 |

---

*文档生成时间: 2026-06-11*
