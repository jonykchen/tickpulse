<template>
  <span class="peg-rating-tag" :class="classMap">{{ label }}</span>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { PegRating, PEG_RATING_LABELS } from "@/types/analysis";

const props = defineProps<{
  rating: PegRating;
}>();

const label = computed(() => PEG_RATING_LABELS[props.rating] ?? "");

const classMap = computed(() => {
  switch (props.rating) {
    case PegRating.ExtremelyUndervalued:
    case PegRating.Undervalued:
      return "tag-undervalued";
    case PegRating.Fair:
      return "tag-fair";
    case PegRating.Overvalued:
    case PegRating.ExtremelyOvervalued:
      return "tag-overvalued";
  }
});
</script>

<style scoped>
.peg-rating-tag {
  display: inline-block;
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 600;
}
.tag-undervalued {
  background: rgba(82, 196, 26, 0.15);
  color: #52c41a;
}
.tag-fair {
  background: rgba(250, 173, 20, 0.15);
  color: #faad14;
}
.tag-overvalued {
  background: rgba(245, 34, 45, 0.15);
  color: #f5222d;
}
</style>
