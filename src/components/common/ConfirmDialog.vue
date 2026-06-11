<template>
  <NModal
    v-model:show="visible"
    preset="dialog"
    :title="title"
    :content="message"
    positive-text="确认"
    negative-text="取消"
    @positive-click="$emit('confirm')"
  />
</template>

<script setup lang="ts">
import { NModal } from "naive-ui";

defineProps<{
  visible: boolean;
  title: string;
  message: string;
}>();

defineEmits<{
  confirm: [];
  cancel: [];
}>();
</script>
