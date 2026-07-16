<template>
  <div class="file-tree">
    <div
      v-for="file in files"
      :key="file.path"
      class="file-item"
      :class="{ active: file.path === selected }"
      @click="$emit('select', file.path)"
    >
      <span class="file-name">{{ file.path }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { TransFile } from '../types';

defineProps<{
  files: TransFile[];
  selected: string;
}>();

defineEmits<{
  select: [path: string];
}>();
</script>

<style scoped>
.file-tree {
  display: flex;
  flex-direction: column;
  min-width: 140px;
  max-width: 220px;
  background: #252526;
  border-right: 1px solid #444;
  overflow-y: auto;
  padding: 4px 0;
}
.file-item {
  padding: 6px 12px;
  cursor: pointer;
  font-size: 12px;
  color: #cccccc;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.file-item:hover {
  background: #2a2d2e;
}
.file-item.active {
  background: #094771;
  color: #ffffff;
}
.file-name {
  font-family: 'Consolas', 'Monaco', monospace;
}
</style>
