<template>
  <div class="autolab-app">
    <NotebookToolbar
      :file-path="filePath"
      :unsaved="unsaved"
      @new-notebook="onNewNotebook"
      @open-file="onOpenFile"
      @save="onSave"
      @run-all="runAll"
    />
    <div class="autolab-body">
      <main class="cell-area">
        <CellCanvas
          :cells="cells"
          @execute="executeCell"
          @add-cell="addCell"
          @delete-cell="deleteCell"
          @move-cell="(p) => moveCell(p.id, p.direction)"
          @update-cell="(p) => updateCell(p.id, p.patch)"
        />
        <AIChatBar @submit="onAIChatSubmit" />
      </main>
      <SidePanel
        :variables="variables"
        :cells="cells"
      />
    </div>
  </div>
  <input
    ref="fileInput"
    type="file"
    accept=".ad"
    style="display: none"
    @change="onFileSelected"
  />
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useNotebook } from './composables/useNotebook'
import NotebookToolbar from './components/layout/NotebookToolbar.vue'
import CellCanvas from './components/cells/CellCanvas.vue'
import AIChatBar from './components/notebook/AIChatBar.vue'
import SidePanel from './components/layout/SidePanel.vue'
import type { Cell, CellType } from './types/cell'

const {
  cells, variables, filePath, unsaved,
  executeCell, addCell, deleteCell, moveCell, runAll,
  loadFromAd, serializeToAd, saveToFile, loadFromFile, askAI,
} = useNotebook()

const fileInput = ref<HTMLInputElement | null>(null)

function onNewNotebook() {
  cells.value = [{
    id: 'c1',
    type: 'code',
    source: '',
    status: 'idle',
    collapsed: false,
    depends_on: [],
  }]
}

function onOpenFile() {
  fileInput.value?.click()
}

async function onFileSelected(e: Event) {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return
  await loadFromFile(file)
  target.value = ''
}

function onSave() {
  const name = filePath.value || 'notebook.ad'
  saveToFile(name)
}

function updateCell(id: string, patch: Partial<Cell>) {
  const cell = cells.value.find((c) => c.id === id)
  if (cell) {
    Object.assign(cell, patch)
  }
}

async function onAIChatSubmit(content: string) {
  // Add user AI request cell
  const userCellId = addCell('ai')
  const userCell = cells.value.find((c) => c.id === userCellId)
  if (userCell) {
    userCell.source = content
  }

  // Add assistant response cell (placeholder)
  const assistantCellId = addCell('ai')
  const assistantCell = cells.value.find((c) => c.id === assistantCellId)
  if (assistantCell) {
    assistantCell.source = '> Thinking...'
    assistantCell.status = 'running'
  }

  // Call backend AI endpoint
  const response = await askAI(content)
  if (assistantCell) {
    assistantCell.source = response
    assistantCell.status = 'success'
  }
}
</script>

<style>
* {
  box-sizing: border-box;
}

html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
  background: #0f0f14;
  color: #cdd6f4;
}

.autolab-app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.autolab-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.cell-area {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  overflow: hidden;
}
</style>
