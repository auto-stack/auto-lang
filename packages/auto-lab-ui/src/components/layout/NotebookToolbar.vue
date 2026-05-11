<template>
  <header class="notebook-toolbar">
    <div class="toolbar-left">
      <FlaskConical :size="18" class="logo-icon" />
      <span class="app-title">AutoLab</span>
      <span v-if="filePath" class="file-path">{{ filePath }}{{ unsaved ? ' •' : '' }}</span>
      <span v-if="sessionStatus" class="status-badge" :class="sessionStatus">
        {{ sessionStatus }}
      </span>
    </div>
    <div class="toolbar-center">
      <button class="toolbar-btn" @click="$emit('new-notebook')">
        <FilePlus :size="14" />
        New
      </button>
      <button class="toolbar-btn" @click="$emit('open-file')">
        <FolderOpen :size="14" />
        Open
      </button>
      <button class="toolbar-btn" @click="$emit('save')">
        <Save :size="14" />
        Save
      </button>
    </div>
    <div class="toolbar-right">
      <button class="toolbar-btn run-all-btn" @click="$emit('run-all')">
        <Play :size="14" />
        Run All
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { FlaskConical, FilePlus, FolderOpen, Save, Play } from 'lucide-vue-next'

defineProps<{
  filePath: string | null
  unsaved: boolean
  sessionStatus?: string
}>()

defineEmits<{
  (e: 'new-notebook'): void
  (e: 'open-file'): void
  (e: 'save'): void
  (e: 'run-all'): void
}>()
</script>

<style scoped>
.notebook-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 1rem;
  background: #181825;
  border-bottom: 1px solid #313244;
  flex-shrink: 0;
  gap: 1rem;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.logo-icon {
  color: #6366f1;
}

.app-title {
  font-weight: 700;
  font-size: 0.95rem;
  color: #cdd6f4;
}

.file-path {
  font-size: 0.8rem;
  color: #6c7086;
  margin-left: 0.5rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.status-badge {
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
  margin-left: 0.5rem;
}

.status-badge.active {
  background: #27c93f22;
  color: #27c93f;
}

.status-badge.idle {
  background: #f9e2af22;
  color: #f9e2af;
}

.status-badge.closed {
  background: #6c708622;
  color: #6c7086;
}

.toolbar-center {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.toolbar-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.35rem 0.6rem;
  background: transparent;
  color: #6c7086;
  border: none;
  border-radius: 6px;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.15s;
}

.toolbar-btn:hover {
  background: #313244;
  color: #cdd6f4;
}

.run-all-btn {
  background: #27c93f;
  color: #1e1e2e;
  font-weight: 600;
}

.run-all-btn:hover {
  background: #2de745;
  color: #1e1e2e;
}
</style>
