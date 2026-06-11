# S04 — 数据源Trait与东方财富API
> 阶段：Phase 1 | 前置依赖：S02 | 来源：v2.0 §3.2 + §6.1 + §6.2 + §6.3

## 1. 概述

数据源 Trait 定义了行情数据的统一接口，支持多数据源切换和容灾。东方财富作为主数据源，提供实时行情、K线、分时、搜索等完整功能。

## 2. 接口定义

### 2.1 MarketDataSource Trait

```rust
#[async_trait]
pub trait MarketDataSource: Send + Sync {
    async fn fetch_quotes(&self, secids: &[String]) -> Result<Vec<StockQuote>>;
    async fn fetch_kline(&self, secid: &str, period: KlinePeriod, limit: u32)
        -> Result<Vec<KlineBar>>;
    async fn fetch_timeline(&self, secid: &str) -> Result<TimelineData>;  // 分时走势
    async fn search(&self, keyword: &str) -> Result<Vec<SearchResult>>;
    async fn fetch_exrights(&self, secid: &str) -> Result<Vec<ExRightInfo>>;  // 除权除息
    fn name(&self) -> &'static str;
    fn priority(&self) -> u8;
}
```

## 3. 数据结构

### 3.1 东方财富字段映射完整表

| 字段 | 含义 | 字段 | 含义 |
|------|------|------|------|
| f2 | 最新价 | f3 | 涨跌幅(%) |
| f4 | 涨跌额 | f5 | 成交量(手) |
| f6 | 成交额 | f8 | 换手率(流通) |
| f9 | 市盈率(动) | f10 | 量比 |
| f12 | 代码 | f13 | 市场(0深/1沪) |
| f14 | 名称 | f15 | 最高 |
| f16 | 最低 | f17 | 开盘 |
| f18 | 昨收 | f20 | 总市值 |
| f22 | 涨速 | f25 | 年初至今涨幅 |
| f62 | 主力净流入 | f136 | 市净率 |
| f23 | 市盈率(静) | f9 | 市盈率(动/TTM) |

### 3.2 市场标识（f13）

| f13 值 | 市场 |
|--------|------|
| 0 | 深交所 |
| 1 | 上交所 |
| 116 | 港股 |
| 134 | 港股 |
| 105 | 美股 |
| 106 | 美股 |

### 3.3 v2.0 字段补充

- `f23`：静态市盈率
- `f9`：东方财富实际返回 TTM 市盈率（非动态，需验证）
- `f8`：流通换手率（非总换手率）

## 4. 核心逻辑

### 4.1 东方财富 API 端点（11个）

| 端点 | 用途 |
|------|------|
| `push2delay.eastmoney.com/api/qt/ulist.np/get` | 实时行情批量查询 |
| `push2delay.eastmoney.com/api/qt/clist/get` | 股票列表查询 |
| `push2delay.eastmoney.com/api/qt/stock/trends2/get` | 分时走势数据 |
| `push2his.eastmoney.com` | 历史K线数据（含复权参数fqt） |
| `searchapi.eastmoney.com/api/Info/Search` | 股票搜索 |
| `fundmobapi.eastmoney.com` | 基金数据 |
| `datacenter-web.eastmoney.com` | ETF/板块成分/龙虎榜/大宗交易 |
| `push2his.eastmoney.com/api/qt/stock/tradeCalendar/get` | 交易日历 |
| `push2his.eastmoney.com/api/qt/kamt.kline/get` | 北向资金 |
| `datacenter-web.eastmoney.com/api/data/v1/get` | 龙虎榜（reportName=RPT_DAILYBILLBOARD_DETAILSNEW） |
| `datacenter-web.eastmoney.com/api/data/v1/get` | 大宗交易（reportName=RPT_DABLOCKTRADE_DETAILS） |

### 4.2 其他数据源 API 端点（4个）

| 数据源 | 端点 | 用途 |
|--------|------|------|
| 新浪财经 | `money.finance.sina.com.cn` | K线 fallback |
| 同花顺 | `dq.10jqka.com.cn/fuyao/...` | 热门股、涨跌分布 |
| 天天基金 | `fundgz.1234567.com.cn` | 基金实时估值（JSONP） |
| 大智慧 | `webrelease.dzh.com.cn` | 涨停天数（涨停天梯） |

## 5. 测试要点

1. **批量行情**：200只股票分批请求，验证并行效率
2. **字段映射**：验证 f2-f136 所有字段正确解析
3. **市场标识**：验证 f13 值正确区分深交所/上交所/港股/美股
4. **复权参数**：验证 fqt 参数 0/1/2 对应不复权/前复权/后复权
5. **错误处理**：验证网络超时、API 限流、数据格式异常的处理
6. **多数据源切换**：验证主数据源失败时自动切换到备用数据源

## 6. 文件清单

| 文件路径 | 说明 |
|----------|------|
| `src-tauri/src/market/mod.rs` | MarketDataSource trait 定义 |
| `src-tauri/src/market/sources/eastmoney.rs` | 东方财富 API 实现 |
| `src-tauri/src/market/sources/ths.rs` | 同花顺 API 实现 |
| `src-tauri/src/market/sources/sina.rs` | 新浪财经 API 实现（K线 fallback） |
| `src-tauri/src/market/sources/fund.rs` | 天天基金 + fundmobapi 实现 |
| `src-tauri/src/market/sources/dzh.rs` | 大智慧 API 实现（涨停天梯） |
| `src-tauri/src/datasource/mod.rs` | DataSourceManager + CircuitBreaker |
| `src-tauri/src/datasource/rate_limiter.rs` | 请求限速器 |
