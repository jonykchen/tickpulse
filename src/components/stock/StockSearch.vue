<template>
  <div class="stock-search">
    <div class="search-wrapper">
      <input
        v-model="keyword"
        class="search-input"
        type="text"
        placeholder="搜索股票代码/名称..."
        @input="onInput"
        @keydown.enter="doSearch"
      />
      <button class="search-btn" @click="doSearch">搜索</button>
    </div>
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
.search-wrapper {
  display: flex;
  gap: var(--spacing-sm);
}
.search-input {
  flex: 1;
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
  outline: none;
  transition: all var(--duration-fast) var(--ease-out);
}
.search-input:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-glow-primary);
}
.search-input::placeholder {
  color: var(--color-text-tertiary);
}
.search-btn {
  padding: var(--spacing-sm) var(--spacing-lg);
  background: var(--color-primary);
  border: none;
  border-radius: var(--radius-md);
  color: white;
  font-size: var(--font-size-md);
  cursor: pointer;
  transition: all var(--duration-fast);
}
.search-btn:hover {
  background: var(--color-primary-hover);
}
.search-results {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--glass-bg);
  backdrop-filter: var(--blur-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  max-height: 300px;
  overflow-y: auto;
  z-index: 100;
  box-shadow: var(--shadow-lg);
  animation: dropdown-enter var(--duration-fast) var(--ease-out);
}
@keyframes dropdown-enter {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
.search-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-md);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
  border-bottom: 1px solid var(--color-border);
}
.search-item:last-child {
  border-bottom: none;
}
.search-item:hover {
  background: var(--color-surface-hover);
  transform: translateX(4px);
}
.search-item__name {
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
}
.search-item__code {
  color: var(--color-text-tertiary);
  font-size: var(--font-size-sm);
}
</style>
