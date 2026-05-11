<template>
  <div class="cell-toolbar">
    <div class="toolbar-left">
      <span class="cell-badge" :class="cell.type">
        <component :is="typeIcon" :size="12" />
        {{ cell.type }}
      </span>
      <span class="cell-index">[{{ index + 1 }}]</span>
    </div>
    <div class="toolbar-right">
      <button
        class="icon-btn"
        title="Move up"
        :disabled="index === 0"
        @click="$emit('move', 'up')"
      >
        <ChevronUp :size="14" />
      </button>
      <button
        class="icon-btn"
        title="Move down"
        :disabled="index === total - 1"
        @click="$emit('move', 'down')"
      >
        <ChevronDown :size="14" />
      </button>
      <button class="icon-btn" title="Collapse/Expand" @click="$emit('toggle-collapse')">
        <PanelTopClose v-if="!cell.collapsed" :size="14" />
        <PanelTopOpen v-else :size="14" />
      </button>
      <select
        class="type-select"
        :value="cell.type"
        @change="$emit('change-type', ($event.target as HTMLSelectElement).value)"
      >
        <option value="code">Code</option>
        <option value="markdown">Markdown</option>
        <option value="ai">AI</option>
        <option value="chart">Chart</option>
        <option value="table">Table</option>
      </select>
      <button class="icon-btn run-btn" title="Run cell" @click="$emit('execute')">
        <Play :size="14" />
      </button>
      <button class="icon-btn" title="Add cell after" @click="$emit('add-after')">
        <Plus :size="14" />
      </button>
      <button class="icon-btn delete-btn" title="Delete cell" @click="$emit('delete')">
        <Trash2 :size="14" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  Play, ChevronUp, ChevronDown, Trash2, Plus,
  PanelTopClose, PanelTopOpen,
  Code, FileText, Bot, BarChart3, Table,
} from 'lucide-vue-next'
import { computed } from 'vue'
import type { Cell } from '@/types/cell'

const props = defineProps<{
  cell: Cell
  index: number
  total: number
}>()

defineEmits<{
  (e: 'execute'): void
  (e: 'delete'): void
  (e: 'move', direction: 'up' | 'down'): void
  (e: 'toggle-collapse'): void
  (e: 'change-type', type: string): void
  (e: 'add-after'): void
}>()

const typeIcon = computed(() => {
  switch (props.cell.type) {
    case 'code': return Code
    case 'markdown': return FileText
    case 'ai': return Bot
    case 'chart': return BarChart3
    case 'table': return Table
    default: return Code
  }
})
</script>

<style scoped>
.cell-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.35rem 0.5rem;
  background: #181825;
  border-bottom: 1px solid #313244;
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.cell-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.02em;
}

.cell-badge.code {
  background: #6366f122;
  color: #6366f1;
}

.cell-badge.markdown {
  background: #f9e2af22;
  color: #f9e2af;
}

.cell-badge.ai {
  background: #cba6f722;
  color: #cba6f7;
}

.cell-badge.chart {
  background: #89b4fa22;
  color: #89b4fa;
}

.cell-index {
  font-size: 0.75rem;
  color: #45475a;
  font-family: 'JetBrains Mono', monospace;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 0.15rem;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  color: #6c7086;
  border: none;
  border-radius: 4px;
  padding: 0.3rem;
  cursor: pointer;
  transition: all 0.15s;
}

.icon-btn:hover:not(:disabled) {
  background: #313244;
  color: #cdd6f4;
}

.icon-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.run-btn {
  color: #27c93f;
}

.run-btn:hover {
  background: #27c93f22;
}

.delete-btn:hover {
  color: #f38ba8;
  background: #f38ba822;
}

.type-select {
  background: #313244;
  color: #cdd6f4;
  border: 1px solid #45475a;
  border-radius: 4px;
  padding: 0.2rem 0.35rem;
  font-size: 0.75rem;
  cursor: pointer;
  outline: none;
}
</style>
