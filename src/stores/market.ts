import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { StockQuote } from "@/types/stock";

export interface MarketSummary {
  upCount: number;
  downCount: number;
  flatCount: number;
  limitUpCount: number;
  limitDownCount: number;
}

export const useMarketStore = defineStore("market", () => {
  // State
  const quotes = ref<Map<string, StockQuote>>(new Map());
  const phase = ref<string>("");
  const intervalSecs = ref<number>(0);
  const isTradingDay = ref<boolean>(false);
  const marketSummary = ref<MarketSummary>({
    upCount: 0,
    downCount: 0,
    flatCount: 0,
    limitUpCount: 0,
    limitDownCount: 0,
  });

  // Getters
  const sortedQuotes = computed(() => {
    return Array.from(quotes.value.values()).sort((a, b) => {
      // Sort by changePercent descending
      return b.changePercent - a.changePercent;
    });
  });

  const upCount = computed(() => {
    return Array.from(quotes.value.values()).filter(
      (q) => q.changePercent > 0
    ).length;
  });

  const downCount = computed(() => {
    return Array.from(quotes.value.values()).filter(
      (q) => q.changePercent < 0
    ).length;
  });

  // Actions
  function updateQuotes(newQuotes: StockQuote[]): void {
    for (const quote of newQuotes) {
      quotes.value.set(quote.secid, quote);
    }
  }

  function updatePhase(status: {
    phase: string;
    intervalSecs: number;
    isTradingDay: boolean;
  }): void {
    phase.value = status.phase;
    intervalSecs.value = status.intervalSecs;
    isTradingDay.value = status.isTradingDay;
  }

  function updateMarketSummary(summary: MarketSummary): void {
    marketSummary.value = summary;
  }

  function clearQuotes(): void {
    quotes.value.clear();
  }

  function removeQuote(secid: string): void {
    quotes.value.delete(secid);
  }

  return {
    // State
    quotes,
    phase,
    intervalSecs,
    isTradingDay,
    marketSummary,
    // Getters
    sortedQuotes,
    upCount,
    downCount,
    // Actions
    updateQuotes,
    updatePhase,
    updateMarketSummary,
    clearQuotes,
    removeQuote,
  };
});
