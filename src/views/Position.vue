<template>
  <div class="position-view">
    <h2>持仓管理</h2>
    <PositionTable :positions="positions" :quotes="currentQuotes" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import PositionTable from "@/components/position/PositionTable.vue";
import { useMarketStore } from "@/stores/market";
import { getPositions } from "@/lib/tauri";
import type { StockQuote } from "@/types/stock";

const marketStore = useMarketStore();
const positions = ref<any[]>([]);
const currentQuotes = computed(() => Array.from(marketStore.quotes.values()));

async function loadPositions() {
  try {
    positions.value = await getPositions(1);
  } catch (e) {
    console.error("加载持仓失败:", e);
  }
}

onMounted(() => {
  loadPositions();
});
</script>

<style scoped>
.position-view {
  padding: var(--spacing-lg);
}
</style>