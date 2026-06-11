<template>
  <div class="stock-search">
    <input
      v-model="keyword"
      class="search-input"
      type="text"
      placeholder="搜索股票代码/名称"
      @input="onInput"
      @keydown.enter="doSearch"
    />
    <div v-if="results.length > 0" class="search-results">
      <div
        v-for="item in results"
        :key="item.secid"
        class="search-item"
        @click="addStock(item)"
      >
        <span class="search-item__name">{{ item.name }}</span>
        <span class="search-item__code num">{{ item.code }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { searchStock, addWatchlistStock } from "@/lib/tauri";

const keyword = ref("");
const results = ref<{ secid: string; code: string; name: string }[]>([]);

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

function onInput() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    doSearch();
  }, 200);
}

async function doSearch() {
  if (!keyword.value.trim()) {
    results.value = [];
    return;
  }
  try {
    results.value = await searchStock(keyword.value.trim());
  } catch {
    results.value = [];
  }
}

async function addStock(item: { secid: string; code: string; name: string }) {
  try {
    await addWatchlistStock(1, item.secid, item.name);
    results.value = [];
    keyword.value = "";
  } catch (e) {
    console.error("添加自选股失败:", e);
  }
}
</script>

<style scoped>
.stock-search {
  position: relative;
}
.search-input {
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface);
  border: 1px solid var(--color-bg-tertiary);
  border-radius: var(--radius-md);
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
  outline: none;
  transition: border-color var(--transition-fast);
}
.search-input:focus {
  border-color: var(--color-primary);
}
.search-input::placeholder {
  color: var(--color-text-tertiary);
}
.search-results {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--color-surface);
  border: 1px solid var(--color-bg-tertiary);
  border-radius: var(--radius-md);
  max-height: 300px;
  overflow-y: auto;
  z-index: 100;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}
.search-item {
  display: flex;
  justify-content: space-between;
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  transition: background var(--transition-fast);
}
.search-item:hover {
  background: var(--color-surface-hover);
}
.search-item__name {
  color: var(--color-text-primary);
}
.search-item__code {
  color: var(--color-text-tertiary);
}
</style>
