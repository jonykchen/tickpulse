/** AI 分析引擎类型定义 */

/** 分析维度 */
export enum AnalysisDimension {
  IndustryTrend = "IndustryTrend",
  CompetitivePosition = "CompetitivePosition",
  FinancialHealth = "FinancialHealth",
  ManagementQuality = "ManagementQuality",
  GrowthPotential = "GrowthPotential",
  Valuation = "Valuation",
  TechnicalSignals = "TechnicalSignals",
}

/** 维度显示名称 */
export const DIMENSION_LABELS: Record<AnalysisDimension, string> = {
  [AnalysisDimension.IndustryTrend]: "行业趋势",
  [AnalysisDimension.CompetitivePosition]: "竞争格局",
  [AnalysisDimension.FinancialHealth]: "财务健康",
  [AnalysisDimension.ManagementQuality]: "管理层质量",
  [AnalysisDimension.GrowthPotential]: "成长性评估",
  [AnalysisDimension.Valuation]: "估值分析",
  [AnalysisDimension.TechnicalSignals]: "技术面信号",
};

/** 维度评级 */
export enum DimensionRating {
  A = "A",
  B = "B",
  C = "C",
  D = "D",
  F = "F",
}

/** 综合评级 */
export enum OverallRating {
  StrongBuy = "StrongBuy",
  Buy = "Buy",
  Hold = "Hold",
  Sell = "Sell",
  StrongSell = "StrongSell",
}

/** 综合评级显示名称 */
export const OVERALL_RATING_LABELS: Record<OverallRating, string> = {
  [OverallRating.StrongBuy]: "强烈推荐",
  [OverallRating.Buy]: "推荐",
  [OverallRating.Hold]: "观望",
  [OverallRating.Sell]: "回避",
  [OverallRating.StrongSell]: "强烈回避",
};

/** 维度报告 */
export interface DimensionReport {
  dimension: AnalysisDimension;
  rating: DimensionRating;
  summary: string;
  keyPoints: string[];
  risks: string[];
  opportunities: string[];
  confidence: number;
}

/** 分析结果 */
export interface AnalysisResult {
  id: string;
  secid: string;
  stockName: string;
  /** 维度报告，key 为维度枚举字符串如 "IndustryTrend" */
  dimensions: Record<string, DimensionReport>;
  overallRating: OverallRating;
  overallScore: number;
  bullArgument: string;
  bearArgument: string;
  verdict: string;
  qualityScore: number;
  qualityGrade: string;
  readableReport: string;
  createdAt: number;
}

/** PEG 评级 */
export enum PegRating {
  ExtremelyUndervalued = "ExtremelyUndervalued",
  Undervalued = "Undervalued",
  Fair = "Fair",
  Overvalued = "Overvalued",
  ExtremelyOvervalued = "ExtremelyOvervalued",
}

/** PEG 评级显示名称 */
export const PEG_RATING_LABELS: Record<PegRating, string> = {
  [PegRating.ExtremelyUndervalued]: "严重低估",
  [PegRating.Undervalued]: "低估",
  [PegRating.Fair]: "合理",
  [PegRating.Overvalued]: "高估",
  [PegRating.ExtremelyOvervalued]: "严重高估",
};

/** PEG 数据 */
export interface PegData {
  secid: string;
  stockName: string;
  peTtm: number;
  cagr: number;
  pegValue: number;
  pegRating: PegRating;
  industryPeg: number | null;
  tradeDate: string;
}

/** LLM 配置 */
export interface LlmConfig {
  provider: "anthropic" | "openai" | "deepseek" | "ollama" | "qwen" | "glm" | "minimax";
  model: string;
  apiKey: string | null;
  baseUrl: string | null;
  mode: "cloud" | "local";
  thinkingEnabled: boolean;
}

/** 双 LLM 配置（Quick-Think + Deep-Think） */
export interface DualLlmConfig {
  quickThink: LlmConfig;
  deepThink: LlmConfig | null;
}

/** 支持的供应商信息 */
export interface SupportedProvider {
  id: string;
  name: string;
  defaultBaseUrl: string;
  isOpenaiCompat: boolean;
  supportsStructuredOutput: boolean;
}

/** 分析预设 */
export interface AnalysisPreset {
  id: string;
  name: string;
  description: string;
  dimensions: AnalysisDimension[];
  maxTokens: number;
  temperature: number;
}

/** 分析进度 */
export interface AnalysisProgress {
  secid: string;
  stockName: string;
  totalDimensions: number;
  completedDimensions: number;
  currentDimension: AnalysisDimension | null;
  currentStep: string;
  percent: number;
  startedAt: number;
  estimatedRemainingSecs: number | null;
}

/** PEG 看板项 */
export interface PegBoardItem {
  secid: string;
  name: string;
  price: number;
  changePercent: number;
  peTtm: number;
  pb: number;
  pegValue: number | null;
  pegRating: PegRating | null;
  digestionYears: number | null;
}

/** 行业 PE 对比项 */
export interface IndustryComparisonItem {
  industryName: string;
  avgPe: number;
  avgPb: number;
  medianPe: number;
  stockPe: number | null;
  stockPb: number | null;
  premiumDiscount: number | null;
}
