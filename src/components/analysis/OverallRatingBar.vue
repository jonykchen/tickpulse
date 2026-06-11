<template>
  <div class="overall-rating-bar">
    <div class="bar-track">
      <div
        class="bar-fill"
        :style="{ width: `${score}%`, background: barColor }"
      />
    </div>
    <div class="bar-labels">
      <span class="bar-score">{{ score.toFixed(0) }}</span>
      <span class="bar-rating">{{ ratingDisplay }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { OverallRating, OVERALL_RATING_LABELS } from "@/types/analysis";

const props = defineProps<{
  score: number;
  rating: OverallRating;
}>();

const ratingDisplay = computed(() => OVERALL_RATING_LABELS[props.rating] ?? "");

const barColor = computed(() => {
  if (props.score >= 60) return "#f5222d";
  if (props.score >= 40) return "#faad14";
  return "#52c41a";
});
</script>

<style scoped>
.overall-rating-bar {
  width: 100%;
}
.bar-track {
  width: 100%;
  height: 6px;
  background: var(--color-bg-secondary);
  border-radius: 3px;
  overflow: hidden;
}
.bar-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s ease;
}
.bar-labels {
  display: flex;
  justify-content: space-between;
  margin-top: 4px;
}
.bar-score {
  font-size: 12px;
  font-weight: 700;
  color: var(--color-text-primary);
}
.bar-rating {
  font-size: 12px;
  color: var(--color-text-tertiary);
}
</style>
