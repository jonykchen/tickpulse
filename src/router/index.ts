import { createRouter, createWebHashHistory } from "vue-router";
import type { RouteRecordRaw } from "vue-router";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    name: "Home",
    component: () => import("@/views/Home.vue"),
  },
  {
    path: "/position",
    name: "Position",
    component: () => import("@/views/Position.vue"),
  },
  {
    path: "/market",
    name: "StockMarket",
    component: () => import("@/views/StockMarket.vue"),
  },
  {
    path: "/settings",
    name: "Settings",
    component: () => import("@/views/Settings.vue"),
  },
  {
    path: "/anomaly",
    name: "AnomalyLog",
    component: () => import("@/views/AnomalyLog.vue"),
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
