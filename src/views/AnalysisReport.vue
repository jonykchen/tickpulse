<template>
  <div class="analysis-report">
    <NEmpty v-if="!result" description="暂无分析报告" />
    <div v-else class="report-content">
      <!-- 综合评级 -->
      <div class="overall-section">
        <div class="overall-rating" :class="ratingClass">
          <span class="rating-label">{{ result.stockName }}</span>
          <span class="rating-value">{{ ratingDisplay }}</span>
          <span class="rating-score">{{ result.overallScore.toFixed(0) }}分</span>
        </div>
      </div>

      <!-- 多空辩论 -->
      <div class="debate-section">
        <div class="debate-card bull">
          <h4>🐂 多方观点</h4>
          <p>{{ result.bullArgument }}</p>
        </div>
        <div class="debate-card bear">
          <h4>🐻 空方观点</h4>
          <p>{{ result.bearArgument }}</p>
        </div>
      </div>

      <!-- 裁决 -->
      <div class="verdict">
        <h4>⚖️ 裁决</h4>
        <p>{{ result.verdict }}</p>
      </div>

      <!-- 维度卡片 -->
      <div class="dimensions-grid">
        <DimensionCard
          v-for="dim in dimensions"
          :key="dim.dimension"
          :report="dim"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { NEmpty } from "naive-ui";
import { useAnalysisStore } from "@/stores/analysis";
import { OverallRating, OVERALL_RATING_LABELS, AnalysisDimension } from "@/types/analysis";
import DimensionCard from "@/components/analysis/DimensionCard.vue";
import type { DimensionReport } from "@/types/analysis";

const analysisStore = useAnalysisStore();
const result = computed(() => {
  const results = analysisStore.latestResults;
  return results.length > 0 ? results[0] : null;
});

const ratingDisplay = computed(() => {
  if (!result.value) return "";
  return OVERALL_RATING_LABELS[result.value.overallRating as OverallRating] ?? "";
});

const ratingClass = computed(() => {
  if (!result.value) return "";
  const score = result.value.overallScore;
  if (score >= 60) return "rating-positive";
  if (score >= 40) return "rating-neutral";
  return "rating-negative";
});

const dimensions = computed<DimensionReport[]>(() => {
  if (!result.value) return [];
  return Object.values(result.value.dimensions);
});
</script>

<style scoped>
.analysis-report {
  padding: var(--spacing-md);
}
.overall-section {
  text-align: center;
  padding: var(--spacing-lg);
}
.overall-rating {
  display: inline-flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}
.rating-label {
  font-size: var(--font-size-lg);
  font-weight: 700;
}
.rating-value {
  font-size: 28px;
  font-weight: 800;
}
.rating-positive .rating-value { color: #f5222d; }
.rating-neutral .rating-value { color: #faad14; }
.rating-negative .rating-value { color: #52c41a; }
.rating-score {
  font-size: var(--font-size-sm);
  color: var(--color-text-tertiary);
}
.debate-section {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin: var(--spacing-md) 0;
}
.debate-card {
  padding: 12px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
}
.debate-card h4 {
  margin: 0 0 8px;
}
.bull {
  border-left: 3px solid #f5222d;
}
.bear {
  border-left: 3px solid #52c41a;
}
.verdict {
  padding: 12px;
  border-radius: 8px;
  background: var(--color-bg-secondary);
  margin-bottom: var(--spacing-md);
}
.verdict h4 {
  margin: 0 0 8px;
}
.dimensions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}
</style>
