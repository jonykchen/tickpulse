<template>
  <div class="stock-group-tab">
    <div
      v-for="group in groups"
      :key="group.id"
      class="group-tab"
      :class="{ 'group-tab--active': group.id === activeGroup }"
      @click="$emit('select', group.id)"
    >
      {{ group.name }}
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  groups: { id: number; name: string; sortOrder: number }[];
  activeGroup: number;
}>();

defineEmits<{
  select: [groupId: number];
}>();
</script>

<style scoped>
.stock-group-tab {
  display: flex;
  gap: var(--spacing-sm);
}
.group-tab {
  padding: var(--spacing-xs) var(--spacing-md);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}
.group-tab:hover {
  color: var(--color-text-primary);
  background: var(--color-surface-hover);
}
.group-tab--active {
  color: var(--color-primary);
  background: rgba(52, 152, 219, 0.15);
  font-weight: 600;
}
</style>