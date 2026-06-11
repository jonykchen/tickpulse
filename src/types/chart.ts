/** K线数据 */
export interface KlineBar {
  /** 时间戳 */
  time: number;
  /** 开盘价 */
  open: number;
  /** 收盘价 */
  close: number;
  /** 最高价 */
  high: number;
  /** 最低价 */
  low: number;
  /** 成交量 */
  volume: number;
  /** 成交额 */
  amount: number;
  /** 涨跌幅(%) */
  changePercent: number;
}

/** K线周期 */
export enum KlinePeriod {
  /** 1分钟 */
  Min1 = "1m",
  /** 5分钟 */
  Min5 = "5m",
  /** 15分钟 */
  Min15 = "15m",
  /** 30分钟 */
  Min30 = "30m",
  /** 60分钟 */
  Min60 = "60m",
  /** 日K */
  Daily = "day",
  /** 周K */
  Weekly = "week",
  /** 月K */
  Monthly = "month",
}

/** 复权类型 */
export enum AdjustType {
  /** 不复权 */
  None = "none",
  /** 前复权 */
  Forward = "forward",
  /** 后复权 */
  Backward = "backward",
}

/** 分时数据 */
export interface TimelinePoint {
  /** 时间 HH:mm */
  time: string;
  /** 价格 */
  price: number;
  /** 均价 */
  avgPrice: number;
  /** 成交量 */
  volume: number;
}

/** 分时走势数据 */
export interface TimelineData {
  secid: string;
  name: string;
  preClose: number;
  points: TimelinePoint[];
}
