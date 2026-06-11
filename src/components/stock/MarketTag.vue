<template>
  <span class="market-tag" :class="tagClass">{{ label }}</span>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { MarketTag } from "@/types/stock";

const props = defineProps<{
  tag: MarketTag;
}>();

const label = computed(() => {
  const map: Record<MarketTag, string> = {
    [MarketTag.MainBoard]: "",
    [MarketTag.ChiNext]: "创",
    [MarketTag.StarMarket]: "科",
    [MarketTag.BSE]: "北",
    [MarketTag.HK]: "港",
    [MarketTag.US]: "美",
  };
  return map[props.tag];
});

const tagClass = computed(() => `tag-${props.tag}`);
</script>

<style scoped>
.market-tag {
  display: inline-block;
  padding: 1px 4px;
  border-radius: 2px;
  font-size: 10px;
  font-weight: 600;
  line-height: 1.2;
}
.tag-chinext {
  background: rgba(52, 152, 219, 0.2);
  color: #3498db;
}
.tag-star {
  background: rgba(155, 89, 182, 0.2);
  color: #9b59b6;
}
.tag-bse {
  background: rgba(46, 204, 113, 0.2);
  color: #2ecc71;
}
.tag-hk {
  background: rgba(243, 156, 18, 0.2);
  color: #f39c12;
}
.tag-us {
  background: rgba(231, 76, 60, 0.2);
  color: #e74c3c;
}
</style>
