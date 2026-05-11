export interface ForgeMessage {
  id: string
  role: 'user' | 'assistant' | 'system' | 'tool'
  content: string
  timestamp: number
  tool_calls?: ToolCall[]
  pending?: boolean
}

export interface ToolCall {
  id: string
  name: string
  arguments: Record<string, unknown>
  result?: string
  status: 'pending' | 'running' | 'success' | 'error'
}

export interface ForgeSession {
  id: string
  project_path: string
  active_role: string
  status: 'idle' | 'thinking' | 'tool_call' | 'waiting_approval' | 'error'
  messages: ForgeMessage[]
}
