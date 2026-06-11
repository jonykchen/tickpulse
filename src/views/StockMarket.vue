<template>
  <div class="market-view">
    <div v-if="selectedStock" class="stock-detail">
      <div class="stock-detail__header">
        <h2>{{ selectedStock.name }}</h2>
        <span class="num" :class="priceClass">
          {{ formatPrice(selectedStock.price) }}
          {{ formatChangePercent(selectedStock.changePercent) }}
        </span>
      </div>

      <div class="stock-detail__charts">
        <div class="chart-tabs">
          <button
            v-for="tab in chartTabs"
            :key="tab.key"
            class="chart-tab"
            :class="{ 'chart-tab--active': activeChart === tab.key }"
            @click="activeChart = tab.key"
          >
            {{ tab.label }}
          </button>
        </div>

        <KlineChart
          v-if="activeChart === 'kline'"
          :data="klineData"
        />
        <TimelineChart
          v-if="activeChart === 'timeline'"
          :priceData="timelinePriceData"
          :avgData="timelineAvgData"
          :preClose="selectedStock.preClose"
        />
      </div>
    </div>

    <div v-else class="market-empty">
      <p>请选择一只股票查看详情</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRoute } from "vue-router";
import KlineChart from "@/components/charts/KlineChart.vue";
import TimelineChart from "@/components/charts/TimelineChart.vue";
import { useMarketStore } from "@/stores/market";
import { fetchKline, fetchTimeline } from "@/lib/tauri";
import { formatPrice, formatChangePercent } from "@/lib/format";
import type { StockQuote } from "@/types/stock";
import type { CandlestickData, LineData, Time } from "lightweight-charts";

const route = useRoute();
const marketStore = useMarketStore();

const selectedSecid = computed(() => route.query.secid as string || "");
const selectedStock = computed(() => {
  if (!selectedSecid.value) return null;
  return marketStore.quotes.get(selectedSecid.value) ?? null;
});

const activeChart = ref<"kline" | "timeline">("kline");
const klineData = ref<CandlestickData<Time>[]>([]);
const timelinePriceData = ref<LineData<Time>[]>([]);
const timelineAvgData = ref<LineData<Time>[]>([]);

const chartTabs = [
  { key: "kline", label: "K线" },
  { key: "timeline", label: "分时" },
];

const priceClass = computed(() => {
  if (!selectedStock.value) return "";
  if (selectedStock.value.changePercent > 0) return "text-up";
  if (selectedStock.value.changePercent < 0) return "text-down";
  return "text-flat";
});

async function loadKlineData() {
  if (!selectedSecid.value) return;
  try {
    const bars = await fetchKline(selectedSecid.value, "day", 120, "forward");
    klineData.value = bars.map((bar) => ({
      time: (bar.time as number) as Time,
      open: bar.open,
      high: bar.high,
      low: bar.low,
      close: bar.close,
    }));
  } catch (e) {
    console.error("K线数据加载失败:", e);
  }
}

async function loadTimelineData() {
  if (!selectedSecid.value) return;
  try {
    const data = await fetchTimeline(selectedSecid.value);
    timelinePriceData.value = data.points.map((p) => ({
      time: p.time as Time,
      value: p.price,
    }));
    timelineAvgData.value = data.points.map((p) => ({
      time: p.time as Time,
      value: p.avgPrice,
    }));
  } catch (e) {
    console.error("分时数据加载失败:", e);
  }
}

watch(selectedSecid, () => {
  loadKlineData();
  loadTimelineData();
});

onMounted(() => {
  loadKlineData();
  loadTimelineData();
});
</script>

<style scoped>
.market-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.stock-detail__header {
  display: flex;
  align-items: baseline;
  gap: var(--spacing-lg);
  padding: var(--spacing-md) var(--spacing-lg);
}
.stock-detail__header h2 {
  font-size: var(--font-size-xl);
}
.chart-tabs {
  display: flex;
  gap: var(--spacing-md);
  padding: var(--spacing-md) var(--spacing-lg);
}
.chart-tab {
  padding: var(--spacing-xs) var(--spacing-md);
  background: transparent;
  border: 1px solid var(--color-bg-tertiary);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: var(--font-size-sm);
  transition: all var(--transition-fast);
}
.chart-tab--active {
  color: var(--color-primary);
  border-color: var(--color-primary);
  background: rgba(52, 152, 219, 0.1);
}
.stock-detail__charts {
  flex: 1;
  overflow: hidden;
}
.market-empty {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  color: var(--color-text-tertiary);
}
</style>