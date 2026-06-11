import { createPinia } from "pinia";

const pinia = createPinia();

export default pinia;

export { useConfigStore } from "./config";
export { useMarketStore } from "./market";
export { useAnomalyStore } from "./anomaly";
export { useAlertStore } from "./alert";
