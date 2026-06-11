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
import { ref, computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import StockSearch from "@/components/stock/StockSearch.vue";
import StockGroupTab from "@/components/stock/StockGroupTab.vue";
import StockTable from "@/components/stock/StockTable.vue";
import { useMarketStore } from "@/stores/market";
import { getWatchlistGroups, getWatchlistStocks } from "@/lib/tauri";
import type { StockQuote } from "@/types/stock";

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

onMounted(() => {
  loadGroups();
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