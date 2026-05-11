import type { ToolCallInfo } from './tool'

export interface ForgeMessage {
  id: string
  role: 'user' | 'assistant' | 'system' | 'tool'
  content: string
  timestamp: number
  tool_calls?: ToolCallInfo[]
}

export interface ForgeSession {
  id: string
  notebook_sid?: string
  project_path: string
  status: 'idle' | 'thinking' | 'tool_call' | 'waiting_approval' | 'error'
  messages: ForgeMessage[]
}

export interface ForgeStreamEvent {
  type: 'delta' | 'tool_call' | 'tool_result' | 'done' | 'error'
  text?: string
  id?: string
  name?: string
  arguments?: Record<string, unknown>
  result?: string
  message?: string
}

export interface ForgeSessionSummary {
  id: string
  status: 'idle' | 'thinking' | 'tool_call' | 'waiting_approval' | 'error'
  preview: string
  message_count: number
  last_activity: number
}
