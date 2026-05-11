import { ref, computed } from 'vue'
import type { ForgeMessage, ForgeSession, ForgeStreamEvent } from '@/types/forge'
import type { ToolCallInfo } from '@/types/tool'

const API_BASE = '/api/smith'

export function useForge() {
  const session = ref<ForgeSession | null>(null)
  const messages = ref<ForgeMessage[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const sessionId = computed(() => session.value?.id ?? null)
  const sessionStatus = computed(() => session.value?.status ?? 'idle')

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
      return data.id
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
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
          } else if (data.type === 'done') {
            eventSource.close()
            isLoading.value = false
          } else if (data.type === 'error') {
            eventSource.close()
            assistantMsg.content += `\n\n[Error: ${data.message}]`
            isLoading.value = false
          }
        } catch {
          // Raw text fallback
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

  return {
    session,
    messages,
    isLoading,
    error,
    sessionId,
    sessionStatus,
    createSession,
    sendMessage,
    loadHistory,
  }
}
