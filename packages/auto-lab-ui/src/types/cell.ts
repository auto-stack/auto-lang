export type CellStatus = 'idle' | 'running' | 'success' | 'error' | 'dirty'

export type CellType = 'code' | 'markdown' | 'ai' | 'chart' | 'table'

export interface CellOutput {
  stdout: string
  stderr: string
  result: string
  time_ms: number
}

export interface Cell {
  id: string
  type: CellType
  source: string
  output?: CellOutput
  status: CellStatus
  collapsed: boolean
  depends_on: string[]
}

export interface VariableInfo {
  name: string
  kind: string
}

export interface NotebookSession {
  session_id: string
  cells: Cell[]
  variables: VariableInfo[]
}

// ============================================================================
// Cell Type Registry
// ============================================================================

export interface CellTypeDef {
  id: CellType
  label: string
  icon: string // lucide icon name
  defaultSource: string
}

export const BUILTIN_CELL_TYPES: CellTypeDef[] = [
  { id: 'code', label: 'Code', icon: 'Code', defaultSource: '' },
  { id: 'markdown', label: 'Markdown', icon: 'FileText', defaultSource: '# New Section\n\nWrite markdown here.' },
  { id: 'ai', label: 'AI', icon: 'Bot', defaultSource: '' },
  { id: 'chart', label: 'Chart', icon: 'BarChart3', defaultSource: '{\n  "type": "bar",\n  "data": [10, 25, 15, 30, 20],\n  "labels": ["A", "B", "C", "D", "E"]\n}' },
  { id: 'table', label: 'Table', icon: 'Table', defaultSource: '[\n  {"name": "Alice", "age": 30},\n  {"name": "Bob", "age": 25}\n]' },
]
