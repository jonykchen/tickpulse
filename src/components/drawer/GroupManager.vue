<template>
  <NModal
    v-model:show="visible"
    preset="dialog"
    title="分组管理"
    positive-text="确认"
    negative-text="取消"
    @positive-click="handleConfirm"
  >
    <div class="group-manager">
      <NInput
        v-model:value="newGroupName"
        placeholder="输入新分组名称"
        maxlength="6"
        @keyup.enter="addGroup"
      />
      <NButton size="small" @click="addGroup">添加分组</NButton>
      <div class="group-list">
        <div v-for="group in groups" :key="group.id" class="group-item">
          <span>{{ group.name }}</span>
          <NButton size="tiny" quaternary @click="removeGroup(group.id)">删除</NButton>
        </div>
      </div>
    </div>
  </NModal>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { NModal, NInput, NButton } from "naive-ui";
import { useMarketStore } from "@/stores/market";
import { getWatchlistGroups, createWatchlistGroup, deleteWatchlistGroup as deleteGroupApi } from "@/lib/tauri";

const visible = computed({
  get: () => true, // 由父组件控制
  set: () => {},
});

const newGroupName = ref("");
const groups = ref<{ id: number; name: string }[]>([]);

async function loadGroups() {
  try {
    groups.value = await getWatchlistGroups();
  } catch (e) {
    console.error("加载分组失败:", e);
  }
}

async function addGroup() {
  if (!newGroupName.value.trim()) return;
  try {
    await createWatchlistGroup(newGroupName.value.trim());
    newGroupName.value = "";
    await loadGroups();
  } catch (e) {
    console.error("创建分组失败:", e);
  }
}

async function removeGroup(id: number) {
  try {
    await deleteGroupApi(id);
    await loadGroups();
  } catch (e) {
    console.error("删除分组失败:", e);
  }
}

function handleConfirm() {
  // 关闭
}

loadGroups();
</script>

<style scoped>
.group-manager {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.group-list {
  max-height: 300px;
  overflow-y: auto;
}
.group-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid var(--color-bg-secondary);
}
</style>
