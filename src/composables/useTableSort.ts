/**
 * 表格排序逻辑
 * 支持多列切换排序，同列点击切换升降序
 */
import { ref, computed } from "vue";

export function useTableSort() {
  const sortKey = ref("");
  const sortOrder = ref<"asc" | "desc">("asc");

  /**
   * 切换排序列
   * 同一列点击切换升降序，不同列默认升序
   */
  function sortBy(key: string): void {
    if (sortKey.value === key) {
      sortOrder.value = sortOrder.value === "asc" ? "desc" : "asc";
    } else {
      sortKey.value = key;
      sortOrder.value = "asc";
    }
  }

  /**
   * 对数据数组按当前排序状态排序
   * @param data 待排序数组
   * @param key 排序字段名（对应对象属性）
   * @returns 排序后的新数组（computed）
   */
  const sortedData = computed(() => {
    return <T extends Record<string, unknown>>(data: T[]): T[] => {
      if (!sortKey.value) return data;

      const key = sortKey.value;
      const order = sortOrder.value;

      return [...data].sort((a, b) => {
        const valA = a[key];
        const valB = b[key];

        // 处理 null/undefined，排到末尾
        if (valA == null && valB == null) return 0;
        if (valA == null) return 1;
        if (valB == null) return -1;

        // 数值比较
        if (typeof valA === "number" && typeof valB === "number") {
          return order === "asc" ? valA - valB : valB - valA;
        }

        // 字符串比较
        const strA = String(valA);
        const strB = String(valB);
        const cmp = strA.localeCompare(strB, "zh-CN", { numeric: true });
        return order === "asc" ? cmp : -cmp;
      });
    };
  });

  return {
    sortKey,
    sortOrder,
    sortBy,
    sortedData,
  };
}
