<template>
  <div class="industry-peg-comparison">
    <!-- 加载中 -->
    <NSpin v-if="loading" :description="loadingText" />

    <!-- 错误状态 -->
    <NAlert v-else-if="error" type="error" :title="error">
      <NButton size="small" @click="fetchData">重试</NButton>
    </NAlert>

    <!-- 主内容 -->
    <template v-else-if="stockInfo">
      <!-- 顶部：股票基本信息 -->
      <div class="stock-header">
        <div class="stock-basic">
          <span class="stock-name">{{ stockInfo.name }}</span>
          <span class="stock-code">{{ stockInfo.code }}</span>
          <NTag v-if="stockInfo.industryName" size="small" type="info">
            {{ stockInfo.industryName }}
          </NTag>
        </div>
        <div class="stock-metrics">
          <div class="metric-item">
            <span class="metric-label">当前价</span>
            <span class="metric-value" :class="priceClass">{{ formatPrice(stockInfo.price) }}</span>
            <span class="metric-change" :class="changeClass">
              {{ formatChangePercent(stockInfo.changePercent) }}
            </span>
          </div>
          <div class="metric-item">
            <span class="metric-label">PE(TTM)</span>
            <span class="metric-value">{{ stockInfo.peTtm?.toFixed(1) ?? '--' }}</span>
          </div>
          <div class="metric-item">
            <span class="metric-label">PB</span>
            <span class="metric-value">{{ stockInfo.pb?.toFixed(2) ?? '--' }}</span>
          </div>
          <div class="metric-item">
            <span class="metric-label">PEG</span>
            <span class="metric-value highlight">{{ stockInfo.pegValue?.toFixed(2) ?? '--' }}</span>
            <PegRatingTag v-if="stockInfo.pegRating" :rating="stockInfo.pegRating" />
          </div>
        </div>
      </div>

      <!-- 中部：行业 PE 分布图表 -->
      <div class="chart-section">
        <div class="section-header">
          <h3>行业估值对比</h3>
          <div class="legend">
            <span class="legend-item">
              <span class="legend-bar stock"></span> 个股 PE
            </span>
            <span class="legend-item">
              <span class="legend-line"></span> 行业均值
            </span>
          </div>
        </div>
        <IndustryPegChart v-if="chartData.length > 0" :data="chartData" />
        <NEmpty v-else description="暂无图表数据" />
      </div>

      <!-- 行业统计摘要 -->
      <div class="industry-summary">
        <div class="summary-card">
          <span class="summary-label">行业平均 PE</span>
          <span class="summary-value">{{ industryStats.avgPe?.toFixed(1) ?? '--' }}</span>
        </div>
        <div class="summary-card">
          <span class="summary-label">行业中位数 PE</span>
          <span class="summary-value">{{ industryStats.medianPe?.toFixed(1) ?? '--' }}</span>
        </div>
        <div class="summary-card">
          <span class="summary-label">行业平均 PB</span>
          <span class="summary-value">{{ industryStats.avgPb?.toFixed(2) ?? '--' }}</span>
        </div>
        <div class="summary-card">
          <span class="summary-label">估值溢价/折价</span>
          <span class="summary-value" :class="premiumClass">
            {{ formatPremium(stockInfo.premiumDiscount) }}
          </span>
        </div>
      </div>

      <!-- 底部：同行业股票列表 -->
      <div class="table-section">
        <h3>同行业股票估值对比</h3>
        <NDataTable
          :columns="tableColumns"
          :data="industryStocks"
          :row-class-name="rowClassName"
          :pagination="false"
          size="small"
          :bordered="false"
        />
      </div>
    </template>

    <!-- 空状态 -->
    <NEmpty v-else description="请通过股票详情页进入行业对比" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, h } from "vue";
import { useRoute } from "vue-router";
import {
  NSpin,
  NAlert,
  NButton,
  NTag,
  NEmpty,
  NDataTable,
  type DataTableColumns,
} from "naive-ui";
import { useAnalysisStore, type IndustryComparisonItem } from "@/stores/analysis";
import { PegRating, type PegRating as PegRatingType } from "@/types/analysis";
import PegRatingTag from "@/components/analysis/PegRatingTag.vue";
import IndustryPegChart from "@/components/charts/IndustryPegChart.vue";
import { formatPrice, formatChangePercent, getChangeColorClass } from "@/lib/format";

