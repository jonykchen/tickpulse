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
