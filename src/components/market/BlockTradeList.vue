<template>
  <div class="block-trade-list">
    <div class="toolbar">
      <NDatePicker
        v-model:formatted-value="selectedDate"
        type="date"
        size="small"
        clearable
        value-format="yyyy-MM-dd"
        :style="{ width: '160px' }"
      />
    </div>
    <table>
      <thead>
        <tr>
          <th class="col-name">名称</th>
          <th class="col-price">成交价</th>
          <th class="col-volume">成交量</th>
          <th class="col-amount">成交额</th>
          <th class="col-buyer">买方</th>
          <th class="col-seller">卖方</th>
          <th class="col-premium">折溢率</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="trade in trades" :key="trade.secid + trade.tradeDate">
          <td class="col-name">
            <div class="trade-name">{{ trade.name }}</div>
            <div class="trade-code num">{{ trade.code }}</div>
          </td>
          <td class="col-price num">{{ formatPrice(trade.price) }}</td>
          <td class="col-volume num">{{ formatVolume(trade.volume) }}</td>
          <td class="col-amount num">{{ formatAmount(trade.amount) }}</td>
          <td class="col-buyer">{{ trade.buyer ?? "--" }}</td>
          <td class="col-seller">{{ trade.seller ?? "--" }}</td>
          <td class="col-premium num" :class="premiumClass(trade.premiumRate)">
            {{ formatPremiumRate(trade.premiumRate) }}
          </td>
        </tr>
        <tr v-if="trades.length === 0">
          <td colspan="7" class="empty-state">暂无大宗交易数据</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { NDatePicker } from "naive-ui";
import { fetchBlockTrades } from "@/lib/tauri";
import { formatPrice, formatAmount } from "@/lib/format";
import type { BlockTrade } from "@/types/portfolio";

/** Expose premium rate colors for v-bind in CSS */
const premiumPositiveColor = "#f5222d";
const premiumNegativeColor = "#52c41a";

const selectedDate = ref<string | null>(null);
const trades = ref<BlockTrade[]>([]);

function formatVolume(volume: number): string {
  if (volume >= 1e4) return (volume / 1e4).toFixed(2) + "万手";
  return volume + "手";
}

function formatPremiumRate(rate: number | null): string {
  if (rate === null || rate === undefined) return "--";
  const prefix = rate > 0 ? "+" : "";
  return `${prefix}${rate.toFixed(2)}%`;
}

function premiumClass(rate: number | null): string {
  if (rate === null || rate === undefined) return "text-flat";
  if (rate > 0) return "premium-positive";
  if (rate < 0) return "premium-negative";
  return "text-flat";
}

async function loadTrades() {
  try {
    trades.value = await fetchBlockTrades(selectedDate.value ?? undefined);
  } catch {
    trades.value = [];
  }
}

watch(selectedDate, () => {
  loadTrades();
});

// Load on mount
loadTrades();
</script>

<style scoped>
.block-trade-list {
  padding: var(--spacing-md);
}

.toolbar {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--font-size-sm);
}

th {
  padding: var(--spacing-sm) var(--spacing-md);
  text-align: right;
  color: var(--color-text-tertiary);
  font-weight: 500;
  white-space: nowrap;
}

th:first-child,
td:first-child {
  text-align: left;
}

td {
  padding: var(--spacing-sm) var(--spacing-md);
  text-align: right;
  border-bottom: 1px solid var(--color-bg-secondary);
}

.trade-name {
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
}

.trade-code {
  color: var(--color-text-tertiary);
  font-size: var(--font-size-xs);
}

.col-buyer,
.col-seller {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-text-secondary);
  font-size: var(--font-size-xs);
}

.premium-positive {
  color: v-bind(premiumPositiveColor);
}

.premium-negative {
  color: v-bind(premiumNegativeColor);
}

.empty-state {
  text-align: center;
  padding: var(--spacing-xl) 0;
  color: var(--color-text-tertiary);
}
</style>
