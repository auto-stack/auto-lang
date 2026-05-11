export interface ToolCallInfo {
  id: string
  name: string
  arguments: Record<string, unknown>
  result?: string
  status: 'pending' | 'running' | 'success' | 'error'
  _expanded?: boolean
}
