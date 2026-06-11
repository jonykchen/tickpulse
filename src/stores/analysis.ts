import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  AnalysisResult,
  AnalysisProgress,
  LlmConfig,
  PegData,
  PegBoardItem,
  IndustryComparisonItem,
} from "@/types/analysis";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Re-export types for external use
export type { PegBoardItem, IndustryComparisonItem } from "@/types/analysis";

/** 分析状态 */
export interface AnalysisStatus {
  isAnalyzing: boolean;
  analyzingSecid: string | null;
  progress: AnalysisProgress | null;
}

export const useAnalysisStore = defineStore("analysis", () => {
  // ==================== State ====================
  const results = ref<Map<string, AnalysisResult>>(new Map());
  const progress = ref<AnalysisProgress | null>(null);
  const llmConfig = ref<LlmConfig>({
    provider: "anthropic",
    model: "claude-sonnet-4-6",
    apiKey: null,
    baseUrl: null,
    mode: "cloud",
  });
  const pegCache = ref<Map<string, PegData>>(new Map());

  // 新增状态
  const isAnalyzing = ref(false);
  const analyzingSecid = ref<string | null>(null);
  const analysisProgress = ref<AnalysisProgress | null>(null);
  const pegBoard = ref<PegBoardItem[]>([]);
  const industryComparison = ref<IndustryComparisonItem[]>([]);

  // 事件监听器清理函数
  let unlistenProgress: UnlistenFn | null = null;

  // ==================== Getters ====================
  const latestResults = computed(() => {
    return Array.from(results.value.values()).sort(
      (a, b) => b.createdAt - a.createdAt
    );
  });

  const getResultBySecid = computed(() => {
    return (secid: string) => results.value.get(secid);
  });

  const analysisStatus = computed<AnalysisStatus>(() => ({
    isAnalyzing: isAnalyzing.value,
    analyzingSecid: analyzingSecid.value,
    progress: analysisProgress.value,
  }));

  // ==================== Actions ====================
  function addResult(result: AnalysisResult): void {
    results.value.set(result.secid, result);
  }

  function updateProgress(p: AnalysisProgress): void {
    progress.value = p;
    isAnalyzing.value = p.percent < 100;
  }

  function clearProgress(): void {
    progress.value = null;
    isAnalyzing.value = false;
    analysisProgress.value = null;
    analyzingSecid.value = null;
  }

  function updateLlmConfig(config: Partial<LlmConfig>): void {
    llmConfig.value = { ...llmConfig.value, ...config };
  }

  function addPegData(data: PegData): void {
    pegCache.value.set(data.secid, data);
  }

  async function loadLlmConfig(): Promise<void> {
    try {
      const config = await invoke<LlmConfig>("get_llm_config");
      if (config) {
        llmConfig.value = config;
      }
    } catch {
      // 使用默认配置
    }
  }

  /**
   * 触发 AI 分析
   * @param secid 股票代码 (如 "000001.SZ")
   * @param stockName 股票名称
   * @param profile 分析预设ID (可选)
   */
  async function triggerAnalysis(
    secid: string,
    stockName: string,
    profile?: string
  ): Promise<AnalysisResult> {
    isAnalyzing.value = true;
    analyzingSecid.value = secid;
    analysisProgress.value = {
      secid,
      stockName,
      totalDimensions: 7,
      completedDimensions: 0,
      currentDimension: null,
      currentStep: "初始化分析引擎...",
      percent: 0,
      startedAt: Date.now(),
      estimatedRemainingSecs: null,
    };

    try {
      const result = await invoke<AnalysisResult>("analyze_stock", {
        secid,
        stockName,
        profile: profile ?? null,
      });

      // 保存结果
      results.value.set(secid, result);
      addResult(result);

      return result;
    } catch (error) {
      console.error("分析失败:", error);
      throw error;
    } finally {
      isAnalyzing.value = false;
      analyzingSecid.value = null;
      analysisProgress.value = null;
    }
  }

  /**
   * 批量获取 PEG 看板数据
   * @param secids 股票代码列表 (可选，不传则返回全部)
   */
  async function fetchPegBoard(secids?: string[]): Promise<PegBoardItem[]> {
    try {
      // 调用 Tauri Command 获取 PEG 看板数据
      const data = await invoke<PegBoardItem[]>("fetch_peg_board", {
        secids: secids ?? null,
      });
      pegBoard.value = data;
      return data;
    } catch (error) {
      console.error("获取PEG看板失败:", error);
      // 返回空数组而不是抛出错误，避免阻塞 UI
      return [];
    }
  }

  /**
   * 获取行业 PE 对比数据
   * @param secid 股票代码
   */
  async function fetchIndustryComparison(
    secid: string
  ): Promise<IndustryComparisonItem[]> {
    try {
      const data = await invoke<IndustryComparisonItem[]>(
        "fetch_industry_comparison",
        { secid }
      );
      industryComparison.value = data;
      return data;
    } catch (error) {
      console.error("获取行业对比失败:", error);
      return [];
    }
  }

  /**
   * 导出分析报告
   * @param analysisId 分析结果ID
   * @param format 导出格式 ('pdf' | 'markdown')
   */
  async function exportReport(
    analysisId: string,
    format: "pdf" | "markdown"
  ): Promise<string> {
    try {
      if (format === "pdf") {
        // 调用 PDF 导出命令
        const filePath = await invoke<string>("export_analysis_pdf", {
          analysisId,
        });
        return filePath;
      } else {
        // 生成 Markdown 格式
        const result = Array.from(results.value.values()).find(
          (r) => r.id === analysisId
        );
        if (!result) {
          throw new Error("未找到分析结果");
        }
        return generateMarkdownReport(result);
      }
    } catch (error) {
      console.error("导出报告失败:", error);
      throw error;
    }
  }

  /**
   * 获取分析历史
   */
  async function fetchHistory(
    secid?: string,
    limit?: number
  ): Promise<AnalysisResult[]> {
    try {
      const history = await invoke<AnalysisResult[]>("get_analysis_history", {
        secid: secid ?? null,
        limit: limit ?? 20,
      });
      // 更新本地缓存
      for (const result of history) {
        results.value.set(result.secid, result);
      }
      return history;
    } catch (error) {
      console.error("获取分析历史失败:", error);
      return [];
    }
  }

  /**
   * 删除分析结果
   */
  async function removeResult(analysisId: string): Promise<void> {
    try {
      await invoke("delete_analysis_result", { id: analysisId });
      // 从本地缓存中移除
      for (const [secid, result] of results.value.entries()) {
        if (result.id === analysisId) {
          results.value.delete(secid);
          break;
        }
      }
    } catch (error) {
      console.error("删除分析结果失败:", error);
      throw error;
    }
  }

  /**
   * 计算 PEG
   */
  async function calculatePeg(
    pe: number,
    cagr: number
  ): Promise<{ peg: number; rating: string; ratingDisplay: string }> {
    try {
      const result = await invoke<{
        peg: number;
        rating: string;
        ratingDisplay: string;
        ratingScore: number;
      }>("calc_peg", { pe, cagr });
      return result;
    } catch (error) {
      console.error("计算PEG失败:", error);
      throw error;
    }
  }

  /**
   * 计算 CAGR
   */
  async function calculateCagr(
    beginValue: number,
    endValue: number,
    years: number
  ): Promise<number> {
    try {
      const cagr = await invoke<number>("calc_cagr", {
        beginValue,
        endValue,
        years,
      });
      return cagr;
    } catch (error) {
      console.error("计算CAGR失败:", error);
      throw error;
    }
  }

  /**
   * 设置事件监听
   * 在组件 setup 中调用
   */
  async function setupEventListeners(): Promise<() => void> {
    // 监听分析进度事件
    unlistenProgress = await listen<AnalysisProgress>(
      "analysis-progress",
      (event) => {
        analysisProgress.value = event.payload;
        // 同步更新旧 progress 以保持兼容
        progress.value = event.payload;
        isAnalyzing.value = event.payload.percent < 100;
      }
    );

    // 返回清理函数
    return () => {
      if (unlistenProgress) {
        unlistenProgress();
        unlistenProgress = null;
      }
    };
  }

  // ==================== Helper Functions ====================

  /**
   * 生成 Markdown 格式的分析报告
   */
  function generateMarkdownReport(result: AnalysisResult): string {
    const lines: string[] = [];

    lines.push(`# ${result.stockName} 分析报告`);
    lines.push("");
    lines.push(
      `**综合评级：${getOverallRatingLabel(result.overallRating)}** (评分: ${result.overallScore.toFixed(1)}/100)`
    );
    lines.push("");
    lines.push(
      `**质量门控：${result.qualityGrade}** (评分: ${result.qualityScore.toFixed(1)})`
    );
    lines.push("");

    lines.push("## 多空观点");
    lines.push("");
    lines.push("### 多方观点");
    lines.push(result.bullArgument);
    lines.push("");
    lines.push("### 空方观点");
    lines.push(result.bearArgument);
    lines.push("");
    lines.push("### 裁决");
    lines.push(result.verdict);
    lines.push("");

    lines.push("## 维度详情");
    lines.push("");

    for (const [dimKey, dim] of Object.entries(result.dimensions)) {
      lines.push(`### ${getDimensionLabel(dimKey)} [${dim.rating}]`);
      lines.push("");
      lines.push(dim.summary);
      lines.push("");

      if (dim.keyPoints.length > 0) {
        lines.push("**关键要点：**");
        for (const p of dim.keyPoints) {
          lines.push(`- ${p}`);
        }
        lines.push("");
      }

      if (dim.risks.length > 0) {
        lines.push("**风险：**");
        for (const r of dim.risks) {
          lines.push(`- ${r}`);
        }
        lines.push("");
      }

      if (dim.opportunities.length > 0) {
        lines.push("**机会：**");
        for (const o of dim.opportunities) {
          lines.push(`- ${o}`);
        }
        lines.push("");
      }

      lines.push(`置信度: ${(dim.confidence * 100).toFixed(0)}%`);
      lines.push("");
    }

    lines.push("---");
    lines.push(`*生成时间: ${new Date(result.createdAt).toLocaleString("zh-CN")}*`);

    return lines.join("\n");
  }

  /**
   * 获取综合评级标签
   */
  function getOverallRatingLabel(rating: string): string {
    const labels: Record<string, string> = {
      StrongBuy: "强烈推荐",
      Buy: "推荐",
      Hold: "观望",
      Sell: "回避",
      StrongSell: "强烈回避",
    };
    return labels[rating] ?? rating;
  }

  /**
   * 获取维度标签
   */
  function getDimensionLabel(dimKey: string): string {
    const labels: Record<string, string> = {
      IndustryTrend: "行业趋势",
      CompetitivePosition: "竞争格局",
      FinancialHealth: "财务健康",
      ManagementQuality: "管理层质量",
      GrowthPotential: "成长性评估",
      Valuation: "估值分析",
      TechnicalSignals: "技术面信号",
    };
    return labels[dimKey] ?? dimKey;
  }

  // ==================== Export ====================

  return {
    // State
    results,
    progress,
    llmConfig,
    pegCache,
    isAnalyzing,
    analyzingSecid,
    analysisProgress,
    pegBoard,
    industryComparison,
    // Getters
    latestResults,
    getResultBySecid,
    analysisStatus,
    // Actions
    addResult,
    updateProgress,
    clearProgress,
    updateLlmConfig,
    addPegData,
    loadLlmConfig,
    triggerAnalysis,
    fetchPegBoard,
    fetchIndustryComparison,
    exportReport,
    fetchHistory,
    removeResult,
    calculatePeg,
    calculateCagr,
    setupEventListeners,
  };
});