// ==================== Types ====================

interface StockInfo {
  secid: string;
  code: string;
  name: string;
  price: number;
  changePercent: number;
  peTtm: number | null;
  pb: number | null;
  pegValue: number | null;
  pegRating: PegRatingType | null;
  industryName: string | null;
  premiumDiscount: number | null;
}

interface IndustryStock {
  secid: string;
  name: string;
  price: number;
  changePercent: number;
  peTtm: number | null;
  pb: number | null;
  pegValue: number | null;
  pegRating: PegRatingType | null;
  isCurrentStock: boolean;
}

// ==================== Setup ====================

const route = useRoute();
const analysisStore = useAnalysisStore();

// ==================== State ====================

const loading = ref(false);
const loadingText = ref("加载中...");
const error = ref<string | null>(null);
const stockInfo = ref<StockInfo | null>(null);
const industryStocks = ref<IndustryStock[]>([]);
const industryStats = ref({
  avgPe: null as number | null,
  medianPe: null as number | null,
  avgPb: null as number | null,
});

// ==================== Computed ====================

const secid = computed(() => {
  const s = route.query.secid as string;
  return s || null;
});

const priceClass = computed(() => getChangeColorClass(stockInfo.value?.changePercent ?? 0));
const changeClass = computed(() => getChangeColorClass(stockInfo.value?.changePercent ?? 0));

const premiumClass = computed(() => {
  const val = stockInfo.value?.premiumDiscount;
  if (val === null || val === undefined) return "";
  return val > 0 ? "text-up" : val < 0 ? "text-down" : "";
});

const chartData = computed(() => {
  if (industryStocks.value.length === 0) return [];

  // 取前 20 只股票用于图表展示
  const displayStocks = industryStocks.value.slice(0, 20);

  // 计算行业 PEG 均值
  const industryPeg = industryStats.value.avgPe
    ? industryStats.value.avgPe * 0.5 // 假设行业平均增长率约 20%，则行业 PEG ≈ avgPe * 0.5
    : null;

  return displayStocks.map((stock) => ({
    name: stock.name.length > 4 ? stock.name.slice(0, 4) : stock.name,
    peg: stock.pegValue ?? 0,
    industryPeg: industryPeg ?? 0,
  }));
});

// ==================== Table Columns ====================

const tableColumns = computed<DataTableColumns<IndustryStock>>(() => [
  {
    title: "股票名称",
    key: "name",
    width: 100,
    render(row) {
      return h("div", { class: "stock-name-cell" }, [
        h("span", { class: "name" }, row.name),
        row.isCurrentStock ? h(NTag, { size: "small", type: "success" }, () => "当前") : null,
      ]);
    },
  },
  {
    title: "代码",
    key: "secid",
    width: 90,
    render(row) {
      // 从 secid 提取代码 (如 "0.000001" -> "000001")
      const code = row.secid.split(".")[1] || row.secid;
      return code;
    },
  },
  {
    title: "现价",
    key: "price",
    width: 80,
    render(row) {
      return h("span", { class: getChangeColorClass(row.changePercent) }, formatPrice(row.price));
    },
  },
  {
    title: "涨跌幅",
    key: "changePercent",
    width: 90,
    render(row) {
      return h("span", { class: getChangeColorClass(row.changePercent) }, formatChangePercent(row.changePercent));
    },
  },
  {
    title: "PE(TTM)",
    key: "peTtm",
    width: 80,
    render(row) {
      return row.peTtm !== null ? row.peTtm.toFixed(1) : "--";
    },
  },
  {
    title: "PB",
    key: "pb",
    width: 70,
    render(row) {
      return row.pb !== null ? row.pb.toFixed(2) : "--";
    },
  },
  {
    title: "PEG",
    key: "pegValue",
    width: 70,
    render(row) {
      if (row.pegValue === null) return "--";
      return h("span", { class: "peg-highlight" }, row.pegValue.toFixed(2));
    },
  },
  {
    title: "评级",
    key: "pegRating",
    width: 70,
    render(row) {
      if (!row.pegRating) return "--";
      return h(PegRatingTag, { rating: row.pegRating });
    },
  },
]);

