<template>
  <div class="stock-table">
    <table>
      <thead>
        <tr>
          <th class="col-name" @click="sortBy('name')">
            名称 {{ sortIndicator('name') }}
          </th>
          <th class="col-price" @click="sortBy('price')">
            最新价 {{ sortIndicator('price') }}
          </th>
          <th class="col-change" @click="sortBy('changePercent')">
            涨跌幅 {{ sortIndicator('changePercent') }}
          </th>
          <th class="col-change" @click="sortBy('change')">
            涨跌额 {{ sortIndicator('change') }}
          </th>
          <th class="col-amount" @click="sortBy('amount')">
            成交额 {{ sortIndicator('amount') }}
          </th>
          <th class="col-turnover" @click="sortBy('turnoverRate')">
            换手率 {{ sortIndicator('turnoverRate') }}
          </th>
          <th class="col-ratio" @click="sortBy('volumeRatio')">
            量比 {{ sortIndicator('volumeRatio') }}
          </th>
          <th class="col-pe" @click="sortBy('peTtm')">
            PE(TTM) {{ sortIndicator('peTtm') }}
          </th>
          <th class="col-inflow" @click="sortBy('mainNetInflow')">
            主力净流入 {{ sortIndicator('mainNetInflow') }}
          </th>
          <th class="col-cap" @click="sortBy('totalMarketCap')">
            总市值 {{ sortIndicator('totalMarketCap') }}
          </th>
          <th class="col-high">最高</th>
          <th class="col-low">最低</th>
          <th class="col-open">开盘</th>
          <th class="col-prev">昨收</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="stock in sortedStocks"
          :key="stock.secid"
          class="stock-row"
          @click="$emit('select', stock)"
          @contextmenu.prevent="$emit('contextmenu', stock, $event)"
        >
          <td class="col-name">
            <div class="stock-name">
              {{ stock.name }}
              <LimitTag
                :is-limit-up="stock.isLimitUp"
                :is-limit-down="stock.isLimitDown"
                :is-near-limit-up="stock.isNearLimitUp"
                :seal-strength="stock.sealStrength"
              />
              <SuspendTag
                :is-suspended="stock.isSuspended"
                :is-temp-suspended="stock.isTempSuspended"
                :temp-suspend-reason="stock.tempSuspendReason"
                :temp-suspend-resume-time="stock.tempSuspendResumeTime"
              />
              <MarginTag :is-margin-target="stock.isMarginTarget" />
              <AnomalyBadge
                :change-speed="stock.changeSpeed"
                :change-percent="stock.changePercent"
              />
              <VolumeRatioNoteTag :note="stock.volumeRatioNote" />
            </div>
            <div class="stock-code num">{{ stock.code }}</div>
          </td>
          <td class="col-price num" :class="priceClass(stock)">
            {{ formatPrice(stock.price) }}
          </td>
          <td class="col-change num" :class="changeClass(stock)">
            {{ formatChangePercent(stock.changePercent) }}
          </td>
          <td class="col-change num" :class="priceClass(stock)">
            {{ formatChange(stock.change) }}
          </td>
          <td class="col-amount num">{{ formatAmount(stock.amount) }}</td>
          <td class="col-turnover num">{{ stock.turnoverRate.toFixed(2) }}%</td>
          <td class="col-ratio num">
            {{ stock.volumeRatio.toFixed(2) }}
          </td>
          <td class="col-pe num">{{ stock.peTtm > 0 ? stock.peTtm.toFixed(1) : '--' }}</td>
          <td class="col-inflow num" :class="inflowBgClass(stock)">
            {{ formatAmount(stock.mainNetInflow) }}
          </td>
          <td class="col-cap num">{{ formatMarketCap(stock.totalMarketCap) }}</td>
          <td class="num">{{ formatPrice(stock.high) }}</td>
          <td class="num">{{ formatPrice(stock.low) }}</td>
          <td class="num">{{ formatPrice(stock.open) }}</td>
          <td class="num">{{ formatPrice(stock.preClose) }}</td>
        </tr>
      </tbody>
    </table>
    <div v-if="sortedStocks.length === 0" class="empty-state">
      <p>暂无自选股，点击搜索添加</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import type { StockQuote } from "@/types/stock";
