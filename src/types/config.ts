/** 应用配置 */
export interface AppConfig {
  /** 主题 dark/light */
  theme: "dark" | "light";
  /** 默认分组ID */
  defaultGroupId: number;
  /** 开机自启 */
  autoStart: boolean;
  /** 最小化到托盘 */
  minimizeToTray: boolean;
  /** 预警开关 */
  alertEnabled: boolean;
  /** 预警声音 */
  alertSound: boolean;
  /** K线复权方式 forward/backward/none */
  klineAdjust: "forward" | "backward" | "none";
  /** PE显示类型 ttm/dynamic/static */
  peDisplayType: "ttm" | "dynamic" | "static";
  /** 首页表格显示列 */
  homeTableFields: string[];
  /** 收藏的证券ID */
  collectSecid: string[];
  /** 行情刷新频率(秒) */
  refreshRate: number;
  /** 分时线颜色 up/down */
  trendLineColor: { up: string; down: string };
  /** 角标类型 */
  badgeType: "changePercent" | "price" | "volume";
  /** 角标颜色 */
  badgeColor: "auto" | "red" | "green" | "blue";
  /** 悬浮窗配置 */
  suspend: SuspendConfig;
}

/** 悬浮窗配置 */
export interface SuspendConfig {
  /** 透明度 0.1-1.0 */
  opacity: number;
  /** 显示股票数量 */
  colorNum: number;
  /** 背景色 */
  bgColor: string;
  /** 显示类型 */
  showEnum: "all" | "self" | "position";
  /** 刷新频率(秒) */
  refreshRate: number;
  /** 排序方式 */
  sortType: "change" | "amount" | "turnover";
  /** 显示微信按钮 */
  showWechat: boolean;
  /** 分组配置 */
  group: {
    show: boolean;
    type: "industry" | "concept" | "self";
    index: number;
  };
}

/** 默认悬浮窗配置 */
export const DEFAULT_SUSPEND_CONFIG: SuspendConfig = {
  opacity: 0.9,
  colorNum: 5,
  bgColor: "#1a1a1a",
  showEnum: "all",
  refreshRate: 10,
  sortType: "change",
  showWechat: false,
  group: {
    show: false,
    type: "industry",
    index: 0,
  },
};

export const DEFAULT_CONFIG: AppConfig = {
  theme: "dark",
  defaultGroupId: 1,
  autoStart: false,
  minimizeToTray: true,
  alertEnabled: true,
  alertSound: true,
  klineAdjust: "forward",
  peDisplayType: "ttm",
  homeTableFields: [
    "name", "price", "changePercent", "change", "amount",
    "turnoverRate", "volumeRatio", "peTtm", "mainNetInflow",
    "totalMarketCap", "high", "low", "open", "preClose",
  ],
  collectSecid: [],
  refreshRate: 10,
  trendLineColor: { up: "#f5222d", down: "#52c41a" },
  badgeType: "changePercent",
  badgeColor: "auto",
  suspend: DEFAULT_SUSPEND_CONFIG,
};
