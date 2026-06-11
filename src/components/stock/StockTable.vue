<template>
  <div class="stock-table">
    <table>
      <thead>
        <tr>
          <th class="col-name" @click="sortBy('name')">
            名称 {{ sortKey === 'name' ? (sortOrder === 'asc' ? '↑' : '↓') : '' }}
          </th>
          <th class="col-price" @click="sortBy('price')">
            最新价 {{ sortKey === 'price' ? (sortOrder === 'asc' ? '↑' : '↓') : '' }}
          </th>
          <th class="col-change" @click="sortBy('changePercent')">
            涨跌幅 {{ sortKey === 'changePercent' ? (sortOrder === 'asc' ? '↑' : '↓') : '' }}
          </th>
          <th class="col-volume" @click="sortBy('amount')">
            成交额 {{ sortKey === 'amount' ? (sortOrder === 'asc' ? '↑' : '↓') : '' }}
          </th>
          <th class="col-turnover" @click="sortBy('turnoverRate')">
            换手率 {{ sortKey === 'turnoverRate' ? (sortOrder === 'asc' ? '↑' : '↓') : '' }}
          </th>
          <th class="col-ratio">量比</th>
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
            <div class="stock-name">{{ stock.name }}</div>
            <div class="stock-code num">{{ stock.code }}</div>
          </td>
          <td class="col-price num" :class="priceClass(stock)">
            {{ formatPrice(stock.price) }}
          </td>
          <td class="col-change num" :class="priceClass(stock)">
            <div>{{ formatChange(stock.changePercent) }}%</div>
            <div class="change-amount">{{ formatChange(stock.change) }}</div>
          </td>
          <td class="col-volume num">{{ formatAmount(stock.amount) }}</td>
          <td class="col-turnover num">{{ stock.turnoverRate.toFixed(2) }}%</td>
          <td class="col-ratio num">{{ stock.volumeRatio.toFixed(2) }}</td>
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

function priceClass(stock: StockQuote) {
  if (stock.changePercent > 0) return "text-up";
  if (stock.changePercent < 0) return "text-down";
  return "text-flat";
}

function formatPrice(price: number): string {
  return price > 0 ? price.toFixed(2) : "--";
}

function formatChange(val: number): string {
  if (val > 0) return `+${val.toFixed(2)}`;
  return val.toFixed(2);
}

function formatAmount(amount: number): string {
  if (amount >= 1e8) return (amount / 1e8).toFixed(2) + "亿";
  if (amount >= 1e4) return (amount / 1e4).toFixed(2) + "万";
  return amount.toFixed(0);
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
}
th:hover {
  color: var(--color-text-secondary);
}
.col-name {
  text-align: left;
  min-width: 100px;
}
td {
  padding: var(--spacing-sm) var(--spacing-md);
  text-align: right;
  border-bottom: 1px solid var(--color-bg-secondary);
}
.stock-row {
  cursor: pointer;
  transition: background var(--transition-fast);
}
.stock-row:hover {
  background: var(--color-surface-hover);
}
.stock-name {
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
}
.stock-code {
  color: var(--color-text-tertiary);
  font-size: var(--font-size-xs);
}
.change-amount {
  font-size: var(--font-size-xs);
  opacity: 0.7;
}
.empty-state {
  text-align: center;
  padding: var(--spacing-xl) 0;
  color: var(--color-text-tertiary);
}
</style>
