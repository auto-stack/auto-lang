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
  current_phase_index?: number | null
  phase_history?: PhaseHistoryEntry[]
}

export interface ForgeStreamEvent {
  type: 'delta' | 'tool_call' | 'tool_result' | 'phase_change' | 'done' | 'error' | 'gate_reached' | 'run_completed'
  text?: string
  id?: string
  name?: string
  arguments?: Record<string, unknown>
  result?: string
  message?: string
  phase?: string
  gate_id?: string
  profession?: string
  title?: string
  section_id?: string
  run_id?: string
  goals_met?: string
  tests_pass?: string
  drift_detected?: string
  cost?: string
  confidence?: 'High' | 'Medium' | 'Low'
  deliverables?: string[]
}

export interface ForgeSessionSummary {
  id: string
  status: 'idle' | 'thinking' | 'tool_call' | 'waiting_approval' | 'error'
  phase: 'intake' | 'spec_draft' | 'spec_review' | 'execution' | 'verification'
  name?: string
  preview: string
  message_count: number
  last_activity: number
}
