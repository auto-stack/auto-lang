import { ref, computed } from 'vue'
import type { ForgeMessage, ForgeSession, ForgeSessionSummary, ForgeStreamEvent } from '@/types/forge'
import type { ToolCallInfo } from '@/types/tool'

const API_BASE = '/api/smith'
const STORAGE_KEY = 'autoforge_session_id'

// ─── Singleton state: persists across component instances ───────────────────
const _session = ref<ForgeSession | null>(null)
const _messages = ref<ForgeMessage[]>([])
const _isLoading = ref(false)
const _error = ref<string | null>(null)
const _sessionList = ref<ForgeSessionSummary[]>([])
const _resuming = ref(false)

export function useForge() {
  const session = _session
  const messages = _messages
  const isLoading = _isLoading
  const error = _error
  const sessionList = _sessionList

  const sessionId = computed(() => session.value?.id ?? null)
  const sessionStatus = computed(() => session.value?.status ?? 'idle')
  const sessionPhase = computed(() => session.value?.phase ?? 'intake')
  const needsApproval = computed(() => sessionStatus.value === 'waiting_approval' && sessionPhase.value === 'spec_review')
  const pendingSpecChanges = computed(() => session.value?.pending_spec_changes ?? [])

  /** Create a brand-new Forge session */
  async function createSession(notebookSid?: string, projectPath?: string) {
    try {
      const resp = await fetch(`${API_BASE}/forge/session`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ notebook_sid: notebookSid, project_path: projectPath }),
      })
      if (!resp.ok) throw new Error(`Failed to create session: ${resp.status}`)
      const data: ForgeSession = await resp.json()
      session.value = data
      messages.value = data.messages
      error.value = null
      localStorage.setItem(STORAGE_KEY, data.id)
      await loadSessionList()
      return data.id
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  /** Restore an existing session by ID (from localStorage or URL) */
  async function restoreSession(sid: string) {
    try {
      const resp = await fetch(`${API_BASE}/forge/session/${sid}`)
      if (!resp.ok) throw new Error(`Session not found: ${resp.status}`)
      const data: ForgeSession | null = await resp.json()
      if (!data) throw new Error('Session returned null')

      session.value = data
      messages.value = data.messages
      error.value = null
      localStorage.setItem(STORAGE_KEY, data.id)
      return data.id
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      localStorage.removeItem(STORAGE_KEY)
      return null
    }
  }

  /** Switch to a different existing session */
  async function switchSession(sid: string) {
    if (sessionId.value === sid) return sid
    const restored = await restoreSession(sid)
    if (restored) {
      await loadSessionList()
    }
    return restored
  }

  /** Start fresh: clear local state and storage, then create a new session */
  async function clearSession() {
    session.value = null
    messages.value = []
    error.value = null
    localStorage.removeItem(STORAGE_KEY)
    await createSession()
  }

  /** Attempt to resume on app load:
   *  1. Check localStorage for a previous session ID
   *  2. Try to restore it from the server
   *  3. Fall back to creating a new session if restoration fails
   */
  async function resume() {
    if (_resuming.value) return _session.value?.id ?? null
    _resuming.value = true
    try {
      const stored = localStorage.getItem(STORAGE_KEY)
      if (stored) {
        const restored = await restoreSession(stored)
        if (restored) return restored
      }
      return await createSession()
    } finally {
      _resuming.value = false
    }
  }

  /** Fetch the list of all sessions from the server */
  async function loadSessionList() {
    try {
      const resp = await fetch(`${API_BASE}/forge/sessions`)
      if (resp.ok) {
        const data: ForgeSessionSummary[] = await resp.json()
        sessionList.value = data
      }
    } catch {
      // ignore
    }
  }

  async function sendMessage(content: string) {
    if (!sessionId.value || isLoading.value) return

    const userMsg: ForgeMessage = {
      id: `u-${Date.now()}`,
      role: 'user',
      content,
      timestamp: Date.now(),
    }
    messages.value.push(userMsg)
    isLoading.value = true
    error.value = null

    try {
      const resp = await fetch(`${API_BASE}/forge/${sessionId.value}/message`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content }),
      })
      if (!resp.ok) throw new Error(`Failed to send message: ${resp.status}`)

      await streamResponse()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      isLoading.value = false
    }
  }

  async function streamResponse() {
    if (!sessionId.value) return

    const assistantMsg: ForgeMessage = {
      id: `a-${Date.now()}`,
      role: 'assistant',
      content: '',
      timestamp: Date.now(),
      tool_calls: [],
    }
    messages.value.push(assistantMsg)

    try {
      const eventSource = new EventSource(`${API_BASE}/forge/${sessionId.value}/stream`)

      eventSource.onmessage = (event) => {
        try {
          const data: ForgeStreamEvent = JSON.parse(event.data)

          if (data.type === 'delta' && data.text) {
            assistantMsg.content += data.text
          } else if (data.type === 'tool_call') {
            const call: ToolCallInfo = {
              id: data.id ?? `tc-${Date.now()}`,
              name: data.name ?? 'unknown',
              arguments: (data.arguments as Record<string, unknown>) ?? {},
              status: 'running',
            }
            assistantMsg.tool_calls = assistantMsg.tool_calls ?? []
            assistantMsg.tool_calls.push(call)
          } else if (data.type === 'tool_result') {
            const call = assistantMsg.tool_calls?.find((c) => c.id === data.id)
            if (call) {
              call.result = data.result ?? ''
              call.status = 'success'
            }
          } else if (data.type === 'phase_change' && data.phase) {
            if (session.value) {
              session.value.phase = data.phase as ForgeSession['phase']
            }
          } else if (data.type === 'done') {
            eventSource.close()
            isLoading.value = false
            // Refresh session to get updated phase/status from server
            if (sessionId.value) {
              restoreSession(sessionId.value)
            }
            loadSessionList() // refresh list so preview updates
          } else if (data.type === 'error') {
            eventSource.close()
            assistantMsg.content += `\n\n[Error: ${data.message}]`
            isLoading.value = false
          }
        } catch {
          assistantMsg.content += event.data
        }
      }

      eventSource.onerror = () => {
        eventSource.close()
        isLoading.value = false
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      isLoading.value = false
    }
  }

  async function loadHistory() {
    if (!sessionId.value) return
    try {
      const resp = await fetch(`${API_BASE}/forge/${sessionId.value}/history`)
      if (resp.ok) {
        const data: ForgeMessage[] = await resp.json()
        if (data.length > 0) messages.value = data
      }
    } catch {
      // Ignore history load errors
    }
  }

  async function approveSpec(editedSpecs?: Record<string, string>) {
    if (!sessionId.value) return
    try {
      const resp = await fetch(`${API_BASE}/forge/${sessionId.value}/approve`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ edited_specs: editedSpecs ?? {} }),
      })
      if (!resp.ok) throw new Error(`Failed to approve: ${resp.status}`)
      const data = await resp.json()
      if (session.value) {
        session.value.phase = data.phase
        session.value.status = 'idle'
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function rejectSpec() {
    if (!sessionId.value) return
    try {
      const resp = await fetch(`${API_BASE}/forge/${sessionId.value}/reject`, {
        method: 'POST',
      })
      if (!resp.ok) throw new Error(`Failed to reject: ${resp.status}`)
      const data = await resp.json()
      if (session.value) {
        session.value.phase = data.phase
        session.value.status = 'idle'
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  return {
    session,
    messages,
    isLoading,
    error,
    sessionList,
    sessionId,
    sessionStatus,
    sessionPhase,
    needsApproval,
    pendingSpecChanges,
    createSession,
    restoreSession,
    switchSession,
    clearSession,
    resume,
    loadSessionList,
    sendMessage,
    loadHistory,
    streamResponse,
    approveSpec,
    rejectSpec,
  }
}
