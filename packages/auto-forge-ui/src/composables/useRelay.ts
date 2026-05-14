import { ref, computed } from 'vue'
import { useEventRouter, type SSEEvent } from './useEventRouter'

const API_BASE = '/api/forge/relay'

// ─── Singleton state ────────────────────────────────────────────────────────
const _runs = ref<RunSummary[]>([])
const _currentRun = ref<RunState | null>(null)
const _professions = ref<ProfessionDto[]>([])
const _souls = ref<SoulDto[]>([])
const _loading = ref(false)
const _error = ref<string | null>(null)
const _liveLog = ref<Array<{ time: string; profession: string; action: string }>>([])
const _professionTokens = ref<Record<string, number>>({})

// ─── Types (mirroring Rust structs) ─────────────────────────────────────────

export interface RunSummary {
  run_id: string
  status: string
  current_step: number
  total_steps: number
  current_profession: string | null
  cumulative_tokens: number
  created_at: number
  updated_at: number
}

export interface RunState {
  run_id: string
  status: string
  current_step: number
  total_steps: number
  steps: StepState[]
  step_history: StepRecord[]
  cumulative_tokens: number
  budget_limit: number
  budget_remaining: number
  waiting_for_gate: GateState | null
  parallel_estimate: number
  savings: number
  savings_ratio: number
}

export interface StepState {
  id: string
  profession_id: string
  status: string
  gate: string
}

export interface StepRecord {
  step_id: string
  profession_id: string
  started_at: number
  completed_at: number
  iteration: number
}

export interface GateState {
  step_id: string
  profession_id: string
  since: number
}

export interface ProfessionDto {
  id: string
  name: string
  phase: string
  owned_sections: string[]
  allowed_tools: string[]
}

export interface SoulDto {
  id: string
  name: string
}

export interface StartRunRequest {
  run_id?: string
  flow_id: string
  steps: { id: string; profession_id: string; gate?: string }[]
}

// ─── Composable ─────────────────────────────────────────────────────────────

export function useRelay() {
  const runs = _runs
  const currentRun = _currentRun
  const professions = _professions
  const souls = _souls
  const loading = _loading
  const error = _error

  const hasActiveGate = computed(() => currentRun.value?.waiting_for_gate != null)
  const runProgress = computed(() => {
    if (!currentRun.value || currentRun.value.total_steps === 0) return 0
    return Math.round((currentRun.value.current_step / currentRun.value.total_steps) * 100)
  })
  const budgetUsedPercent = computed(() => {
    if (!currentRun.value || currentRun.value.budget_limit === 0) return 0
    const used = currentRun.value.budget_limit - currentRun.value.budget_remaining
    return Math.round((used / currentRun.value.budget_limit) * 100)
  })
  const liveLog = _liveLog
  const professionTokens = _professionTokens

  async function loadProfessions() {
    try {
      const resp = await fetch(`${API_BASE}/professions`)
      if (!resp.ok) throw new Error(`Failed: ${resp.status}`)
      const data = await resp.json()
      professions.value = data.professions
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function loadSouls() {
    try {
      const resp = await fetch(`${API_BASE}/souls`)
      if (!resp.ok) throw new Error(`Failed: ${resp.status}`)
      const data = await resp.json()
      souls.value = data.souls
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function loadRuns() {
    try {
      const resp = await fetch(`${API_BASE}/runs`)
      if (!resp.ok) throw new Error(`Failed: ${resp.status}`)
      const data = await resp.json()
      runs.value = data
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function loadRun(runId: string) {
    try {
      const resp = await fetch(`${API_BASE}/runs/${runId}`)
      if (!resp.ok) throw new Error(`Failed: ${resp.status}`)
      const data = await resp.json()
      currentRun.value = data
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function startRun(req: StartRunRequest) {
    loading.value = true
    error.value = null
    try {
      const resp = await fetch(`${API_BASE}/runs`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(req),
      })
      if (!resp.ok) throw new Error(`Failed: ${resp.status}`)
      const data = await resp.json()
      currentRun.value = data.state
      await loadRuns()
      return data.run_id as string
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  async function advanceRun(runId: string) {
    try {
      const resp = await fetch(`${API_BASE}/runs/${runId}/advance`, { method: 'POST' })
      if (!resp.ok) throw new Error(`Failed: ${resp.status}`)
      await loadRun(runId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function resolveGate(runId: string, decision: 'approve' | 'reject' | 'edit', feedback?: string) {
    try {
      const body: any = { decision }
      if (feedback) body.feedback = feedback
      const resp = await fetch(`${API_BASE}/runs/${runId}/gate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      })
      if (!resp.ok) throw new Error(`Failed: ${resp.status}`)
      await loadRun(runId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function submitHandoff(runId: string, handoff: any) {
    try {
      const resp = await fetch(`${API_BASE}/runs/${runId}/handoff`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ handoff }),
      })
      if (!resp.ok) throw new Error(`Failed: ${resp.status}`)
      await loadRun(runId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  // SSE for live updates
  function subscribeToRun(runId: string, onEvent?: (event: any) => void) {
    const eventRouter = useEventRouter()
    const es = new EventSource(`${API_BASE}/runs/${runId}/events`)
    es.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        if (onEvent) onEvent(data)
        // Route through event router for cross-view coordination
        const sseEvent: SSEEvent = {
          type: data.event_type || data.type,
          runId,
          payload: data,
        }
        eventRouter.handleEvent(sseEvent, 'relay')
        // Append to live log
        if (data.event_type === 'handoff_submitted') {
          _liveLog.value.push({
            time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
            profession: data.profession_id || data.from_profession || 'unknown',
            action: `Handoff to ${data.to_profession || 'next'}`,
          })
        }
        if (data.event_type === 'step_advanced') {
          _liveLog.value.push({
            time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
            profession: data.profession_id || 'system',
            action: `Step advanced: ${data.step_id || ''}`,
          })
        }
        // Track per-profession tokens (best-effort from event data)
        if (data.tokens_used && data.profession_id) {
          const prev = _professionTokens.value[data.profession_id] || 0
          _professionTokens.value[data.profession_id] = prev + (data.tokens_used as number)
        }
        // Auto-refresh run state on relevant events
        if (['run_started', 'step_advanced', 'handoff_submitted', 'gate_resolved'].includes(data.event_type)) {
          loadRun(runId)
        }
      } catch {
        // ignore parse errors
      }
    }
    es.onerror = () => {
      // Will auto-reconnect or close
    }
    return () => es.close()
  }

  return {
    runs,
    currentRun,
    professions,
    souls,
    loading,
    error,
    hasActiveGate,
    runProgress,
    budgetUsedPercent,
    liveLog,
    professionTokens,
    loadProfessions,
    loadSouls,
    loadRuns,
    loadRun,
    startRun,
    advanceRun,
    resolveGate,
    submitHandoff,
    subscribeToRun,
  }
}
