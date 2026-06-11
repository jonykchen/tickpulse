import { createApp } from "vue";
import { createPinia } from "pinia";
import piniaPluginPersistedstate from "pinia-plugin-persistedstate";
import App from "./App.vue";
import router from "./router";
import "virtual:uno.css";
import "./assets/styles/variables.css";
import "./assets/styles/global.css";

// 全局错误恢复
function handleInitError(error: unknown) {
  console.error('[App] Init error:', error);

  // 清除可能的损坏缓存
  try {
    localStorage.removeItem('app-config');
    localStorage.removeItem('watchlist');
    sessionStorage.clear();
  } catch (e) {
    console.warn('[App] Clear cache failed:', e);
  }

  // 显示错误提示并刷新页面
  alert('应用初始化失败，即将刷新重试...');
  window.location.reload();
}

// 全局错误监听
window.addEventListener('error', (event) => {
  console.error('[App] Global error:', event.error);
  // 对于初始化阶段的错误，执行恢复
  if (event.message.includes('initialize') || event.message.includes('setup')) {
    event.preventDefault();
    handleInitError(event.error);
  }
});

// 未处理 Promise 拒绝
window.addEventListener('unhandledrejection', (event) => {
  console.error('[App] Unhandled rejection:', event.reason);
  // 对于关键错误，执行恢复
  const reason = String(event.reason);
  if (reason.includes('initialize') || reason.includes('setup') || reason.includes('import')) {
    event.preventDefault();
    handleInitError(event.reason);
  }
});

const pinia = createPinia();
pinia.use(piniaPluginPersistedstate);

const app = createApp(App);
app.use(pinia);
app.use(router);
app.mount("#app");
