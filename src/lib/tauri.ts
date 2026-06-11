/**
 * Tauri IPC 封装
 * 统一前后端通信接口
 */
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { StockQuote } from "@/types/stock";

// ==================== 命令调用 ====================

/** 获取自选股行情 */
export async function fetchQuotes(secids: string[]): Promise<StockQuote[]> {
  return invoke<StockQuote[]>("fetch_quotes", { secids });
}

/** 添加自选股 */
export async function addWatchlistStock(
  groupId: number,
  secid: string,
  name: string
): Promise<void> {
  return invoke("add_watchlist_stock", { groupId, secid, name });
}

/** 删除自选股 */
export async function removeWatchlistStock(id: number): Promise<void> {
  return invoke("remove_watchlist_stock", { id });
}

/** 获取自选股分组列表 */
export async function getWatchlistGroups(): Promise<
  { id: number; name: string; sortOrder: number }[]
> {
  return invoke("get_watchlist_groups");
}

/** 创建自选股分组 */
export async function createWatchlistGroup(name: string): Promise<void> {
  return invoke("create_watchlist_group", { name });
}

/** 删除自选股分组 */
export async function deleteWatchlistGroup(id: number): Promise<void> {
  return invoke("delete_watchlist_group", { id });
}

/** 获取分组下的自选股 */
export async function getWatchlistStocks(
  groupId: number
): Promise<{ id: number; secid: string; name: string; sortOrder: number }[]> {
  return invoke("get_watchlist_stocks", { groupId });
}

/** 搜索股票 */
export async function searchStock(
  keyword: string
): Promise<{ secid: string; code: string; name: string }[]> {
  return invoke("search_stock", { keyword });
}

/** 获取K线数据 */
export async function fetchKline(
  secid: string,
  period: string,
  limit: number,
  adjust: string
): Promise<unknown[]> {
  return invoke("fetch_kline", { secid, period, limit, adjust });
}

/** 获取分时走势 */
export async function fetchTimeline(secid: string): Promise<unknown> {
  return invoke("fetch_timeline", { secid });
}

/** 获取除权除息 */
export async function fetchExrights(secid: string): Promise<unknown[]> {
  return invoke("fetch_exrights", { secid });
}

/** 获取健康诊断 */
export async function getHealthMetrics(): Promise<{
  lastRefreshMs: number;
  avgRefreshMs: number;
  emitLatencyMs: number;
  circuitBreakerStatus: { sourceName: string; state: string; consecutiveFailures: number }[];
  memoryRssMb: number;
  dbSizeMb: number;
  uptimeSecs: number;
}> {
  return invoke("get_health_metrics");
}

/** 获取持仓列表 */
export async function getPositions(
  groupId: number
): Promise<{ id: number; secid: string; name: string; costPrice: string; quantity: string }[]> {
  return invoke("get_positions", { groupId });
}

/** 添加持仓 */
export async function addPosition(
  groupId: number,
  secid: string,
  name: string | null,
  costPrice: string,
  quantity: string
): Promise<number> {
  return invoke("add_position", { groupId, secid, name, costPrice, quantity });
}

/** 更新持仓 */
export async function updatePosition(
  id: number,
  costPrice: string,
  quantity: string
): Promise<void> {
  return invoke("update_position", { id, costPrice, quantity });
}

/** 删除持仓 */
export async function deletePosition(id: number): Promise<void> {
  return invoke("delete_position", { id });
}

/** 获取配置 */
export async function getSettings(): Promise<Record<string, string>> {
  return invoke("get_settings");
}

/** 更新配置 */
export async function updateSetting(
  key: string,
  value: string
): Promise<void> {
  return invoke("update_setting", { key, value });
}

// ==================== 事件监听 ====================

/** 行情增量更新事件 */
export function onStockUpdate(
  callback: (quotes: StockQuote[]) => void
): Promise<() => void> {
  return listen<StockQuote[]>("stock-update", (event) => {
    callback(event.payload);
  });
}

