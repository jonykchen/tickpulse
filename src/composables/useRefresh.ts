/**
 * 智能刷新调度
 * 监听 Tauri 后端推送事件，自动订阅/取消订阅
 */
import { ref, onMounted, onUnmounted } from "vue";
import {
  onStockUpdate,
  onSchedulerStatus,
  onMarketSummary,
  onAnomalyDetected,
} from "@/lib/tauri";
import type { StockQuote } from "@/types/stock";
import type { AnomalyEvent } from "@/types/anomaly";

/** 调度器状态 */
export interface SchedulerStatus {
  phase: string;
  intervalSecs: number;
  isTradingDay: boolean;
}

/** 涨跌家数摘要 */
export interface MarketSummary {
  upCount: number;
  downCount: number;
  flatCount: number;
  limitUpCount: number;
  limitDownCount: number;
}

export function useRefresh() {
  const isConnected = ref(false);
  const quotes = ref<StockQuote[]>([]);
  const schedulerStatus = ref<SchedulerStatus | null>(null);
  const marketSummary = ref<MarketSummary | null>(null);
  const anomalies = ref<AnomalyEvent[]>([]);

  /** 事件取消订阅函数列表 */
  const unlistenFns: (() => void)[] = [];

  /** 启动刷新（订阅所有后端事件） */
  async function startRefresh() {
    if (isConnected.value) return;

    try {
      // 行情增量更新
      const unlistenStock = await onStockUpdate((payload: StockQuote[]) => {
        quotes.value = payload;
      });
      unlistenFns.push(unlistenStock);

      // 调度器状态
      const unlistenScheduler = await onSchedulerStatus(
        (payload: SchedulerStatus) => {
          schedulerStatus.value = payload;
        }
      );
      unlistenFns.push(unlistenScheduler);

      // 涨跌家数摘要
      const unlistenSummary = await onMarketSummary(
        (payload: MarketSummary) => {
          marketSummary.value = payload;
        }
      );
      unlistenFns.push(unlistenSummary);

      // 异动检测
      const unlistenAnomaly = await onAnomalyDetected(
        (payload: unknown[]) => {
          anomalies.value = payload as AnomalyEvent[];
        }
      );
      unlistenFns.push(unlistenAnomaly);

      isConnected.value = true;
    } catch (e) {
      console.error("[useRefresh] 订阅事件失败:", e);
      isConnected.value = false;
    }
  }

  /** 停止刷新（取消所有订阅） */
  function stopRefresh() {
    for (const unlisten of unlistenFns) {
      unlisten();
    }
    unlistenFns.length = 0;
    isConnected.value = false;
    quotes.value = [];
    schedulerStatus.value = null;
    marketSummary.value = null;
    anomalies.value = [];
  }

  // 组件挂载时自动订阅，卸载时自动取消
  onMounted(() => {
    startRefresh();
  });

  onUnmounted(() => {
    stopRefresh();
  });

  return {
    isConnected,
    quotes,
    schedulerStatus,
    marketSummary,
    anomalies,
    startRefresh,
    stopRefresh,
  };
}
