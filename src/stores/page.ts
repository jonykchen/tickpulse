import { defineStore } from "pinia";
import { ref } from "vue";

/** 页面级 UI 状态管理 */
export const usePageDataStore = defineStore("page", () => {
  // ==================== 抽屉/弹窗状态 ====================

  /** K线抽屉 */
  const klineDrawer = ref<{
    visible: boolean;
    secid: string | null;
    name: string | null;
    period: string;
    adjust: string;
  }>({
    visible: false,
    secid: null,
    name: null,
    period: "daily",
    adjust: "forward",
  });

  /** ETF 持仓抽屉 */
  const etfDrawer = ref<{
    visible: boolean;
    secid: string | null;
    name: string | null;
  }>({
    visible: false,
    secid: null,
    name: null,
  });

  /** 基金抽屉 */
  const fundDrawer = ref<{
    visible: boolean;
    secid: string | null;
    name: string | null;
  }>({
    visible: false,
    secid: null,
    name: null,
  });

  // ==================== 编辑弹窗状态 ====================

  /** 分组管理 */
  const editGroup = ref<{
    visible: boolean;
    groupId: number | null;
    name: string | null;
  }>({
    visible: false,
    groupId: null,
    name: null,
  });

  /** 备注编辑 */
  const editNote = ref<{
    visible: boolean;
    stockId: number | null;
    note: string | null;
  }>({
    visible: false,
    stockId: null,
    note: null,
  });

  /** 自选股列表编辑 */
  const editStockList = ref<{
    visible: boolean;
    groupId: number | null;
  }>({
    visible: false,
    groupId: null,
  });

  /** 持仓编辑 */
  const editPosition = ref<{
    visible: boolean;
    positionId: number | null;
    secid: string | null;
    costPrice: string | null;
    quantity: string | null;
  }>({
    visible: false,
    positionId: null,
    secid: null,
    costPrice: null,
    quantity: null,
  });

  /** 分组选择器 */
  const choseGroup = ref<{
    visible: boolean;
    secid: string | null;
    name: string | null;
  }>({
    visible: false,
    secid: null,
    name: null,
  });

  /** 预警规则编辑器 */
  const alertRuleEditor = ref<{
    visible: boolean;
    secid: string | null;
    name: string | null;
    ruleId: string | null;
  }>({
    visible: false,
    secid: null,
    name: null,
    ruleId: null,
  });

  /** 设置弹窗 */
  const settings = ref<{
    visible: boolean;
    tab: string | null;
  }>({
    visible: false,
    tab: null,
  });

  // ==================== 右键菜单 ====================

  const contextMenu = ref<{
    visible: boolean;
    x: number;
    y: number;
    secid: string | null;
    name: string | null;
    stockId: number | null;
  }>({
    visible: false,
    x: 0,
    y: 0,
    secid: null,
    name: null,
    stockId: null,
  });

  // ==================== 搜索 ====================

  const search = ref<{
    visible: boolean;
    keyword: string;
  }>({
    visible: false,
    keyword: "",
  });

  // ==================== Actions ====================

  function openKline(secid: string, name: string) {
    klineDrawer.value = {
      visible: true,
      secid,
      name,
      period: "daily",
      adjust: "forward",
    };
  }

  function closeKline() {
    klineDrawer.value.visible = false;
  }

  function openEtf(secid: string, name: string) {
    etfDrawer.value = { visible: true, secid, name };
  }

  function closeEtf() {
    etfDrawer.value.visible = false;
  }

  function openFund(secid: string, name: string) {
    fundDrawer.value = { visible: true, secid, name };
  }

  function closeFund() {
    fundDrawer.value.visible = false;
  }

  function openEditPosition(
    positionId: number,
    secid: string,
    costPrice: string,
    quantity: string
  ) {
    editPosition.value = {
      visible: true,
      positionId,
      secid,
      costPrice,
      quantity,
    };
  }

  function closeEditPosition() {
    editPosition.value.visible = false;
  }

  function openContextMenu(
    x: number,
    y: number,
    secid: string,
    name: string,
    stockId: number
  ) {
    contextMenu.value = { visible: true, x, y, secid, name, stockId };
  }

  function closeContextMenu() {
    contextMenu.value.visible = false;
  }

  function openSearch() {
    search.value.visible = true;
  }

  function closeSearch() {
    search.value.visible = false;
    search.value.keyword = "";
  }

  function openAlertRuleEditor(
    secid: string,
    name: string,
    ruleId: string | null = null
  ) {
    alertRuleEditor.value = { visible: true, secid, name, ruleId };
  }

  function closeAlertRuleEditor() {
    alertRuleEditor.value.visible = false;
  }

  return {
    // State
    klineDrawer,
    etfDrawer,
    fundDrawer,
    editGroup,
    editNote,
    editStockList,
    editPosition,
    choseGroup,
    alertRuleEditor,
    settings,
    contextMenu,
    search,
    // Actions
    openKline,
    closeKline,
    openEtf,
    closeEtf,
    openFund,
    closeFund,
    openEditPosition,
    closeEditPosition,
    openContextMenu,
    closeContextMenu,
    openSearch,
    closeSearch,
    openAlertRuleEditor,
    closeAlertRuleEditor,
  };
});
