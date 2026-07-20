<template>
  <div class="file-tree">
    <div
      v-for="file in files"
      :key="file.path"
      class="file-item"
      :class="{ active: file.path === selected, mapped: mappedFiles?.includes(file.path) }"
      @click="$emit('select', file.path)"
    >
      <span class="file-name">{{ file.path }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  files: { path: string }[];
  selected: string;
  mappedFiles?: string[];
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
.file-item.mapped {
  font-weight: 600;
}
.file-item.mapped .file-name::before {
  content: '•';
  color: #ff9d00;
  margin-right: 6px;
}
.file-name {
  font-family: 'Consolas', 'Monaco', monospace;
}
</style>
