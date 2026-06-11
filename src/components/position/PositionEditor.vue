<template>
  <NModal
    v-model:show="visible"
    preset="dialog"
    title="编辑持仓"
    positive-text="保存"
    negative-text="取消"
    @positive-click="handleSave"
  >
    <NForm label-placement="left" label-width="80">
      <NFormItem label="成本价">
        <NInputNumber
          v-model:value="formData.costPrice"
          :min="0"
          :step="0.01"
          :precision="2"
          placeholder="输入成本价"
          style="width: 100%"
        />
      </NFormItem>
      <NFormItem label="持股数量">
        <NInputNumber
          v-model:value="formData.quantity"
          :min="0"
          :step="100"
          placeholder="输入持股数量"
          style="width: 100%"
        />
      </NFormItem>
    </NForm>
  </NModal>
</template>

<script setup lang="ts">
import { reactive, computed } from "vue";
import { NModal, NForm, NFormItem, NInputNumber } from "naive-ui";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  visible: boolean;
  positionId: number | null;
  secid: string | null;
  initialCostPrice?: string;
  initialQuantity?: string;
}>();

const emit = defineEmits<{
  saved: [];
  cancel: [];
}>();

const formData = reactive({
  costPrice: parseFloat(props.initialCostPrice ?? "0"),
  quantity: parseInt(props.initialQuantity ?? "0"),
});

async function handleSave() {
  if (!props.positionId) return;
  try {
    await invoke("update_position", {
      id: props.positionId,
      costPrice: formData.costPrice.toFixed(2),
      quantity: formData.quantity.toString(),
    });
    emit("saved");
  } catch (e) {
    console.error("保存持仓失败:", e);
  }
}
</script>
