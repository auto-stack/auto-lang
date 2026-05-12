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
  phase: 'intake' | 'spec_draft' | 'spec_review' | 'execution' | 'verification'
  messages: ForgeMessage[]
}

export interface ForgeStreamEvent {
  type: 'delta' | 'tool_call' | 'tool_result' | 'phase_change' | 'done' | 'error'
  text?: string
  id?: string
  name?: string
  arguments?: Record<string, unknown>
  result?: string
  message?: string
  phase?: string
}

export interface ForgeSessionSummary {
  id: string
  status: 'idle' | 'thinking' | 'tool_call' | 'waiting_approval' | 'error'
  phase: 'intake' | 'spec_draft' | 'spec_review' | 'execution' | 'verification'
  preview: string
  message_count: number
  last_activity: number
}
