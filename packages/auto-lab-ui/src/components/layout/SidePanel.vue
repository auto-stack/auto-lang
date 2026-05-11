<template>
  <aside class="side-panel">
    <div class="panel-tabs">
      <button
        class="panel-tab"
        :class="{ active: activeTab === 'variables' }"
        @click="activeTab = 'variables'"
      >
        <Database :size="14" />
        Variables
      </button>
      <button
        class="panel-tab"
        :class="{ active: activeTab === 'cells' }"
        @click="activeTab = 'cells'"
      >
        <List :size="14" />
        Cells
      </button>
    </div>
    <div class="panel-content">
      <VariableInspector
        v-if="activeTab === 'variables'"
        :variables="variables"
      />
      <div v-else class="cell-list">
        <div
          v-for="cell in cells"
          :key="cell.id"
          class="cell-list-item"
          :class="cell.type"
        >
          <span class="cell-list-id">{{ cell.id }}</span>
          <span class="cell-list-type">{{ cell.type }}</span>
          <span
            class="cell-list-status"
            :class="cell.status"
          />
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Database, List } from 'lucide-vue-next'
import VariableInspector from '../notebook/VariableInspector.vue'
import type { Cell, VariableInfo } from '@/types/cell'

defineProps<{
  variables: VariableInfo[]
  cells: Cell[]
}>()

const activeTab = ref<'variables' | 'cells'>('variables')
</script>

<style scoped>
.side-panel {
  width: 260px;
  flex-shrink: 0;
  background: #181825;
  border-left: 1px solid #313244;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-tabs {
  display: flex;
  border-bottom: 1px solid #313244;
}

.panel-tab {
  flex: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.35rem;
  padding: 0.5rem;
  background: transparent;
  color: #6c7086;
  border: none;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.15s;
}

.panel-tab.active {
  color: #cdd6f4;
  border-bottom: 2px solid #6366f1;
}

.panel-tab:hover {
  color: #cdd6f4;
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
}

.cell-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.cell-list-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.35rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  background: #1e1e2e;
}

.cell-list-id {
  font-family: 'JetBrains Mono', monospace;
  color: #6366f1;
  font-size: 0.75rem;
  min-width: 2rem;
}

.cell-list-type {
  flex: 1;
  color: #6c7086;
  text-transform: capitalize;
}

.cell-list-status {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #45475a;
}

.cell-list-status.running {
  background: #f9e2af;
  animation: pulse 1s infinite;
}

.cell-list-status.success {
  background: #27c93f;
}

.cell-list-status.error {
  background: #f38ba8;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