import {
  formatPrice,
  formatChangePercent,
  formatChange,
  formatAmount,
  formatMarketCap,
} from "@/lib/format";
import LimitTag from "./LimitTag.vue";
import SuspendTag from "./SuspendTag.vue";
import MarginTag from "./MarginTag.vue";
import AnomalyBadge from "./AnomalyBadge.vue";
import VolumeRatioNoteTag from "./VolumeRatioNoteTag.vue";

const props = defineProps<{
  stocks: StockQuote[];
}>();

defineEmits<{
  select: [stock: StockQuote];
  contextmenu: [stock: StockQuote, event: MouseEvent];
}>();

const sortKey = ref("changePercent");
const sortOrder = ref<"asc" | "desc">("desc");

function sortBy(key: string) {
  if (sortKey.value === key) {
    sortOrder.value = sortOrder.value === "asc" ? "desc" : "asc";
  } else {
    sortKey.value = key;
    sortOrder.value = "desc";
  }
}

function sortIndicator(key: string): string {
  if (sortKey.value !== key) return "";
  return sortOrder.value === "asc" ? "↑" : "↓";
}

const sortedStocks = computed(() => {
  const arr = [...props.stocks];
  const key = sortKey.value as keyof StockQuote;
  const order = sortOrder.value === "asc" ? 1 : -1;
  arr.sort((a, b) => {
    const va = a[key] ?? 0;
    const vb = b[key] ?? 0;
    if (typeof va === "number" && typeof vb === "number") {
      return (va - vb) * order;
    }
    return String(va).localeCompare(String(vb)) * order;
  });
  return arr;
});

function changeClass(stock: StockQuote) {
  if (stock.changePercent > 0) return "col-change-pos text-up";
  if (stock.changePercent < 0) return "col-change-neg text-down";
  return "text-flat";
}

function inflowBgClass(stock: StockQuote) {
  if (stock.mainNetInflow > 0) return "col-inflow-pos text-up";
  if (stock.mainNetInflow < 0) return "col-inflow-neg text-down";
  return "";
}
</script>

<style scoped>
.stock-table {
  width: 100%;
  overflow-x: auto;
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
  cursor: pointer;
  user-select: none;
  position: sticky;
  top: 0;
  background: var(--color-bg);
  z-index: 1;
  transition: color var(--duration-fast);
}
th:hover {
  color: var(--color-text-secondary);
}
th.sortable-asc::after {
  content: " ↑";
  color: var(--color-primary);
  font-size: 10px;
}
th.sortable-desc::after {
  content: " ↓";
  color: var(--color-primary);
  font-size: 10px;
}
.col-name {
  text-align: left;
  min-width: 120px;
}
td {
  padding: var(--spacing-sm) var(--spacing-md);
  text-align: right;
  border-bottom: 1px solid var(--color-border);
}
.stock-row {
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
  position: relative;
}
.stock-row:hover {
  background: var(--color-surface-hover);
  box-shadow: var(--shadow-md);
  transform: translateX(4px);
  z-index: 1;
}
.stock-row:active {
  transform: translateX(2px);
  box-shadow: var(--shadow-sm);
}
/* 涨跌幅背景色带 */
.col-change-pos {
  background: linear-gradient(90deg, var(--color-up-bg) 0%, transparent 70%);
}
.col-change-neg {
  background: linear-gradient(90deg, var(--color-down-bg) 0%, transparent 70%);
}
/* 主力净流入背景 */
.col-inflow-pos {
  background: linear-gradient(90deg, var(--color-up-bg) 0%, transparent 70%);
}
.col-inflow-neg {
  background: linear-gradient(90deg, var(--color-down-bg) 0%, transparent 70%);
}
.stock-name {
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
}
.stock-code {
  color: var(--color-text-tertiary);
  font-size: var(--font-size-xs);
  margin-top: 2px;
}
.empty-state {
  text-align: center;
  padding: var(--spacing-xl) 0;
  color: var(--color-text-tertiary);
}

/* 异动闪烁动画 */
@keyframes pulse-anomaly {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}
.stock-row.has-anomaly {
  animation: pulse-anomaly 2s infinite;
}
</style>
