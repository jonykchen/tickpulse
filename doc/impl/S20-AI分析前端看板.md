# S20 — AI 分析前端看板

> 阶段：Phase 9 | 前置依赖：S19 | 来源：v3.0 §3.1-3.6

## 1. 概述

本子文档覆盖 v3.0 §3 前端分析看板的完整设计，包含新增页面与组件、PEG 看板、行业 PE 对比图、分析报告页面、AnalysisStore 定义、自选股列扩展及路由设计。

## 2. 新增页面与组件目录

```
新增视图/
├── AnalysisDashboard.vue      # AI分析主页面
├── PegBoard.vue               # PEG看板（表格+评级色条）
├── AnalysisReport.vue         # 分析报告展示（Markdown渲染）
├── IndustryPegComparison.vue  # 行业PE/PEG横向对比图
└── AnalysisHistory.vue        # 历史分析记录

新增组件/
├── stock/
│   ├── PegRatingTag.vue       # PEG评级标签（低估绿/合理黄/高估红）
│   ├── AnalysisBadge.vue      # AI分析状态标记
│   └── IndustryPegTag.vue     # 行业PE排名小标签
├── charts/
│   ├── IndustryPegChart.vue   # 行业PE分布图（ECharts箱线图）
│   └── PegTrendChart.vue      # PEG趋势图（历史PE/CAGR变化）
└── analysis/
    ├── DimensionCard.vue      # 单维度分析卡片（评级+摘要+详情折叠）
    ├── OverallRatingBar.vue   # 综合评级条（5级色条）
    └── AnalysisTriggerConfig.vue # 自动分析触发配置
```

## 3. PEG 看板设计

```
┌──────────────────────────────────────────────────────────────────┐
│ PEG 看板                                    [AI分析] [刷新行情]  │
├──────────────────────────────────────────────────────────────────┤
│ 名称     价格   涨跌%   PE(TTM)  PB   PEG  评级   消化年限  操作 │
│ ─────────────────────────────────────────────────────────────────│
│ 贵州茅台 1856   +1.2%   28.5    9.2  1.2  🟢合理  1.8年   分析 │
│ 宁德时代 218    -0.8%   22.1    5.8  0.7  🟢低估  1.2年   分析 │
│ 比亚迪   312    +2.5%   35.2    7.1  1.8  🟡偏贵  3.5年   分析 │
│ 某ST股   3.2    +5.0%   —       —    —    🔴亏损  —       分析 │
│                                                                  │
│  评级色条：🟢<0.5极度低估 🟢0.5-1.0低估 🟡1.0-1.5合理          │
│           🟠1.5-2.0偏贵 🔴>2.0高估                              │
└──────────────────────────────────────────────────────────────────┘
```

## 4. 行业PE对比图

```
┌──────────────────────────────────────────────────────────────────┐
│ 行业PE对比 — 白酒                                                 │
│                                                                  │
│  PE(TTM)                                                         │
│  80 ┤                                                            │
│  60 ┤  ●五粮液58                                                  │
│  40 ┤       ●茅台28    ●洋河22                                    │
│  20 ┤              ●老白干18    ●古井16                           │
│   0 ┼────┬────┬────┬────┬────┬────                               │
│      排名 1    2    3    4    5                                   │
│                                                                  │
│  行业均值: 32.5  中位数: 25.0  当前股排位: 3/20                    │
│  ▲ 当前股PE在行业中处于 [偏低] 位置                               │
└──────────────────────────────────────────────────────────────────┘
```

## 5. 分析报告页面

```
┌──────────────────────────────────────────────────────────────────┐
│ ← 返回    贵州茅台(600519) AI分析报告                             │
│           2026-06-10 15:35  盘后自动触发                          │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  综合评级：█████████████░ Hold（中性）                            │
│                                                                  │
│  ┌── PEG 估值 ──────────────────────────────────────────┐       │
│  │ PE(TTM): 28.5  前瞻PE: 25.2  CAGR: 21.3%            │       │
│  │ PEG: 1.18 🟢合理区间                                  │       │
│  │ PE消化年限: 1.8年（成长性强）                          │       │
│  │ 行业PE百分位: 35%（偏低）                              │       │
│  └───────────────────────────────────────────────────────┘       │
│                                                                  │
│  ┌── 技术面 📊 ─────────── 利好 ────────────────────────┐       │
│  │ K线站上5日/10日均线，MACD金叉形成...                    │       │
│  │ [展开详情 ▼]                                           │       │
│  └───────────────────────────────────────────────────────┘       │
│                                                                  │
│  ┌── 游资追踪 🔥 ───────── 中性 ───────────────────────┐       │
│  │ 近3日主力净流入1.2亿，龙虎榜无上榜...                   │       │
│  │ [展开详情 ▼]                                           │       │
│  └───────────────────────────────────────────────────────┘       │
│                                                                  │
│  ┌── 解禁监控 🔓 ───────── 利好 ───────────────────────┐       │
│  │ 未来90天无限售解禁，近6月无大股东减持...                │       │
│  │ [展开详情 ▼]                                           │       │
│  └───────────────────────────────────────────────────────┘       │
│                                                                  │
│  ...（其他维度折叠展示）                                          │
│                                                                  │
│  ┌── 风险提示 ──────────────────────────────────────────┐       │
│  │ ① 白酒行业增速放缓，PEG基于历史增速可能高估            │       │
│  │ ② 北向资金近期持续流出白酒板块                         │       │
│  └───────────────────────────────────────────────────────┘       │
│                                                                  │
│  [导出PDF] [复制Markdown] [历史对比]                              │
└──────────────────────────────────────────────────────────────────┘
```