// ==================== Methods ====================

function rowClassName(row: IndustryStock): string {
  return row.isCurrentStock ? "current-stock-row" : "";
}

function formatPremium(val: number | null): string {
  if (val === null) return "--";
  const prefix = val > 0 ? "+" : "";
  return `${prefix}${val.toFixed(1)}%`;
}

async function fetchData() {
  if (!secid.value) {
    error.value = "缺少股票代码参数";
    return;
  }

  loading.value = true;
  loadingText.value = "加载行业对比数据...";
  error.value = null;

  try {
    // 尝试调用 Tauri Command
    const data = await analysisStore.fetchIndustryComparison(secid.value);

    if (data && data.length > 0) {
      processRealData(data);
    } else {
      // 使用模拟数据展示
      loadingText.value = "使用模拟数据展示...";
      useMockData();
    }
  } catch (e) {
    console.warn("调用 fetch_industry_comparison 失败，使用模拟数据:", e);
    useMockData();
  } finally {
    loading.value = false;
  }
}

function processRealData(data: IndustryComparisonItem[]) {
  // 处理真实数据
  const current = data.find((item) => item.stockPe !== null);

  if (current) {
    stockInfo.value = {
      secid: secid.value!,
      code: secid.value!.split(".")[1] || secid.value!,
      name: current.industryName || "未知股票",
      price: 0,
      changePercent: 0,
      peTtm: current.stockPe,
      pb: current.stockPb,
      pegValue: null,
      pegRating: null,
      industryName: current.industryName,
      premiumDiscount: current.premiumDiscount,
    };

    industryStats.value = {
      avgPe: current.avgPe,
      medianPe: current.medianPe,
      avgPb: current.avgPb,
    };
  }

  industryStocks.value = data.map((item, index) => ({
    secid: `mock.${index}`,
    name: item.industryName || `股票${index + 1}`,
    price: 0,
    changePercent: 0,
    peTtm: item.avgPe,
    pb: item.avgPb,
    pegValue: null,
    pegRating: null,
    isCurrentStock: item.stockPe !== null,
  }));
}

function useMockData() {
  // 模拟股票基本信息
  const mockSecid = secid.value || "0.000001";
  const code = mockSecid.split(".")[1] || "000001";

  stockInfo.value = {
    secid: mockSecid,
    code,
    name: "平安银行",
    price: 12.35,
    changePercent: 2.15,
    peTtm: 5.8,
    pb: 0.65,
    pegValue: 0.58,
    pegRating: PegRating.Undervalued,
    industryName: "银行",
    premiumDiscount: -15.2,
  };

  // 模拟行业统计数据
  industryStats.value = {
    avgPe: 6.8,
    medianPe: 6.2,
    avgPb: 0.72,
  };

  // 模拟同行业股票列表
  const mockStocks: IndustryStock[] = [
    { secid: "1.600000", name: "浦发银行", price: 8.92, changePercent: 1.25, peTtm: 4.5, pb: 0.48, pegValue: 0.45, pegRating: PegRating.Undervalued, isCurrentStock: false },
    { secid: "1.600016", name: "民生银行", price: 4.15, changePercent: -0.48, peTtm: 4.2, pb: 0.42, pegValue: 0.42, pegRating: PegRating.ExtremelyUndervalued, isCurrentStock: false },
    { secid: "1.600036", name: "招商银行", price: 35.68, changePercent: 0.85, peTtm: 6.2, pb: 1.05, pegValue: 0.62, pegRating: PegRating.Undervalued, isCurrentStock: false },
    { secid: "0.000001", name: "平安银行", price: 12.35, changePercent: 2.15, peTtm: 5.8, pb: 0.65, pegValue: 0.58, pegRating: PegRating.Undervalued, isCurrentStock: true },
    { secid: "0.002142", name: "宁波银行", price: 22.45, changePercent: 1.68, peTtm: 7.1, pb: 1.12, pegValue: 0.71, pegRating: PegRating.Fair, isCurrentStock: false },
    { secid: "1.601166", name: "兴业银行", price: 18.92, changePercent: 0.52, peTtm: 5.2, pb: 0.58, pegValue: 0.52, pegRating: PegRating.Undervalued, isCurrentStock: false },
    { secid: "1.601398", name: "工商银行", price: 5.28, changePercent: 0.38, peTtm: 5.0, pb: 0.55, pegValue: 0.50, pegRating: PegRating.Undervalued, isCurrentStock: false },
    { secid: "1.601939", name: "建设银行", price: 6.85, changePercent: 0.29, peTtm: 5.2, pb: 0.58, pegValue: 0.52, pegRating: PegRating.Undervalued, isCurrentStock: false },
    { secid: "1.601288", name: "农业银行", price: 4.12, changePercent: 0.24, peTtm: 4.8, pb: 0.52, pegValue: 0.48, pegRating: PegRating.Undervalued, isCurrentStock: false },
    { secid: "1.601328", name: "交通银行", price: 6.25, changePercent: 0.64, peTtm: 4.6, pb: 0.48, pegValue: 0.46, pegRating: PegRating.Undervalued, isCurrentStock: false },
    { secid: "0.000002", name: "万科A", price: 8.52, changePercent: -1.25, peTtm: 8.5, pb: 0.85, pegValue: 0.85, pegRating: PegRating.Fair, isCurrentStock: false },
    { secid: "1.600030", name: "中信证券", price: 18.65, changePercent: 1.82, peTtm: 12.5, pb: 1.25, pegValue: 1.25, pegRating: PegRating.Overvalued, isCurrentStock: false },
  ];

  // 按 PE 排序
  industryStocks.value = mockStocks.sort((a, b) => (a.peTtm ?? 999) - (b.peTtm ?? 999));
}

