<template>
  <div
    :class="[
      'card',
      {
        'card-elevated': elevated,
        'card-interactive': interactive,
        'card-glass': glass,
      }
    ]"
    :style="paddingStyle"
  >
    <slot />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    elevated?: boolean;
    interactive?: boolean;
    glass?: boolean;
    padding?: "none" | "sm" | "md" | "lg";
  }>(),
  {
    elevated: false,
    interactive: false,
    glass: false,
    padding: "md",
  }
);

const paddingStyle = computed(() => {
  const paddingMap = {
    none: "0",
    sm: "var(--spacing-sm)",
    md: "var(--spacing-lg)",
    lg: "var(--spacing-xl)",
  };
  return { padding: paddingMap[props.padding] };
});
</script>

<style scoped>
.card {
  /* 基础样式在 global.css 中定义 */
}

.card-glass {
  background: var(--glass-bg);
  backdrop-filter: var(--blur-md);
  -webkit-backdrop-filter: var(--blur-md);
}

.card-interactive {
  cursor: pointer;
}
</style>
