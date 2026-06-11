import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { AnomalyEvent } from "@/types/anomaly";

const MAX_ANOMALIES = 200;

export const useAnomalyStore = defineStore("anomaly", () => {
  // State - FIFO queue with max 200 items
  const anomalies = ref<AnomalyEvent[]>([]);

  // Getters
  const recentAnomalies = computed(() => {
    // Return last 20 anomalies (most recent first)
    return anomalies.value.slice(-20).reverse();
  });

  const anomaliesByStock = computed(() => {
    return (secid: string) => {
      return anomalies.value.filter((a) => a.secid === secid);
    };
  });

  const anomalyCount = computed(() => {
    return anomalies.value.length;
  });

  // Actions
  function addAnomaly(event: AnomalyEvent): void {
    anomalies.value.push(event);

    // FIFO: remove oldest if exceeds max
    if (anomalies.value.length > MAX_ANOMALIES) {
      anomalies.value.shift();
    }
  }

  function addAnomalies(events: AnomalyEvent[]): void {
    for (const event of events) {
      addAnomaly(event);
    }
  }

  function clearAnomalies(): void {
    anomalies.value = [];
  }

  function removeAnomaly(id: string): void {
    const index = anomalies.value.findIndex((a) => a.id === id);
    if (index !== -1) {
      anomalies.value.splice(index, 1);
    }
  }

  return {
    // State
    anomalies,
    // Getters
    recentAnomalies,
    anomaliesByStock,
    anomalyCount,
    // Actions
    addAnomaly,
    addAnomalies,
    clearAnomalies,
    removeAnomaly,
  };
});
