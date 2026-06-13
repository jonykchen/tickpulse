# TickPulse

<p align="center">
  <strong>A股桌面行情监控应用</strong>
</p>

<p align="center">
  基于 Tauri 2 + Vue 3 + Rust 构建的高性能股票行情监控工具
</p>

<p align="center">
  <a href="#功能特性">功能特性</a> •
  <a href="#截图">截图</a> •
  <a href="#快速开始">快速开始</a> •
  <a href="#技术架构">技术架构</a> •
  <a href="#文档">文档</a>
</p>

---

## 功能特性

### 📊 实时行情
- 自选股实时行情刷新，支持 14 阶段精细化调度
- 涨跌停判断（6 板块差异化引擎）
- 封板强度监控 + 炸板检测
- 量比衰减提示

### 📈 持仓管理
- 持仓盈亏计算
- 组合收益汇总
- 基准对比（沪深300超额收益）

### 🔔 预警系统
- 价格突破、涨跌幅、量比等多维度预警规则
- 系统原生 Toast 通知

### 🤖 AI 分析引擎
- 7 维度分析（行业趋势、竞争格局、财务健康等）
- PEG 估值计算
- 多空辩论引擎
- 质量门控
- 支持多 LLM 供应商（Claude、OpenAI、DeepSeek、Ollama）

### 📉 K线图表
- 分时走势图 + 五日分时
- 日K/周K/月K
- MACD、BOLL、MA 指标叠加
- 前复权/后复权/不复权切换

### 💹 市场数据
- 北向资金流向
- 大宗交易
- 龙虎榜
- 涨跌停统计
- 涨停天梯

### ⚡ 系统功能
- 系统托盘（跨平台）
- 开机自启
- 全局快捷键
- 悬浮窗

---

## 截图

> 📸 待添加应用截图

---

## 快速开始

### 系统要求

- **操作系统**: Windows 10/11、macOS 10.15+、Linux
- **Node.js**: v18+
- **Rust**: 1.77+
- **Python**: 3.10+（可选，用于 Sidecar）

### 安装依赖

```bash
# 克隆项目
git clone https://github.com/jonychen/tickpulse.git
cd tickpulse

# 安装 Node.js 依赖
npm install

# 验证 Rust 环境
rustc -V   # 需要 1.77+
cargo -V
```

### 开发运行

```bash
# 启动开发模式（前端热重载 + Rust 后端）
npm run tauri dev
```

首次启动会编译 Rust 后端，可能需要几分钟。

### 构建发布

```bash
# 构建生产版本
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`：
- Windows: `.msi` / `.exe`
- macOS: `.dmg` / `.app`
- Linux: `.deb` / `.AppImage`

---

## 技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                      TickPulse 应用架构                      │
├─────────────────────────────────────────────────────────────┤
│  前端 (Vue 3 + TypeScript)                                  │
│  ├── 视图层: 12 个路由视图                                   │
│  ├── 组件层: 50+ 个 Vue 组件                                  │
│  ├── 状态管理: 7 个 Pinia Stores                             │
│  └── IPC 封装: 50+ API 函数                                  │
├─────────────────────────────────────────────────────────────┤
│  Tauri Bridge (IPC)                                         │
├─────────────────────────────────────────────────────────────┤
│  后端 (Rust)                                                │
│  ├── 行情模块: 14阶段调度、6数据源容灾                        │
│  ├── 分析引擎: 7维度分析、LLM集成                            │
│  ├── 数据库: SQLite + WAL模式、16张表                        │
│  ├── 预警系统: 规则引擎、触发检测                             │
│  └── 异动检测: 快速拉升/下跌/量比突增                        │
└─────────────────────────────────────────────────────────────┘
```

### 技术栈

| 类别 | 技术 |
|-----|------|
| 桌面框架 | Tauri 2.x |
| 后端语言 | Rust (2021 edition) |
| 前端框架 | Vue 3 + TypeScript |
| 状态管理 | Pinia |
| UI 库 | Naive UI |
| 图表 | LightweightCharts + ECharts |
| 数据库 | SQLite (rusqlite) |
| 异步运行时 | Tokio |

---

## 文档

- [快速上手文档](doc/QUICKSTART.md) - 完整的开发指南
- [实现文档](doc/impl/) - S01-S20 各阶段实现

---

## 核心亮点

### 14 阶段精细化调度

根据交易时段自动调整刷新策略，开盘/收盘阶段高频刷新，午休阶段降频节省资源。

### 6 板块差异化涨跌停

主板 10%、创业板/科创板 20%、北交所 30%、ST 5%，涨停价向上取整、跌停价向下取整。

### 6 数据源容灾

东方财富、腾讯、同花顺、新浪、大智慧、基金数据源自动切换，熔断保护。

### 7 维度 AI 分析

行业趋势 → 竞争格局 → 财务健康 → 管理层质量 → 成长性 → 估值 → 技术面

---

## 贡献

欢迎提交 Issue 和 Pull Request！

请阅读 [贡献指南](CONTRIBUTING.md) 了解详情。

---

## 许可证

本项目基于 [MIT License](LICENSE) 开源。

---

## 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Naive UI](https://www.naiveui.com/) - Vue 3 组件库
- [Lightweight Charts](https://www.tradingview.com/lightweight-charts/) - TradingView 图表库
- [ECharts](https://echarts.apache.org/) - Apache 开源图表库
