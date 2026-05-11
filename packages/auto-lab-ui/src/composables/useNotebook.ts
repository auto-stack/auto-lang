import { ref, computed, watch } from 'vue'
import type { Cell, CellType, CellOutput, VariableInfo } from '@/types/cell'

const API_BASE = '/api/notebook'

let idCounter = 0
function nextCellId(): string {
  return `c${++idCounter}`
}

export function useNotebook() {
  const sessionId = ref<string | null>(null)
  const cells = ref<Cell[]>([
    {
      id: nextCellId(),
      type: 'code',
      source: '// Welcome to AutoLab!\nfn greet(name string) string {\n    f"Hello, ${name}!"\n}\n\nprint(greet("AutoLab"))',
      status: 'idle',
      collapsed: false,
      depends_on: [],
    },
  ])
  const variables = ref<VariableInfo[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const filePath = ref<string | null>(null)
  const unsaved = ref(false)

  const hasSession = computed(() => sessionId.value !== null)

  // Track unsaved changes
  watch(
    cells,
    () => { unsaved.value = true },
    { deep: true }
  )

  async function ensureSession() {
    if (sessionId.value) return
    const res = await fetch(`${API_BASE}/session`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ title: 'Untitled' }),
    })
    const data = await res.json()
    sessionId.value = data.session_id
  }

  // Track last executed source for dirty cell detection
  const lastExecutedSource = ref<Record<string, string>>({})

  function isDirty(cell: Cell): boolean {
    return lastExecutedSource.value[cell.id] !== cell.source
  }

  function markCellDirtyFrom(cellId: string) {
    // Mark the cell and all downstream cells as dirty
    const idx = cells.value.findIndex((c) => c.id === cellId)
    if (idx < 0) return
    for (let i = idx; i < cells.value.length; i++) {
      lastExecutedSource.value[cells.value[i].id] = '__DIRTY__'
    }
  }

  async function executeCell(cell: Cell) {
    await ensureSession()
    if (!sessionId.value) return

    cell.status = 'running'
    cell.output = undefined
    isLoading.value = true
    error.value = null

    try {
      const notebookCells = cells.value.map((c) => ({
        cell_id: c.id,
        source: c.source,
        depends_on: c.depends_on,
      }))

      const res = await fetch(`${API_BASE}/${sessionId.value}/execute`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          cell_id: cell.id,
          source: cell.source,
          notebook_cells: notebookCells,
        }),
      })
      const data: CellOutput = await res.json()
      cell.output = data
      cell.status = data.stderr ? 'error' : 'success'
      // Update snapshot: this cell is now clean
      lastExecutedSource.value[cell.id] = cell.source
      // Also mark upstream cells as clean since they were executed
      // (backend executed dirty upstream cells before target)
      for (const c of cells.value) {
        if (c.id !== cell.id && !isDirty(c)) {
          lastExecutedSource.value[c.id] = c.source
        }
      }
    } catch (e: any) {
      cell.status = 'error'
      cell.output = {
        stdout: '',
        stderr: `Network error: ${e.message}`,
        result: '',
        time_ms: 0,
      }
    } finally {
      isLoading.value = false
      await refreshVariables()
    }
  }

  async function refreshVariables() {
    if (!sessionId.value) return
    try {
      const res = await fetch(`${API_BASE}/${sessionId.value}/variables`)
      const data = await res.json()
      variables.value = data.variables || []
    } catch {
      variables.value = []
    }
  }

  function addCell(type: CellType = 'code', afterId?: string) {
    const newCell: Cell = {
      id: nextCellId(),
      type,
      source: defaultSourceFor(type),
      status: 'idle',
      collapsed: false,
      depends_on: [],
    }

    if (afterId) {
      const idx = cells.value.findIndex((c) => c.id === afterId)
      if (idx >= 0) {
        cells.value.splice(idx + 1, 0, newCell)
        return newCell.id
      }
    }
    cells.value.push(newCell)
    return newCell.id
  }

  function deleteCell(id: string) {
    const idx = cells.value.findIndex((c) => c.id === id)
    if (idx >= 0) {
      cells.value.splice(idx, 1)
    }
    if (cells.value.length === 0) {
      addCell('code')
    }
  }

  function moveCell(id: string, direction: 'up' | 'down') {
    const idx = cells.value.findIndex((c) => c.id === id)
    if (idx < 0) return
    if (direction === 'up' && idx > 0) {
      const tmp = cells.value[idx]
      cells.value[idx] = cells.value[idx - 1]
      cells.value[idx - 1] = tmp
    } else if (direction === 'down' && idx < cells.value.length - 1) {
      const tmp = cells.value[idx]
      cells.value[idx] = cells.value[idx + 1]
      cells.value[idx + 1] = tmp
    }
  }

  function runAll() {
    cells.value.forEach(async (cell) => {
      if (cell.type === 'code') {
        await executeCell(cell)
      }
    })
  }

  async function loadFromAd(source: string) {
    // Parse /// cell: directives
    const lines = source.split('\n')
    const newCells: Cell[] = []
    let currentSource: string[] = []
    let currentType: CellType = 'code'
    let currentId = nextCellId()
    let currentDepends: string[] = []

    function flushCell() {
      if (currentSource.length > 0 || newCells.length === 0) {
        const src = currentSource.join('\n').trim()
        newCells.push({
          id: currentId,
          type: currentType,
          source: src,
          status: 'idle',
          collapsed: false,
          depends_on: currentDepends,
        })
      }
    }

    for (const line of lines) {
      const trimmed = line.trimStart()
      if (trimmed.startsWith('/// cell:')) {
        flushCell()
        const body = trimmed.slice('/// cell:'.length).trim()
        const parts = body.split(/\s+/)
        currentId = parts[0] || nextCellId()
        currentType = 'code'
        currentDepends = []
        currentSource = []
        for (const part of parts.slice(1)) {
          if (part.startsWith('type:')) {
            const t = part.slice('type:'.length)
            if (['code', 'markdown', 'ai', 'chart'].includes(t)) {
              currentType = t as CellType
            }
          } else if (part.startsWith('depends_on:')) {
            currentDepends = part.slice('depends_on:'.length).split(',').map(s => s.trim()).filter(Boolean)
          }
        }
      } else {
        currentSource.push(line)
      }
    }
    flushCell()

    if (newCells.length > 0) {
      cells.value = newCells
    } else {
      cells.value = [{
        id: nextCellId(),
        type: 'markdown',
        source: source.trim(),
        status: 'idle',
        collapsed: false,
        depends_on: [],
      }]
    }
    unsaved.value = false
  }

  function serializeToAd(): string {
    const lines: string[] = []
    for (const cell of cells.value) {
      lines.push(`/// cell:${cell.id} type:${cell.type}${cell.depends_on.length ? ` depends_on:${cell.depends_on.join(',')}` : ''}`)
      lines.push(cell.source)
      lines.push('')
    }
    return lines.join('\n').trim() + '\n'
  }

  async function saveToFile(path: string) {
    // In a real implementation this would call a backend file API
    // For now we use browser download
    const blob = new Blob([serializeToAd()], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = path.endsWith('.ad') ? path : path + '.ad'
    a.click()
    URL.revokeObjectURL(url)
    filePath.value = path
    unsaved.value = false
  }

  async function loadFromFile(file: File): Promise<void> {
    const text = await file.text()
    await loadFromAd(text)
    filePath.value = file.name
  }

  async function askAI(prompt: string): Promise<string> {
    await ensureSession()
    if (!sessionId.value) return ''

    // Build context from preceding code cells
    const context = cells.value
      .filter((c) => c.type === 'code' && c.source.trim())
      .map((c) => `// Cell ${c.id}\n${c.source}`)
      .join('\n\n')

    try {
      const res = await fetch(`${API_BASE}/${sessionId.value}/ai`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ prompt, context }),
      })
      const data = await res.json()
      if (data.error) {
        return `Error: ${data.error}`
      }
      return data.content || ''
    } catch (e: any) {
      return `Error: ${e.message}`
    }
  }

  return {
    sessionId,
    cells,
    variables,
    isLoading,
    error,
    filePath,
    unsaved,
    hasSession,
    ensureSession,
    executeCell,
    refreshVariables,
    addCell,
    deleteCell,
    moveCell,
    runAll,
    loadFromAd,
    serializeToAd,
    saveToFile,
    loadFromFile,
    askAI,
  }
}

function defaultSourceFor(type: CellType): string {
  switch (type) {
    case 'code':
      return ''
    case 'markdown':
      return '# New Section\n\nWrite markdown here.'
    case 'ai':
      return ''
    case 'chart':
      return '{\n  "type": "bar",\n  "data": [10, 25, 15, 30, 20],\n  "labels": ["A", "B", "C", "D", "E"]\n}'
    case 'table':
      return '[\n  {"name": "Alice", "age": 30},\n  {"name": "Bob", "age": 25}\n]'
    default:
      return ''
  }
}
