import { createRouter, createWebHashHistory } from "vue-router";
import type { RouteRecordRaw } from "vue-router";
import AppLayout from "@/components/layout/AppLayout.vue";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    component: AppLayout,
    children: [
      {
        path: "",
        name: "Home",
        component: () => import("@/views/Home.vue"),
      },
      {
        path: "position",
        name: "Position",
        component: () => import("@/views/Position.vue"),
      },
      {
        path: "market",
        name: "StockMarket",
        component: () => import("@/views/StockMarket.vue"),
      },
      {
        path: "settings",
        name: "Settings",
        component: () => import("@/views/Settings.vue"),
      },
      {
        path: "anomaly",
        name: "AnomalyLog",
        component: () => import("@/views/AnomalyLog.vue"),
      },
      {
        path: "analysis",
        name: "AnalysisDashboard",
        component: () => import("@/views/AnalysisDashboard.vue"),
      },
      {
        path: "analysis/peg",
        name: "PegBoard",
        component: () => import("@/views/PegBoard.vue"),
      },
      {
        path: "analysis/report",
        name: "AnalysisReport",
        component: () => import("@/views/AnalysisReport.vue"),
      },
      {
        path: "analysis/industry",
        name: "IndustryPegComparison",
        component: () => import("@/views/IndustryPegComparison.vue"),
      },
      {
        path: "analysis/history",
        name: "AnalysisHistory",
        component: () => import("@/views/AnalysisHistory.vue"),
      },
      {
        path: "logs",
        name: "Logs",
        component: () => import("@/views/Logs.vue"),
      },
    ],
  },
  {
    path: "/suspend",
    name: "Suspend",
    component: () => import("@/views/Suspend.vue"),
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
