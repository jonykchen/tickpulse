<template>
  <div class="all-top-market">
    <NTabs type="line" size="small" animated>
      <NTabPane
        v-for="(indices, key) in INDEX_CATEGORIES"
        :key="key"
        :name="key"
        :tab="tabLabels[key]"
      >
        <div class="index-grid">
          <div
            v-for="idx in indices"
            :key="idx.secid"
            class="index-card"
          >
            <div class="index-name">{{ idx.name }}</div>
            <div
              v-if="indexQuotes[idx.secid]"
              class="index-values"
            >
              <span
                class="index-price num"
                :class="getChangeColorClass(indexQuotes[idx.secid].changePercent)"
              >
                {{ formatPrice(indexQuotes[idx.secid].price) }}
              </span>
              <span
                class="index-change num"
                :class="getChangeColorClass(indexQuotes[idx.secid].changePercent)"
              >
                {{ formatChangePercent(indexQuotes[idx.secid].changePercent) }}
              </span>
            </div>
            <div v-else class="index-values">
              <span class="index-price num text-flat">--</span>
              <span class="index-change num text-flat">--</span>
            </div>
          </div>
        </div>
      </NTabPane>
    </NTabs>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { NTabs, NTabPane } from "naive-ui";
import { INDEX_CATEGORIES } from "@/lib/constants";
import { formatPrice, formatChangePercent, getChangeColorClass } from "@/lib/format";
import { fetchQuotes, onStockUpdate } from "@/lib/tauri";
import type { StockQuote } from "@/types/stock";

const tabLabels: Record<string, string> = {
  china: "中国指数",
  largeCap: "大盘指数",
  fifty: "50指数",
  hk: "港股指数",
  forex: "汇率期货",
  overseas: "海外指数",
};

const indexQuotes = ref<Record<string, StockQuote>>({});

/** 收集所有 secid */
function getAllSecids(): string[] {
  const secids: string[] = [];
  for (const indices of Object.values(INDEX_CATEGORIES)) {
    for (const idx of indices) {
      if (!secids.includes(idx.secid)) {
        secids.push(idx.secid);
      }
    }
  }
  return secids;
}

async function loadQuotes() {
  try {
    const secids = getAllSecids();
    const quotes = await fetchQuotes(secids);
    const map: Record<string, StockQuote> = {};
    for (const q of quotes) {
      map[q.secid] = q;
    }
    indexQuotes.value = map;
  } catch {
    // silently ignore fetch errors
  }
}

let unsubscribe: (() => void) | null = null;

onMounted(async () => {
  await loadQuotes();
  unsubscribe = await onStockUpdate((quotes) => {
    for (const q of quotes) {
      if (indexQuotes.value[q.secid]) {
        indexQuotes.value[q.secid] = q;
      }
    }
  });
});

onUnmounted(() => {
  unsubscribe?.();
});
</script>

<style scoped>
.all-top-market {
  padding: var(--spacing-md);
}

.index-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: var(--spacing-md);
}

.index-card {
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  transition: background var(--transition-fast);
}

.index-card:hover {
  background: var(--color-surface-hover);
}

.index-name {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-xs);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.index-values {
  display: flex;
  align-items: baseline;
  gap: var(--spacing-sm);
}

.index-price {
  font-size: var(--font-size-lg);
  font-weight: 600;
}

.index-change {
  font-size: var(--font-size-xs);
}
</style>
