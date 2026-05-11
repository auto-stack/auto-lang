<template>
  <div class="cell-item" :class="[cell.type, cell.status]">
    <CellToolbar
      :cell="cell"
      :index="index"
      :total="total"
      @execute="emit('execute', cell)"
      @delete="emit('delete', cell.id)"
      @move="(dir) => emit('move', { id: cell.id, direction: dir })"
      @toggle-collapse="emit('update', { id: cell.id, patch: { collapsed: !cell.collapsed } })"
      @change-type="(t) => emit('update', { id: cell.id, patch: { type: t as CellType, source: defaultSourceFor(t as CellType) } })"
      @add-after="emit('add-after', cell.id)"
    />
    <div v-if="!cell.collapsed" class="cell-body">
      <CellEditor
        :cell="cell"
        @update="(src) => emit('update', { id: cell.id, patch: { source: src } })"
      />
      <CellOutput
        v-if="cell.output"
        :output="cell.output"
        :status="cell.status"
        :cell-type="cell.type"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import CellToolbar from './CellToolbar.vue'
import CellEditor from './CellEditor.vue'
import CellOutput from './CellOutput.vue'
import type { Cell, CellType } from '@/types/cell'

const props = defineProps<{
  cell: Cell
  index: number
  total: number
}>()

const emit = defineEmits<{
  execute: [cell: Cell]
  delete: [id: string]
  move: [payload: { id: string; direction: 'up' | 'down' }]
  update: [payload: { id: string; patch: Partial<Cell> }]
  'add-after': [id: string]
}>()

function defaultSourceFor(type: CellType): string {
  switch (type) {
    case 'code': return ''
    case 'markdown': return '# New Section\n\nWrite markdown here.'
    case 'ai': return ''
    case 'chart': return '$Chart(type: "bar", data: [1, 2, 3])'
    default: return ''
  }
}
</script>

<style scoped>
.cell-item {
  border: 1px solid #313244;
  border-radius: 8px;
  background: #1e1e2e;
  overflow: hidden;
  transition: border-color 0.2s;
}

.cell-item.running {
  border-color: #f9e2af;
}

.cell-item.success {
  border-color: #27c93f44;
}

.cell-item.error {
  border-color: #f38ba844;
}

.cell-body {
  display: flex;
  flex-direction: column;
}
</style>
