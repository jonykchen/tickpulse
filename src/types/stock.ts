/** 量比衰减提示 */
export enum VolumeRatioNote {
  /** 开盘30分钟内，量比偏高，参考价值有限 */
  Early = "early",
}

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
  /** 主力净流入 */
  mainNetInflow: number;
  /** 市场标识 0=深 1=沪 */
  market: number;

  // ── 换手率（v2.0: 区分流通/总） ──
  /** 流通换手率（东方财富 f8，常用） */
  turnoverRate: number;
  /** 总换手率（v2.0 新增） */
  totalTurnoverRate: number | null;

  // ── 市盈率（v2.0: 三种 PE） ──
  /** TTM 市盈率（最常用，默认展示） */
  peTtm: number;
  /** 动态市盈率 */
  peDynamic: number | null;
  /** 静态市盈率（上年度 EPS） */
  peStatic: number;
  /** 市净率 */
  pb: number;

  // ── 量比（v2.0: 衰减标记） ──
  /** 量比 */
  volumeRatio: number;
  /** 量比衰减提示 */
  volumeRatioNote: VolumeRatioNote | null;

  // ── 其他指标 ──
  /** 涨速 */
  changeSpeed: number;
  /** 年初至今涨幅 */
  ytdChange: number;

  // ── 涨跌停/停牌 ──
  /** 板块类型 */
  boardType: BoardType;
  /** 股票状态 */
  stockStatus: StockStatus;
  /** 是否涨停 */
  isLimitUp: boolean;
  /** 是否跌停 */
  isLimitDown: boolean;
  /** 是否接近涨停（涨幅≥8%） */
  isNearLimitUp: boolean;
  /** 涨停价 */
  limitUpPrice: number | null;
  /** 跌停价 */
  limitDownPrice: number | null;
  /** 是否停牌 */
  isSuspended: boolean;

  // ── 新股临停（v2.0） ──
  /** 是否临时停牌 */
  isTempSuspended: boolean;
  /** 临停原因 */
  tempSuspendReason: string | null;
  /** 预计恢复时间（时间戳） */
  tempSuspendResumeTime: number | null;

  // ── 封板信息 ──
  /** 封板强度 0.0-1.0（排除午休的有效封板率） */
  sealStrength: number | null;
  /** 炸板次数 */
  sealBreakCount: number;

  // ── 融资融券（v2.0） ──
  /** 是否为两融标的 */
  isMarginTarget: boolean;
  /** 融资余额（亿元） */
  marginBalance: number | null;
  /** 融券余量（万股） */
  shortVolume: number | null;
}

/** 板块类型枚举 */
export enum BoardType {
  /** 沪市主板 */
  MainBoardSH = "main_board_sh",
  /** 深市主板 */
  MainBoardSZ = "main_board_sz",
  /** 创业板 */
  ChiNext = "chinext",
  /** 科创板 */
  StarMarket = "star_market",
  /** 北交所 */
  BSE = "bse",
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
