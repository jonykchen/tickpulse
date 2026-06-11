<template>
  <router-view v-slot="{ Component }">
    <transition name="fade-slide" mode="out-in">
      <component :is="Component" />
    </transition>
  </router-view>
</template>

<script setup lang="ts">
// App.vue 现在使用 router-view 包装，由 AppLayout 作为默认路由组件
</script>

<style>
/* 页面过渡动画 */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.2s var(--ease-out);
}
.fade-slide-enter-from {
  opacity: 0;
  transform: translateX(20px);
}
.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}
</style>