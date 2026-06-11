/**
 * LightweightCharts 配置常量
 */

/** K线主图颜色配置 */
export const CANDLE_COLORS = {
  upColor: "#e74c3c",
  downColor: "#2ecc71",
  upWickColor: "#e74c3c",
  downWickColor: "#2ecc71",
  borderUpColor: "#e74c3c",
  borderDownColor: "#2ecc71",
} as const;

/** 成交量颜色 */
export const VOLUME_COLORS = {
  upColor: "rgba(231, 76, 60, 0.5)",
  downColor: "rgba(46, 204, 113, 0.5)",
} as const;

/** MA 线颜色 */
export const MA_COLORS = {
  ma5: "#f1c40f",
  ma10: "#3498db",
  ma20: "#e74c3c",
  ma60: "#9b59b6",
} as const;

/** 网格线配置 */
export const GRID_CONFIG = {
  horzLines: { color: "rgba(255, 255, 255, 0.05)" },
  vertLines: { color: "rgba(255, 255, 255, 0.05)" },
} as const;

/** 十字光标配置 */
export const CROSSHAIR_CONFIG = {
  mode: 0, // Normal
  vertLine: { color: "rgba(255, 255, 255, 0.1)", width: 1, style: 2 },
  horzLine: { color: "rgba(255, 255, 255, 0.1)", width: 1, style: 2 },
} as const;

/** 时间刻度配置 */
export const TIME_SCALE_CONFIG = {
  borderColor: "rgba(255, 255, 255, 0.1)",
  timeVisible: true,
  secondsVisible: false,
} as const;

/** 分时图颜色 */
export const TIMELINE_COLORS = {
  lineColor: "#3498db",
  areaTopColor: "rgba(52, 152, 219, 0.3)",
  areaBottomColor: "rgba(52, 152, 219, 0.02)",
  avgLineColor: "#f39c12",
} as const;
