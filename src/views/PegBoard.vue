<template>
  <div class="peg-board">
    <NEmpty v-if="pegList.length === 0" description="暂无 PEG 数据，请先执行分析" />
    <div v-else class="peg-grid">
      <div v-for="peg in pegList" :key="peg.secid" class="peg-card" :class="pegClass(peg.pegRating)">
        <div class="peg-header">
          <span class="peg-name">{{ peg.stockName }}</span>
          <PegRatingTag :rating="peg.pegRating" />
        </div>
        <div class="peg-values">
          <div class="peg-item">
            <span class="peg-label">PE(TTM)</span>
            <span class="peg-value">{{ peg.peTtm.toFixed(1) }}</span>
          </div>
          <div class="peg-item">
            <span class="peg-label">CAGR</span>
            <span class="peg-value">{{ peg.cagr.toFixed(1) }}%</span>
          </div>
          <div class="peg-item">
            <span class="peg-label">PEG</span>
            <span class="peg-value highlight">{{ peg.pegValue.toFixed(2) }}</span>
          </div>
          <div v-if="peg.industryPeg" class="peg-item">
            <span class="peg-label">行业PEG</span>
            <span class="peg-value">{{ peg.industryPeg.toFixed(2) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { NEmpty } from "naive-ui";
import { useAnalysisStore } from "@/stores/analysis";
import { PegRating, PEG_RATING_LABELS } from "@/types/analysis";
import PegRatingTag from "@/components/analysis/PegRatingTag.vue";

const analysisStore = useAnalysisStore();
const pegList = computed(() => Array.from(analysisStore.pegCache.values()));

function pegClass(rating: PegRating): string {
  switch (rating) {
    case PegRating.ExtremelyUndervalued:
    case PegRating.Undervalued:
      return "peg-undervalued";
    case PegRating.Fair:
      return "peg-fair";
    case PegRating.Overvalued:
    case PegRating.ExtremelyOvervalued:
      return "peg-overvalued";
  }
}
</script>

<style scoped>
.peg-board {
  padding: var(--spacing-md);
}
.peg-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
}
.peg-card {
  padding: 12px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
}
.peg-undervalued {
  border-left: 3px solid #52c41a;
}
.peg-fair {
  border-left: 3px solid #faad14;
}
.peg-overvalued {
  border-left: 3px solid #f5222d;
}
.peg-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
.peg-name {
  font-weight: 600;
  color: var(--color-text-primary);
}
.peg-values {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}
.peg-item {
  display: flex;
  flex-direction: column;
}
.peg-label {
  font-size: 10px;
  color: var(--color-text-tertiary);
}
.peg-value {
  font-size: 14px;
  color: var(--color-text-secondary);
}
.highlight {
  font-weight: 700;
  color: var(--color-text-primary);
}
</style>
