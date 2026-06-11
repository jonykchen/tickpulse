<template>
  <NModal
    v-model:show="visible"
    preset="dialog"
    :title="isEdit ? '编辑预警规则' : '添加预警规则'"
    positive-text="确认"
    negative-text="取消"
    @positive-click="handleConfirm"
  >
    <NForm ref="formRef" :model="formData" label-placement="left" label-width="80">
      <NFormItem label="股票">
        <NInput :value="stockName" disabled />
      </NFormItem>
      <NFormItem label="预警类型">
        <NSelect
          v-model:value="formData.ruleType"
          :options="ruleTypeOptions"
          @update:value="onRuleTypeChange"
        />
      </NFormItem>
      <NFormItem v-if="needsThreshold" label="阈值">
        <NInputNumber
          v-model:value="formData.threshold"
          :placeholder="thresholdPlaceholder"
          :step="thresholdStep"
          :min="0"
          style="width: 100%"
        />
      </NFormItem>
    </NForm>
  </NModal>
</template>

<script setup lang="ts">
import { ref, computed, reactive } from "vue";
import { NModal, NForm, NFormItem, NInput, NSelect, NInputNumber } from "naive-ui";
import {
  AlertRuleType,
  ALERT_TYPE_LABELS,
  ALERT_TYPE_NEEDS_THRESHOLD,
} from "@/types/alert";
import { addAlertRule } from "@/lib/tauri";

const props = defineProps<{
  visible: boolean;
  secid: string;
  stockName: string;
  ruleId?: string | null;
}>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();

const isEdit = computed(() => !!props.ruleId);

const formData = reactive({
  ruleType: AlertRuleType.PriceAbove,
  threshold: 0,
});

const needsThreshold = computed(
  () => ALERT_TYPE_NEEDS_THRESHOLD[formData.ruleType]
);

const ruleTypeOptions = Object.entries(ALERT_TYPE_LABELS).map(
  ([value, label]) => ({
    label,
    value: value as AlertRuleType,
  })
);

const thresholdPlaceholder = computed(() => {
  switch (formData.ruleType) {
    case AlertRuleType.PriceAbove:
    case AlertRuleType.PriceBelow:
      return "输入价格";
    case AlertRuleType.ChangePercentAbove:
    case AlertRuleType.ChangePercentBelow:
      return "输入涨跌幅(%)";
    case AlertRuleType.VolumeRatioAbove:
      return "输入量比";
    case AlertRuleType.TurnoverRateAbove:
      return "输入换手率(%)";
    case AlertRuleType.AnomalyRise:
    case AlertRuleType.AnomalyFall:
      return "输入涨速(%)";
    default:
      return "输入阈值";
  }
});

const thresholdStep = computed(() => {
  switch (formData.ruleType) {
    case AlertRuleType.PriceAbove:
    case AlertRuleType.PriceBelow:
      return 0.01;
    default:
      return 0.1;
  }
});

function onRuleTypeChange() {
  if (!ALERT_TYPE_NEEDS_THRESHOLD[formData.ruleType]) {
    formData.threshold = 0;
  }
}

async function handleConfirm() {
  try {
    await addAlertRule(
      props.secid,
      props.stockName,
      formData.ruleType,
      formData.threshold
    );
    emit("confirm");
  } catch (e) {
    console.error("保存预警规则失败:", e);
  }
}
</script>
