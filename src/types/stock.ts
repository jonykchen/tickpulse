/** 股票行情数据 */
export interface StockQuote {
  /** 证券ID，如 "0.000001" */
  secid: string;
  /** 股票代码 */
  code: string;
  /** 股票名称 */
  name: string;
  /** 最新价 */
  price: number;
  /** 涨跌额 */
  change: number;
  /** 涨跌幅(%) */
  changePercent: number;
  /** 成交量(手) */
  volume: number;
  /** 成交额 */
  amount: number;
  /** 换手率(流通) */
  turnoverRate: number;
  /** 量比 */
  volumeRatio: number;
  /** 最高价 */
  high: number;
  /** 最低价 */
  low: number;
  /** 开盘价 */
  open: number;
  /** 昨收价 */
  preClose: number;
  /** 总市值 */
  totalMarketCap: number;
  /** 市盈率(动/TTM) */
  peTtm: number;
  /** 市盈率(静) */
  peStatic: number;
  /** 市净率 */
  pb: number;
  /** 涨速 */
  changeSpeed: number;
  /** 年初至今涨幅 */
  ytdChange: number;
  /** 主力净流入 */
  mainNetInflow: number;
  /** 市场标识 0=深 1=沪 */
  market: number;
  /** 涨停价 */
  limitUp?: number;
  /** 跌停价 */
  limitDown?: number;
  /** 封板强度 0-1 */
  sealStrength?: number;
  /** 炸板次数 */
  breakCount?: number;
  /** 是否停牌 */
  isSuspended: boolean;
  /** 股票状态 */
  status: StockStatus;
}

/** 股票状态枚举 */
export enum StockStatus {
  /** 正常交易 */
  Normal = "normal",
  /** ST股 */
  ST = "st",
  /** 退市整理 */
  Delisting = "delisting",
  /** 新股（上市首日/次日起） */
  NewStock = "new_stock",
  /** 停牌 */
  Suspended = "suspended",
  /** 临时停牌 */
  TempSuspended = "temp_suspended",
}

/** 交易所枚举 */
export enum Exchange {
  /** 上交所 */
  SSE = "sse",
  /** 深交所 */
  SZSE = "szse",
  /** 北交所 */
  BSE = "bse",
  /** 港股 */
  HK = "hk",
  /** 美股 */
  US = "us",
}

/** 市场标签 */
export enum MarketTag {
  /** 主板 */
  MainBoard = "main",
  /** 创业板 */
  ChiNext = "chinext",
  /** 科创板 */
  StarMarket = "star",
  /** 北交所 */
  BSE = "bse",
  /** 港股 */
  HK = "hk",
  /** 美股 */
  US = "us",
}

/** 自选股项 */
export interface WatchlistStock {
  id: number;
  groupId: number;
  secid: string;
  name: string;
  note?: string;
  sortOrder: number;
  isPinned: boolean;
  createdAt: number;
}

/** 自选股分组 */
export interface WatchlistGroup {
  id: number;
  name: string;
  sortOrder: number;
  createdAt: number;
  updatedAt: number;
}
