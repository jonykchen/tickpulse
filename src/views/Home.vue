<template>
  <div class="home-view">
    <div class="home-header">
      <StockSearch />
      <StockGroupTab
        :groups="groups"
        :active-group="activeGroupId"
        @select="switchGroup"
      />
    </div>
    <StockTable
      :stocks="currentStocks"
      @select="openStockDetail"
      @contextmenu="showContextMenu"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import StockSearch from "@/components/stock/StockSearch.vue";
import StockGroupTab from "@/components/stock/StockGroupTab.vue";
import StockTable from "@/components/stock/StockTable.vue";
import { useMarketStore } from "@/stores/market";
import { getWatchlistGroups, getWatchlistStocks } from "@/lib/tauri";
import type { StockQuote } from "@/types/stock";

// 类型定义
interface SchedulerStatus {
  status: string;
  nextRefresh: number;
}

interface MarketSummary {
  upCount: number;
  downCount: number;
  flatCount: number;
  limitUpCount: number;
  limitDownCount: number;
}

interface AnomalyEvent {
  secid: string;
  name: string;
  type: string;
  value: number;
  threshold: number;
  timestamp: number;
}

interface AlertTriggered {
  alertId: number;
  secid: string;
  name: string;
  condition: string;
  currentValue: number;
  triggerTime: number;
}

// 事件监听器清理
const unlisteners: UnlistenFn[] = [];

// 响应式状态
const schedulerStatus = ref<SchedulerStatus | null>(null);
const marketSummary = ref<MarketSummary | null>(null);

const router = useRouter();
const marketStore = useMarketStore();

const groups = ref<{ id: number; name: string; sortOrder: number }[]>([]);
const activeGroupId = ref(1);

const currentStocks = computed(() => {
  return Array.from(marketStore.quotes.values());
});

async function loadGroups() {
  try {
    groups.value = await getWatchlistGroups();
    if (groups.value.length > 0) {
      activeGroupId.value = groups.value[0].id;
    }
  } catch (e) {
    console.error("加载分组失败:", e);
  }
}

function switchGroup(groupId: number) {
  activeGroupId.value = groupId;
}

function openStockDetail(stock: StockQuote) {
  router.push(`/market?secid=${stock.secid}`);
}

function showContextMenu(_stock: StockQuote, _event: MouseEvent) {
  // TODO: 右键菜单功能
}

// 更新本地行情缓存
function updateLocalQuote(secid: string, quote: StockQuote) {
  marketStore.quotes.set(secid, quote);
}

// 显示异动提示
function showAnomalyNotification(event: AnomalyEvent) {
  console.log("异动检测:", event);
  // TODO: 集成通知系统
}

// 显示预警通知
function showAlertNotification(alert: AlertTriggered) {
  console.log("预警触发:", alert);
  // TODO: 集成通知系统
}

onMounted(async () => {
  loadGroups();

  // Tauri 事件监听
  if (typeof window !== "undefined" && "__TAURI__" in window) {
    // 行情更新事件
    const unlisten1 = await listen<{ secid: string; quote: StockQuote }>("stock-update", (event) => {
      updateLocalQuote(event.payload.secid, event.payload.quote);
    });
    unlisteners.push(unlisten1);

    // 调度器状态事件
    const unlisten2 = await listen<SchedulerStatus>("scheduler-status", (event) => {
      schedulerStatus.value = event.payload;
    });
    unlisteners.push(unlisten2);

    // 市场概况事件
    const unlisten3 = await listen<MarketSummary>("market-summary", (event) => {
      marketSummary.value = event.payload;
    });
    unlisteners.push(unlisten3);

    // 异动检测事件
    const unlisten4 = await listen<AnomalyEvent>("anomaly-detected", (event) => {
      showAnomalyNotification(event.payload);
    });
    unlisteners.push(unlisten4);

    // 预警触发事件
    const unlisten5 = await listen<AlertTriggered>("alert-triggered", (event) => {
      showAlertNotification(event.payload);
    });
    unlisteners.push(unlisten5);
  }
});

onUnmounted(() => {
  unlisteners.forEach((unlisten) => unlisten());
});
</script>

<style scoped>
.home-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.home-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md) var(--spacing-lg);
}
</style>