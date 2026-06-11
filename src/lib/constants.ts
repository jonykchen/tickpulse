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

/** 指数分类（6 组） */
export const INDEX_CATEGORIES = {
  /** 中国指数 */
  china: [
    { secid: "1.000001", name: "上证指数" },
    { secid: "0.399001", name: "深证成指" },
    { secid: "0.399006", name: "创业板指" },
    { secid: "1.000016", name: "上证50" },
    { secid: "1.000300", name: "沪深300" },
    { secid: "1.000905", name: "中证500" },
    { secid: "1.000852", name: "中证1000" },
  ],
  /** 大盘指数 */
  largeCap: [
    { secid: "1.000300", name: "沪深300" },
    { secid: "1.000016", name: "上证50" },
    { secid: "0.399673", name: "创业板50" },
    { secid: "1.000688", name: "科创50" },
  ],
  /** 50 指数 */
  fifty: [
    { secid: "1.000016", name: "上证50" },
    { secid: "0.399673", name: "创业板50" },
    { secid: "1.000688", name: "科创50" },
    { secid: "0.399380", name: "深证50" },
  ],
  /** 港股指数 */
  hk: [
    { secid: "100.HSI", name: "恒生指数" },
    { secid: "100.HSCEI", name: "国企指数" },
    { secid: "100.HSCI", name: "红筹指数" },
    { secid: "100.HSTECH", name: "恒生科技" },
  ],
  /** 汇率期货 */
  forex: [
    { secid: "120.USDX", name: "美元指数" },
    { secid: "120.CNY", name: "人民币" },
  ],
  /** 海外指数 */
  overseas: [
    { secid: "100.DJIA", name: "道琼斯" },
    { secid: "100.SPX", name: "标普500" },
    { secid: "100.NDX", name: "纳斯达克" },
    { secid: "100.N225", name: "日经225" },
  ],
} as const;

/** 大宗交易折溢率颜色 */
export const PREMIUM_RATE_COLORS = {
  positive: "#f5222d", // 溢价红
  negative: "#52c41a", // 折价绿
  zero: "#8c8c8c",     // 平价灰
} as const;

/** 抽屉层级 */
export const DRAWER_Z_INDEX = {
  SEARCH: 1900,
  OTC_FUND: 2000,
  KLINE: 2100,
  CHOOSE_GROUP: 2200,
} as const;