// ==================== Lifecycle ====================

onMounted(() => {
  fetchData();
});
</script>

<style scoped>
.industry-peg-comparison {
  padding: var(--spacing-md);
  min-height: 100vh;
}

/* 股票头部信息 */
.stock-header {
  padding: 16px;
  border-radius: 12px;
  background: var(--color-bg-secondary);
  margin-bottom: 16px;
}

.stock-basic {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.stock-name {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.stock-code {
  font-size: 14px;
  color: var(--color-text-tertiary);
}

.stock-metrics {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
}

.metric-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.metric-label {
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.metric-value {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.metric-value.highlight {
  color: #1890ff;
}

.metric-change {
  font-size: 12px;
}

/* 图表区域 */
.chart-section {
  padding: 16px;
  border-radius: 12px;
  background: var(--color-bg-secondary);
  margin-bottom: 16px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.section-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.legend {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.legend-bar {
  width: 12px;
  height: 8px;
  border-radius: 2px;
}

.legend-bar.stock {
  background: #1890ff;
}

.legend-line {
  width: 16px;
  height: 2px;
  background: #faad14;
  border-style: dashed;
}

/* 行业统计摘要 */
.industry-summary {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}

@media (max-width: 768px) {
  .industry-summary {
    grid-template-columns: repeat(2, 1fr);
  }
}

.summary-card {
  padding: 12px;
  border-radius: 8px;
  background: var(--color-bg-secondary);
  text-align: center;
}

.summary-label {
  display: block;
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-bottom: 4px;
}

.summary-value {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text-primary);
}

/* 表格区域 */
.table-section {
  padding: 16px;
  border-radius: 12px;
  background: var(--color-bg-secondary);
}

.table-section h3 {
  margin: 0 0 12px;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
}

/* 表格行样式 */
:deep(.current-stock-row) {
  background: rgba(24, 144, 255, 0.1) !important;
}

:deep(.current-stock-row:hover) {
  background: rgba(24, 144, 255, 0.15) !important;
}

.stock-name-cell {
  display: flex;
  align-items: center;
  gap: 4px;
}

.stock-name-cell .name {
  font-weight: 500;
}

.peg-highlight {
  font-weight: 600;
  color: #1890ff;
}

/* 涨跌颜色 */
.text-up {
  color: #f5222d;
}

.text-down {
  color: #52c41a;
}

.text-flat {
  color: var(--color-text-secondary);
}
</style>