/** 调度器状态事件 */
export function onSchedulerStatus(
  callback: (status: {
    phase: string;
    intervalSecs: number;
    isTradingDay: boolean;
  }) => void
): Promise<() => void> {
  return listen<{
    phase: string;
    intervalSecs: number;
    isTradingDay: boolean;
  }>("scheduler-status", (event) => {
    callback(event.payload);
  });
}

/** 异动检测事件 */
export function onAnomalyDetected(
  callback: (anomalies: unknown[]) => void
): Promise<() => void> {
  return listen<unknown[]>("anomaly-detected", (event) => {
    callback(event.payload);
  });
}

/** 涨跌家数摘要事件 */
export function onMarketSummary(
  callback: (summary: {
    upCount: number;
    downCount: number;
    flatCount: number;
    limitUpCount: number;
    limitDownCount: number;
  }) => void
): Promise<() => void> {
  return listen<{
    upCount: number;
    downCount: number;
    flatCount: number;
    limitUpCount: number;
    limitDownCount: number;
  }>("market-summary", (event) => {
    callback(event.payload);
  });
}

// ==================== 持仓汇总 ====================

/** 获取持仓汇总 */
export async function getPortfolioSummary(
  groupId: number,
  benchmarkChange?: number
): Promise<import("@/types/portfolio").PortfolioSummary> {
  return invoke("get_portfolio_summary", { groupId, benchmarkChange: benchmarkChange ?? null });
}

// ==================== 预警规则 ====================

/** 获取预警规则列表 */
export async function getAlertRules(): Promise<import("@/types/alert").AlertRule[]> {
  return invoke("get_alert_rules");
}

/** 添加预警规则 */
export async function addAlertRule(
  secid: string,
  stockName: string,
  ruleType: string,
  threshold: number
): Promise<void> {
  return invoke("add_alert_rule", { secid, stockName, ruleType, threshold });
}

/** 删除预警规则 */
export async function removeAlertRule(ruleId: string): Promise<void> {
  return invoke("remove_alert_rule", { ruleId });
}

/** 切换预警规则启用/禁用 */
export async function toggleAlertRule(
  ruleId: string,
  enabled: boolean
): Promise<void> {
  return invoke("toggle_alert_rule", { ruleId, enabled });
}

// ==================== 大宗交易 ====================

/** 获取大宗交易数据 */
export async function fetchBlockTrades(
  date?: string
): Promise<import("@/types/portfolio").BlockTrade[]> {
  return invoke("fetch_block_trades", { date: date ?? null });
}

// ==================== 预警触发事件 ====================

/** 预警触发事件 */
export function onAlertTriggered(
  callback: (alerts: import("@/types/alert").AlertTriggered[]) => void
): Promise<() => void> {
  return listen<import("@/types/alert").AlertTriggered[]>("alert-triggered", (event) => {
    callback(event.payload);
  });
}

// ==================== AI 分析引擎 ====================

/** 执行股票分析 */
export async function analyzeStock(
  secid: string,
  stockName: string
): Promise<import("@/types/analysis").AnalysisResult> {
  return invoke("analyze_stock", { secid, stockName });
}

/** 获取分析历史 */
export async function getAnalysisHistory(
  secid?: string,
  limit?: number
): Promise<import("@/types/analysis").AnalysisResult[]> {
  return invoke("get_analysis_history", { secid: secid ?? null, limit: limit ?? null });
}

/** 获取 LLM 配置 */
export async function getLlmConfig(): Promise<import("@/types/analysis").LlmConfig> {
  return invoke("get_llm_config");
}

/** 保存 LLM 配置 */
export async function saveLlmConfig(
  provider: string,
  model: string,
  apiKey?: string,
  baseUrl?: string,
  mode?: string,
  thinkingEnabled?: boolean
): Promise<void> {
  return invoke("save_llm_config", {
    provider, model,
    apiKey: apiKey ?? null,
    baseUrl: baseUrl ?? null,
    mode: mode ?? "cloud",
    thinkingEnabled: thinkingEnabled ?? false,
  });
}

