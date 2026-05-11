<template>
  <div class="cell-canvas" ref="canvasRef">
    <div
      v-for="(cell, index) in cells"
      :key="cell.id"
      class="cell-wrapper"
    >
      <CellItem
        :cell="cell"
        :index="index"
        :total="cells.length"
        @execute="onExecute"
        @delete="onDelete"
        @move="onMove"
        @update="onUpdate"
        @add-after="onAddAfter"
        @extract-code="onExtractCode"
      />
    </div>
    <button class="add-cell-btn" @click="onAdd">
      <Plus :size="16" />
      Add Cell
    </button>
  </div>
</template>

<script setup lang="ts">
import { Plus } from 'lucide-vue-next'
import CellItem from './CellItem.vue'
import type { Cell, CellType } from '@/types/cell'

const props = defineProps<{
  cells: Cell[]
}>()

const emit = defineEmits<{
  execute: [cell: Cell]
  'add-cell': [type: CellType, afterId?: string]
  'delete-cell': [id: string]
  'move-cell': [payload: { id: string; direction: 'up' | 'down' }]
  'update-cell': [payload: { id: string; patch: Partial<Cell> }]
  'extract-code': [id: string]
}>()

function onExecute(cell: Cell) { emit('execute', cell) }
function onDelete(id: string) { emit('delete-cell', id) }
function onMove(payload: { id: string; direction: 'up' | 'down' }) { emit('move-cell', payload) }
function onUpdate(payload: { id: string; patch: Partial<Cell> }) { emit('update-cell', payload) }
function onAddAfter(id: string) { emit('add-cell', 'code', id) }
function onExtractCode(id: string) { emit('extract-code', id) }
function onAdd() { emit('add-cell', 'code') }
</script>

<style scoped>
.cell-canvas {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.cell-wrapper {
  display: flex;
  flex-direction: column;
}

.add-cell-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.35rem;
  padding: 0.5rem;
  margin: 0.5rem auto;
  background: transparent;
  color: #6c7086;
  border: 1px dashed #45475a;
  border-radius: 6px;
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.15s;
  width: 140px;
}

.add-cell-btn:hover {
  color: #cdd6f4;
  border-color: #6366f1;
  background: #1e1e2e;
}
</style>
