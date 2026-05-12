import type { ToolCallInfo } from './tool'

export interface ForgeMessage {
  id: string
  role: 'user' | 'assistant' | 'system' | 'tool'
  content: string
  timestamp: number
  tool_calls?: ToolCallInfo[]
}

export interface SpecChange {
  section_id: string
  old_content: string
  new_content: string
  old_status: string
  new_status: string
}

export interface PhaseHistoryEntry {
  phase: string
  entered_at: number
}

export interface ForgeSession {
  id: string
  notebook_sid?: string
  project_path: string
  status: 'idle' | 'thinking' | 'tool_call' | 'waiting_approval' | 'error'
  phase: 'intake' | 'spec_draft' | 'spec_review' | 'execution' | 'verification'
  messages: ForgeMessage[]
  pending_spec_changes?: SpecChange[]
  current_todo_index?: number | null
  phase_history?: PhaseHistoryEntry[]
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