/** 获取双 LLM 配置 */
export async function getDualLlmConfig(): Promise<import("@/types/analysis").DualLlmConfig> {
  return invoke("get_dual_llm_config");
}

/** 保存双 LLM 配置 */
export async function saveDualLlmConfig(
  quickProvider: string,
  quickModel: string,
  quickApiKey?: string,
  quickBaseUrl?: string,
  quickMode?: string,
  quickThinkingEnabled?: boolean,
  deepProvider?: string,
  deepModel?: string,
  deepApiKey?: string,
  deepBaseUrl?: string,
  deepMode?: string,
  deepThinkingEnabled?: boolean
): Promise<void> {
  return invoke("save_dual_llm_config", {
    quickProvider, quickModel,
    quickApiKey: quickApiKey ?? null,
    quickBaseUrl: quickBaseUrl ?? null,
    quickMode: quickMode ?? "cloud",
    quickThinkingEnabled: quickThinkingEnabled ?? false,
    deepProvider: deepProvider ?? null,
    deepModel: deepModel ?? null,
    deepApiKey: deepApiKey ?? null,
    deepBaseUrl: deepBaseUrl ?? null,
    deepMode: deepMode ?? null,
    deepThinkingEnabled: deepThinkingEnabled ?? false,
  });
}

/** 获取支持的供应商列表 */
export async function getSupportedProviders(): Promise<import("@/types/analysis").SupportedProvider[]> {
  return invoke("get_supported_providers");
}

/** 获取分析预设 */
export async function getAnalysisPresets(): Promise<unknown[]> {
  return invoke("get_analysis_presets");
}

/** 计算 PEG */
export async function calcPeg(
  pe: number,
  cagr: number
): Promise<{ peg: number; rating: string; ratingDisplay: string; ratingScore: number }> {
  return invoke("calc_peg", { pe, cagr });
}

/** 计算 CAGR */
export async function calcCagr(
  beginValue: number,
  endValue: number,
  years: number
): Promise<number> {
  return invoke("calc_cagr", { beginValue, endValue, years });
}

/** 删除分析结果 */
export async function deleteAnalysisResult(id: string): Promise<void> {
  return invoke("delete_analysis_result", { id });
}

// ==================== 系统功能 ====================

/** 重置每日预警 */
export async function resetDailyAlerts(): Promise<void> {
  return invoke("reset_daily_alerts");
}

/** 查询自启状态 */
export async function isAutostartEnabled(): Promise<boolean> {
  return invoke("is_autostart_enabled");
}

/** 设置自启状态 */
export async function setAutostart(enabled: boolean): Promise<void> {
  return invoke("set_autostart", { enabled });
}

/** 打开悬浮窗 */
export async function openSuspendWindow(): Promise<void> {
  return invoke("open_suspend_window");
}

/** 关闭悬浮窗 */
export async function closeSuspendWindow(): Promise<void> {
  return invoke("close_suspend_window");
}

// ==================== 北向资金 ====================

/** 获取北向资金缓存数据 */
export async function getNorthboundCache(
  days: number
): Promise<{ tradeDate: string; shNetInflow: number; szNetInflow: number; totalNetInflow: number }[]> {
  return invoke("get_northbound_cache", { days });
}

// ==================== 日志系统 ====================

/** 获取最近日志 */
export async function getRecentLogs(
  count?: number
): Promise<import("@/types/log").LogEntry[]> {
  return invoke("get_recent_logs", { count: count ?? 100 });
}

/** 清空日志缓冲区 */
export async function clearLogs(): Promise<void> {
  return invoke("clear_logs");
}

/** 获取日志文件路径 */
export async function getLogFilePath(): Promise<string> {
  return invoke("get_log_file_path");
}
