<template>
  <div class="dimension-card" :class="ratingClass">
    <div class="dim-header">
      <span class="dim-name">{{ DIMENSION_LABELS[report.dimension as AnalysisDimension] }}</span>
      <span class="dim-rating">{{ report.rating }}</span>
    </div>
    <p class="dim-summary">{{ report.summary }}</p>
    <div v-if="report.keyPoints.length" class="dim-section">
      <span class="dim-section-title">关键要点</span>
      <ul>
        <li v-for="(point, i) in report.keyPoints" :key="i">{{ point }}</li>
      </ul>
    </div>
    <div v-if="report.risks.length" class="dim-section risks">
      <span class="dim-section-title">风险</span>
      <ul>
        <li v-for="(risk, i) in report.risks" :key="i">⚠️ {{ risk }}</li>
      </ul>
    </div>
    <div v-if="report.opportunities.length" class="dim-section opportunities">
      <span class="dim-section-title">机会</span>
      <ul>
        <li v-for="(opp, i) in report.opportunities" :key="i">✅ {{ opp }}</li>
      </ul>
    </div>
    <div class="dim-confidence">
      置信度: {{ (report.confidence * 100).toFixed(0) }}%
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { DimensionReport } from "@/types/analysis";
import { AnalysisDimension, DIMENSION_LABELS } from "@/types/analysis";

const props = defineProps<{
  report: DimensionReport;
}>();

const ratingClass = computed(() => {
  switch (props.report.rating) {
    case "A": return "rating-a";
    case "B": return "rating-b";
    case "C": return "rating-c";
    case "D": return "rating-d";
    case "F": return "rating-f";
    default: return "";
  }
});
</script>

<style scoped>
.dimension-card {
  padding: 12px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
}
.rating-a { border-left: 3px solid #52c41a; }
.rating-b { border-left: 3px solid #1890ff; }
.rating-c { border-left: 3px solid #faad14; }
.rating-d { border-left: 3px solid #fa8c16; }
.rating-f { border-left: 3px solid #f5222d; }
.dim-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.dim-name {
  font-weight: 600;
  color: var(--color-text-primary);
}
.dim-rating {
  font-size: 18px;
  font-weight: 800;
}
.dim-summary {
  margin: 8px 0;
  font-size: 13px;
  color: var(--color-text-secondary);
}
.dim-section {
  margin-top: 8px;
}
.dim-section-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-tertiary);
}
.dim-section ul {
  margin: 4px 0;
  padding-left: 16px;
  font-size: 12px;
}
.risks li { color: #fa8c16; }
.opportunities li { color: #52c41a; }
.dim-confidence {
  margin-top: 8px;
  font-size: 10px;
  color: var(--color-text-tertiary);
  text-align: right;
}
</style>
