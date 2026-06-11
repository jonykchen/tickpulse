/** 业务常量 */

/** 市场代码映射 */
export const MARKET_CODE: Record<number, string> = {
  0: "SZ", // 深交所
  1: "SH", // 上交所
  116: "HK", // 港股
  134: "HK", // 港股
  105: "US", // 美股
  106: "US", // 美股
};

/** 创业板代码前缀 */
export const CHINEXT_PREFIX = "300";

/** 科创板代码前缀 */
export const STAR_PREFIX = "688";

/** 北交所代码前缀 */
export const BSE_PREFIX = "8";

/** ST 匹配正则（含全角星号/退市前缀） */
export const ST_PATTERN = /[SＳ][TＴ]|[\*＊]ST|退[A-Z]/;

/** 行情刷新阶段配置 */
export const PHASE_CONFIG = {
  Holiday: { interval: 3600, label: "休市" },
  PreMarket: { interval: 300, label: "盘前" },
  AuctionCancelable: { interval: 5, label: "集合竞价" },
  AuctionUncancelable: { interval: 3, label: "集合竞价" },
  PreOpen: { interval: 30, label: "即将开盘" },
  MorningVolatile: { interval: 6, label: "交易中" },
  MorningStable: { interval: 10, label: "交易中" },
  LunchBreak: { interval: 60, label: "午间休市" },
  AfternoonOpen: { interval: 6, label: "交易中" },
  AfternoonStable: { interval: 10, label: "交易中" },
  ClosingAuction: { interval: 5, label: "收盘竞价" },
  ContinuousClosing: { interval: 5, label: "收盘竞价(沪)" },
  AfterHours: { interval: 300, label: "已收盘" },
  PostMarketTrading: { interval: 15, label: "盘后交易" },
  PostMarketEnd: { interval: 300, label: "已收盘" },
} as const;
