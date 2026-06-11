<template>
  <div class="suspend-window">
    <div class="suspend-header" @mousedown="startDrag">
      <span class="index-value" :class="indexClass">
        {{ indexPrice }} {{ indexChange }}
      </span>
      <span class="index-name">上证指数</span>
    </div>
    <div class="suspend-list">
      <div
        v-for="stock in displayStocks"
        :key="stock.secid"
        class="suspend-item"
        :class="priceClass(stock)"
      >
        <span class="suspend-name">{{ stock.name }}</span>
        <span class="suspend-price">{{ formatPrice(stock.price) }}</span>
        <span class="suspend-change">{{ formatChangePercent(stock.changePercent) }}</span>
      </div>
      <div v-if="displayStocks.length === 0" class="suspend-empty">
        暂无自选股
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { StockQuote } from "@/types/stock";
import { useMarketStore } from "@/stores/market";
import { formatPrice, formatChangePercent } from "@/lib/format";
import { getCurrentWindow } from "@tauri-apps/api/window";

const marketStore = useMarketStore();

// 上证指数
const shIndex = computed(() =>
  marketStore.quotes.get("1.000001")
);
const indexPrice = computed(() =>
  shIndex.value ? formatPrice(shIndex.value.price) : "--"
);
const indexChange = computed(() =>
  shIndex.value ? formatChangePercent(shIndex.value.changePercent) : ""
);
const indexClass = computed(() => {
  if (!shIndex.value) return "";
  return shIndex.value.changePercent > 0 ? "text-up" : shIndex.value.changePercent < 0 ? "text-down" : "";
});

// 显示前5只自选股
const displayStocks = computed(() => {
  return marketStore.sortedQuotes.slice(0, 5);
});

function priceClass(stock: StockQuote) {
  if (stock.changePercent > 0) return "text-up";
  if (stock.changePercent < 0) return "text-down";
  return "text-flat";
}

// 拖拽逻辑
async function startDrag(_e: MouseEvent) {
  try {
    const appWindow = getCurrentWindow();
    await appWindow.startDragging();
  } catch {
    // 忽略拖拽错误
  }
}
</script>

<style scoped>
.suspend-window {
  width: 100%;
  height: 100%;
  background: var(--color-bg);
  border-radius: 8px;
  overflow: hidden;
  user-select: none;
}
.suspend-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--color-bg-secondary);
  cursor: move;
}
.index-value {
  font-size: 14px;
  font-weight: 700;
}
.index-name {
  font-size: 10px;
  color: var(--color-text-tertiary);
}
.suspend-list {
  padding: 4px 0;
}
.suspend-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  font-size: 12px;
}
.suspend-item:hover {
  background: var(--color-surface-hover);
}
.suspend-name {
  flex: 1;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.suspend-price {
  margin: 0 8px;
  font-weight: 600;
}
.suspend-change {
  min-width: 60px;
  text-align: right;
}
.suspend-empty {
  text-align: center;
  padding: 20px;
  color: var(--color-text-tertiary);
  font-size: 12px;
}
</style>
