<template>
  <span v-if="isMarginTarget" class="margin-tag" title="融资融券标的">融</span>
</template>

<script setup lang="ts">
defineProps<{
  isMarginTarget: boolean;
}>();
</script>

<style scoped>
.margin-tag {
  display: inline-block;
  padding: 0 4px;
  border-radius: 2px;
  font-size: 10px;
  font-weight: 600;
  line-height: 16px;
  background: rgba(24, 144, 255, 0.15);
  color: #1890ff;
  margin-left: 4px;
}
</style>
