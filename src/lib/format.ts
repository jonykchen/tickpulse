/**
 * 数字格式化工具
 * A股行情展示专用
 */

/** 格式化价格（2位小数） */
export function formatPrice(price: number): string {
  if (price <= 0) return "--";
  return price.toFixed(2);
}

/** 格式化涨跌幅（+2位小数+%） */
export function formatChangePercent(val: number): string {
  const prefix = val > 0 ? "+" : "";
  return `${prefix}${val.toFixed(2)}%`;
}

/** 格式化涨跌额 */
export function formatChange(val: number): string {
  const prefix = val > 0 ? "+" : "";
  return `${prefix}${val.toFixed(2)}`;
}

/** 格式化金额（万/亿） */
export function formatAmount(amount: number): string {
  if (amount >= 1e8) return (amount / 1e8).toFixed(2) + "亿";
  if (amount >= 1e4) return (amount / 1e4).toFixed(2) + "万";
  return amount.toFixed(0);
}

/** 格式化市值 */
export function formatMarketCap(cap: number): string {
  if (cap >= 1e12) return (cap / 1e12).toFixed(2) + "万亿";
  if (cap >= 1e8) return (cap / 1e8).toFixed(2) + "亿";
  if (cap >= 1e4) return (cap / 1e4).toFixed(2) + "万";
  return cap.toFixed(0);
}

/** 格式化成交量（手/万手） */
export function formatVolume(volume: number): string {
  if (volume >= 1e4) return (volume / 1e4).toFixed(2) + "万手";
  return volume + "手";
}

/** 获取涨跌颜色 class */
export function getChangeColorClass(val: number): string {
  if (val > 0) return "text-up";
  if (val < 0) return "text-down";
  return "text-flat";
}

/** 从代码判断市场标签 */
export function getMarketTag(code: string): string {
  if (code.startsWith("688")) return "科";
  if (code.startsWith("300") || code.startsWith("301")) return "创";
  if (code.startsWith("8") || code.startsWith("4")) return "北";
  return "";
}