### 5.1 报告页面结构说明

| 区域 | 内容 | 交互 |
|------|------|------|
| Header | 返回按钮 + 股票名+代码 + 日期时间 + 触发方式 | 返回导航 |
| 综合评级条 | 5级色条 (Buy/Overweight/Hold/Underweight/Sell) + 评级文字 | 无 |
| PEG 估值卡片 | PE(TTM)/前瞻PE/CAGR/PEG/PE消化年限/行业PE百分位 | 无 |
| 7个维度折叠卡片 | 技术面📊/新闻面📰/基本面💰/政策面🏛️/游资追踪🔥/解禁监控🔓/PEG估值📈 | 评级+摘要+展开详情 |
| 风险提示卡片 | 风险条目列表 | 无 |
| 底部操作按钮 | 导出PDF / 复制Markdown / 历史对比 | 各按钮触发对应操作 |

## 6. AnalysisStore 定义

```typescript
// stores/analysis.ts
interface AnalysisStore {
  // 分析结果
  analyses: Map<string, AnalysisResult>;  // secid → 最新分析
  analysisHistory: AnalysisRecord[];       // 历史记录列表

  // PEG 看板数据
  pegBoard: Map<string, PegSummary>;      // secid → PEG概要

  // 行业PE对比
  industryComparison: Map<string, IndustryPegData>;  // secid → 行业PE

  // 分析状态
  isAnalyzing: boolean;
  analyzingSecid: string | null;
  analysisProgress: AnalysisProgress | null;

  // 操作
  triggerAnalysis(secid: string, trigger: AnalysisTrigger): Promise<void>;
  fetchPegBoard(): Promise<void>;
  fetchIndustryComparison(secid: string): Promise<void>;
  exportReport(analysisId: string, format: 'pdf' | 'markdown'): Promise<void>;
}

interface AnalysisProgress {
  total_dimensions: number;      // 总维度数
  completed_dimensions: number;  // 已完成维度
  current_dimension: string;     // 当前分析维度
  data_collecting: string[];     // 正在采集的数据
  estimated_seconds: u32;        // 预计剩余时间
}
```

## 7. 自选股列扩展

| 新增列 | 默认可见 | 可排序 | 说明 |
|--------|----------|--------|------|
| PEG | ❌ | ✅ | PEG值（需有一致预期EPS数据） |
| PEG评级 | ❌ | ✅ | 极度低估/低估/合理/偏贵/高估 |
| 综合评级 | ❌ | ✅ | Buy/Overweight/Hold/Underweight/Sell |
| 分析时间 | ❌ | ✅ | 最近一次AI分析时间 |
| 行业PE排名 | ❌ | ✅ | 在行业中的PE百分位 |

## 8. 路由设计

分析相关页面的路由扩展：

| 路由路径 | 组件 | 说明 |
|----------|------|------|
| `/analysis` | AnalysisDashboard.vue | AI分析主页面 |
| `/analysis/peg-board` | PegBoard.vue | PEG看板 |
| `/analysis/report/:secid` | AnalysisReport.vue | 分析报告（按股票） |
| `/analysis/industry/:secid` | IndustryPegComparison.vue | 行业PE对比（按股票） |
| `/analysis/history` | AnalysisHistory.vue | 历史分析记录 |

## 9. 测试要点

| 测试项 | 验证内容 |
|--------|---------|
| PEG 看板渲染 | 表格数据正确显示，评级色条颜色与 PEG 值对应 |
| PEG 评级色条 | 🟢<0.5极度低估 / 🟢0.5-1.0低估 / 🟡1.0-1.5合理 / 🟠1.5-2.0偏贵 / 🔴>2.0高估 |
| 行业PE对比图 | 散点位置正确，行业均值/中位数/当前股排位准确 |
| 分析报告页面 | 7维度卡片折叠/展开，综合评级条色段比例正确 |
| 分析报告导出 | PDF 导出内容完整，Markdown 复制格式正确 |
| AnalysisStore | triggerAnalysis/fetchPegBoard/fetchIndustryComparison/exportReport 调用正确 |
| 分析进度 | AnalysisProgress 各字段正确更新，前端实时反映 |
| 自选股列 | 5个新增列默认不可见，可排序，数据正确 |
| 亏损股处理 | PEG/PE/PB 显示 "—" 而非错误值 |
| 路由导航 | 各分析页面路由正确跳转，参数传递正确 |

## 10. 文件清单

| 文件路径 | 说明 |
|----------|------|
| `src/views/AnalysisDashboard.vue` | AI分析主页面 |
| `src/views/PegBoard.vue` | PEG看板页面 |
| `src/views/AnalysisReport.vue` | 分析报告页面 |
| `src/views/IndustryPegComparison.vue` | 行业PE对比页面 |
| `src/views/AnalysisHistory.vue` | 历史分析记录页面 |
| `src/components/stock/PegRatingTag.vue` | PEG评级标签组件 |
| `src/components/stock/AnalysisBadge.vue` | AI分析状态标记组件 |
| `src/components/stock/IndustryPegTag.vue` | 行业PE排名小标签组件 |
| `src/components/charts/IndustryPegChart.vue` | 行业PE分布图组件 |
| `src/components/charts/PegTrendChart.vue` | PEG趋势图组件 |
| `src/components/analysis/DimensionCard.vue` | 单维度分析卡片组件 |
| `src/components/analysis/OverallRatingBar.vue` | 综合评级条组件 |
| `src/components/analysis/AnalysisTriggerConfig.vue` | 自动分析触发配置组件 |
| `src/stores/analysis.ts` | 分析 Store |
