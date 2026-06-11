<template>
  <span v-if="label" class="limit-tag" :class="limitClass" :style="sealStyle">
    {{ label }}
  </span>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  /** 是否涨停（后端计算） */
  isLimitUp: boolean;
  /** 是否跌停（后端计算） */
  isLimitDown: boolean;
  /** 是否接近涨停 */
  isNearLimitUp?: boolean;
  /** 封板强度 0-1（排除午休的有效封板率） */
  sealStrength?: number | null;
}>();

const label = computed(() => {
  if (props.isLimitUp) return "涨停";
  if (props.isLimitDown) return "跌停";
  if (props.isNearLimitUp) return "接近涨停";
  return "";
});

const limitClass = computed(() => {
  if (props.isLimitUp || props.isNearLimitUp) return "limit-up";
  if (props.isLimitDown) return "limit-down";
  return "";
});

/** 封板强度色条：深红=强封板，浅红+闪烁=弱封板 */
const sealStyle = computed(() => {
  if (props.isLimitUp && props.sealStrength != null) {
    const strength = props.sealStrength;
    if (strength >= 0.8) {
      // 强封板：深红背景
      return { background: `rgba(255, 68, 68, ${0.2 + strength * 0.3})` };
    } else {
      // 弱封板：浅红 + CSS 闪烁动画
      return { background: `rgba(255, 68, 68, ${0.1 + strength * 0.2})` };
    }
  }
  return {};
});
</script>

<style scoped>
.limit-tag {
  display: inline-block;
  padding: 1px 4px;
  border-radius: 2px;
  font-size: 10px;
  font-weight: 700;
}
.limit-up {
  background: rgba(255, 68, 68, 0.2);
  color: var(--color-limit-up);
}
.limit-down {
  background: rgba(0, 204, 102, 0.2);
  color: var(--color-limit-down);
}
</style>
