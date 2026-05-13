<template>
  <div class="furnace-view">
    <!-- Session Sidebar -->
    <aside class="session-sidebar" :class="{ collapsed: sidebarCollapsed }">
      <div class="sidebar-header">
        <span class="sidebar-title">Sessions</span>
        <button class="sidebar-new-btn" @click="clearSession" title="New session">
          <Plus :size="14" />
        </button>
        <button class="sidebar-collapse-btn" @click="sidebarCollapsed = !sidebarCollapsed" title="Toggle sidebar">
          <PanelLeft :size="14" />
        </button>
      </div>
      <div class="session-list">
        <div
          v-for="s in sessionList"
          :key="s.id"
          class="session-item"
          :class="{ active: sessionId === s.id }"
          @click="switchSession(s.id)"
        >
          <div v-if="editingSessionId !== s.id" class="session-preview">{{ s.name || s.preview || 'New session' }}</div>
          <input
            v-else
            :data-rename-input="s.id"
            v-model="editingName"
            class="session-rename-input"
            @keydown.enter="commitRename"
            @keydown.escape="cancelRename"
            @blur="commitRename"
            @click.stop
          />
          <div class="session-meta">
            <span class="session-count">{{ s.message_count }} msgs</span>
            <span class="session-phase" :class="s.phase">{{ s.phase }}</span>
            <span class="session-status" :class="s.status">{{ s.status }}</span>
            <button
              v-if="editingSessionId !== s.id"
              class="session-rename-btn"
              title="Rename session"
              @click.stop="startRename(s)"
            >
              <Pencil :size="11" />
            </button>
            <button
              class="session-delete-btn"
              title="Delete session"
              @click.stop="confirmDelete(s.id)"
            >
              <Trash2 :size="12" />
            </button>
          </div>
        </div>
        <div v-if="sessionList.length === 0" class="session-empty">
          No sessions yet
        </div>
      </div>
    </aside>

    <!-- Main Chat Area -->
    <div class="furnace-body">
      <div class="furnace-header">
        <div class="header-title-row">
          <button v-if="sidebarCollapsed" class="sidebar-toggle-btn" @click="sidebarCollapsed = false" title="Show sessions">
            <PanelLeft :size="16" />
          </button>
          <h2>Chat</h2>
        </div>
        <div class="header-center">
          <div v-if="projectName" class="header-project">{{ projectName }}</div>
          <div class="header-search">
            <Search :size="13" />
            <input
              v-model="chatSearch"
              type="text"
              class="search-input"
              placeholder="Search messages..."
            />
          </div>
        </div>
        <div class="header-actions">
          <span class="session-badge phase" :class="sessionPhase">
            {{ sessionPhase }}
          </span>
          <span class="session-badge" :class="sessionStatus">
            {{ sessionStatus }}
          </span>
        </div>
      </div>
      <div class="chat-canvas" ref="chatRef">
        <div class="chat-inner">
          <div
            v-for="msg in filteredMessages"
            :key="msg.id"
            class="message"
            :class="msg.role"
          >
            <div class="message-header">
              <span class="role-badge" :class="msg.role">{{ msg.role }}</span>
              <span class="msg-time">{{ formatTime(msg.timestamp) }}</span>
            </div>
            <div class="message-content" :class="{ 'has-border': msg.role === 'assistant' && msg.content.length > 200 }">
              <StreamingRenderer
                v-if="msg.role === 'assistant'"
                :source="msg.content"
                :streaming="isStreamingMessage(msg)"
              />
              <div v-else-if="msg.role === 'system'" class="system-welcome">
                <span class="welcome-icon">👋</span>
                <span>Hi! I'm <strong>AutoForge</strong>, a spec-driven AI coding assistant. I'm glad to help you build your next great project!</span>
              </div>
              <div v-else-if="msg.content">{{ msg.content }}</div>
            </div>
            <div v-if="msg.tool_calls && msg.tool_calls.length > 0" class="tool-calls">
              <div
                v-for="tc in msg.tool_calls"
                :key="tc.id"
                class="tool-card"
                :class="tc.status"
              >
                <div class="tool-header" @click="tc._expanded = !tc._expanded">
                  <span class="tool-icon">🔧</span>
                  <span class="tool-name">{{ tc.name }}</span>
                  <span class="tool-status" :class="tc.status">{{ tc.status }}</span>
                  <ChevronDown v-if="!tc._expanded" :size="14" class="tool-chevron" />
                  <ChevronUp v-else :size="14" class="tool-chevron" />
                </div>
                <div v-if="tc._expanded" class="tool-body">
                  <div class="tool-section">
                    <div class="tool-section-title">Arguments</div>
                    <pre class="tool-code">{{ JSON.stringify(tc.arguments, null, 2) }}</pre>
                  </div>
                  <div v-if="tc.result" class="tool-section">
                    <div class="tool-section-title">Result</div>
                    <pre class="tool-code result">{{ tc.result }}</pre>
                  </div>
                </div>
              </div>
            </div>
            <!-- Message toolbar -->
            <div v-if="msg.role === 'user'" class="message-toolbar">
              <button class="toolbar-btn" title="Copy" @click="copyText(msg.content)">
                <Clipboard :size="13" />
              </button>
            </div>
            <div v-else-if="msg.role === 'assistant'" class="message-toolbar">
              <button class="toolbar-btn" title="Copy" @click="copyText(msg.content)">
                <Clipboard :size="13" />
              </button>
              <button class="toolbar-btn" title="Regenerate" @click="regenerate(msg)">
                <RefreshCw :size="13" />
              </button>
            </div>
          </div>
          <div v-if="isLoading && !hasPendingAssistant" class="message assistant pending">
            <div class="message-header">
              <span class="role-badge assistant">assistant</span>
            </div>
            <div class="message-content">
              <span class="typing">Thinking</span>
              <span class="typing-dots">...</span>
            </div>
          </div>
          <div v-if="error" class="message error">
            <div class="message-content error">
              {{ error }}
            </div>
          </div>
        </div>
      </div>
      <!-- Approval Gate -->
      <div v-if="needsApproval" class="approval-gate">
        <div class="approval-message">
          <span class="approval-icon">📋</span>
          <span>Spec drafted. Review the proposed Specs changes below.</span>
        </div>
        <div v-if="pendingSpecChanges.length > 0" class="approval-diff-list">
          <div
            v-for="change in pendingSpecChanges"
            :key="change.section_id"
            class="diff-card"
          >
            <div class="diff-header" @click="toggleDiff(change.section_id)">
              <span class="diff-title">{{ change.section_id }}</span>
              <span class="diff-status" :class="change.new_status">
                {{ change.old_status }} → {{ change.new_status }}
              </span>
              <ChevronDown v-if="!expandedDiffs.has(change.section_id)" :size="14" class="diff-chevron" />
              <ChevronUp v-else :size="14" class="diff-chevron" />
            </div>
            <div v-if="expandedDiffs.has(change.section_id)" class="diff-body">
              <div class="diff-side">
                <div class="diff-label">Before</div>
                <pre class="diff-content old">{{ change.old_content }}</pre>
              </div>
              <div class="diff-side">
                <div class="diff-label">After</div>
                <textarea
                  v-model="editedSpecs[change.section_id]"
                  class="diff-editor"
                  rows="6"
                />
              </div>
            </div>
          </div>
        </div>
        <div class="approval-actions">
          <button class="approve-btn" @click="handleApprove">
            <Check :size="14" />
            Approve & Execute
          </button>
          <button class="reject-btn" @click="handleReject">
            <X :size="14" />
            Reject & Redraft
          </button>
        </div>
      </div>
      <div v-else class="furnace-input-bar">
        <div class="input-inner">
          <div class="input-row">
            <textarea
              v-model="inputText"
              class="furnace-input"
              placeholder="Describe what you want to build... (Shift+Enter to send)"
              :disabled="isLoading"
              @keydown.shift.enter.prevent="sendMessage"
            />
            <button
              class="send-btn"
              :disabled="!inputText.trim() || isLoading"
              @click="sendMessage"
            >
              <Send :size="16" />
            </button>
          </div>
          <div class="input-extras">
            <div class="model-select-wrap">
              <Cpu :size="12" />
              <select v-model="selectedModel" class="model-select" title="Model">
                <option v-for="m in modelOptions" :key="m.value" :value="m.value">{{ m.label }}</option>
              </select>
            </div>
            <button
              class="thinking-toggle"
              :class="{ active: thinkingMode }"
              @click="thinkingMode = !thinkingMode"
              title="Toggle thinking mode"
            >
              <Lightbulb :size="12" />
              <span>Thinking</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import {
  Send, ChevronDown, ChevronUp, Plus, PanelLeft,
  Check, X, Clipboard, RefreshCw, Lightbulb, Cpu, Search, Trash2, Pencil,
} from 'lucide-vue-next'
import { useForge } from '@/composables/useForge'
import StreamingRenderer from '@/components/StreamingRenderer.vue'

const {
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
  resume,
  switchSession,
  clearSession,
  loadSessionList,
  sendMessage: forgeSendMessage,
  streamResponse,
  approveSpec,
  rejectSpec,
  renameSession,
  deleteSession,
} = useForge()

const expandedDiffs = ref<Set<string>>(new Set())
const editedSpecs = ref<Record<string, string>>({})
const chatSearch = ref('')
const thinkingMode = ref(false)

const projectName = computed(() => {
  const path = session.value?.project_path
  if (!path || path === '.') return null
  // Extract last dir name from path
  const parts = path.replace(/\\/g, '/').split('/').filter(Boolean)
  return parts.length > 0 ? parts[parts.length - 1] : null
})
const selectedModel = ref('glm-5')

const filteredMessages = computed(() => {
  const q = chatSearch.value.trim().toLowerCase()
  if (!q) return messages.value
  return messages.value.filter((m) =>
    m.content.toLowerCase().includes(q) ||
    m.role.toLowerCase().includes(q)
  )
})
const modelOptions = [
  { value: 'glm-5', label: 'GLM-5' },
  { value: 'glm-5.1', label: 'GLM-5.1' },
  { value: 'glm-5-turbo', label: 'GLM-5 Turbo' },
  { value: 'glm-4.7', label: 'GLM-4.7' },
]

function toggleDiff(sectionId: string) {
  if (expandedDiffs.value.has(sectionId)) {
    expandedDiffs.value.delete(sectionId)
  } else {
    expandedDiffs.value.add(sectionId)
  }
}

watch(pendingSpecChanges, (changes) => {
  for (const change of changes) {
    if (!(change.section_id in editedSpecs.value)) {
      editedSpecs.value[change.section_id] = change.new_content
    }
  }
}, { immediate: true })

const CHAT_SIDEBAR_KEY = 'autoforge-chat-sidebar-collapsed'

const inputText = ref('')
const chatRef = ref<HTMLDivElement>()
const sidebarCollapsed = ref(localStorage.getItem(CHAT_SIDEBAR_KEY) === 'true')

watch(sidebarCollapsed, (v) => {
  localStorage.setItem(CHAT_SIDEBAR_KEY, String(v))
})

const hasPendingAssistant = computed(() => {
  return messages.value.some((m) => m.role === 'assistant' && m.content === '' && !m.tool_calls?.length)
})

const lastAssistantMessage = computed(() => {
  for (let i = messages.value.length - 1; i >= 0; i--) {
    if (messages.value[i].role === 'assistant') {
      return messages.value[i]
    }
  }
  return null
})

function isStreamingMessage(msg: typeof messages.value[number]): boolean {
  return isLoading.value && msg === lastAssistantMessage.value
}

function formatTime(ts: number): string {
  return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

async function scrollToBottom() {
  await nextTick()
  if (chatRef.value) {
    chatRef.value.scrollTop = chatRef.value.scrollHeight
  }
}

watch(messages, scrollToBottom, { deep: true })

async function sendMessage() {
  const text = inputText.value.trim()
  if (!text) return
  inputText.value = ''
  await forgeSendMessage(text)
}

async function handleApprove() {
  await approveSpec(editedSpecs.value)
  await streamResponse()
}

async function handleReject() {
  await rejectSpec()
}

const editingSessionId = ref<string | null>(null)
const editingName = ref('')

function startRename(s: { id: string; name?: string; preview: string }) {
  editingSessionId.value = s.id
  editingName.value = s.name || s.preview
  nextTick(() => {
    const el = document.querySelector<HTMLInputElement>(`[data-rename-input="${s.id}"]`)
    el?.focus()
    el?.select()
  })
}

async function commitRename() {
  const sid = editingSessionId.value
  if (!sid) return
  const name = editingName.value.trim()
  if (name) {
    await renameSession(sid, name)
  }
  editingSessionId.value = null
  editingName.value = ''
}

function cancelRename() {
  editingSessionId.value = null
  editingName.value = ''
}

async function confirmDelete(sid: string) {
  const ok = confirm('Delete this session? All messages and memory will be lost.')
  if (!ok) return
  await deleteSession(sid)
}

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    // fallback
    const ta = document.createElement('textarea')
    ta.value = text
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
  }
}

function regenerate(_msg: typeof messages.value[number]) {
  // TODO: wire up to backend regenerate endpoint
  alert('Regenerate: not yet implemented')
}

onMounted(async () => {
  if (!session.value) {
    await resume()
  }
  await loadSessionList()
})
</script>

<style scoped>
.furnace-view {
  display: flex;
  flex-direction: row;
  height: 100%;
  overflow: hidden;
}

/* ─── Session Sidebar ─────────────────────────────────────────────────────── */

.session-sidebar {
  width: 220px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: transparent;
  border-right: 1px solid var(--af-border);
  transition: width 0.2s ease, margin-left 0.2s ease;
}

.session-sidebar.collapsed {
  width: 0;
  margin-left: -1px;
  overflow: hidden;
}

.sidebar-header {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.75rem 1rem;
  flex-shrink: 0;
  height: 48px;
}

.sidebar-title {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--af-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  flex: 1;
  line-height: 1;
}

.sidebar-new-btn,
.sidebar-collapse-btn,
.sidebar-toggle-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  background: transparent;
  border: none;
  border-radius: 5px;
  color: var(--af-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.sidebar-new-btn:hover,
.sidebar-collapse-btn:hover,
.sidebar-toggle-btn:hover {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
}

.session-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.session-item {
  padding: 0.5rem 0.6rem;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.session-item:hover {
  background: hsl(var(--muted-foreground) / 0.05);
}

.session-item.active {
  background: hsl(var(--primary) / 0.06);
  border-left: 2px solid var(--af-primary);
  margin-left: -2px;
}

.session-preview {
  font-size: 0.8rem;
  color: var(--af-fg);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.session-meta {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin-top: 0.2rem;
}

.session-rename-btn,
.session-delete-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--af-muted);
  cursor: pointer;
  opacity: 0;
  transition: all 0.15s;
  flex-shrink: 0;
}

.session-rename-btn {
  margin-left: auto;
}

.session-item:hover .session-rename-btn,
.session-item:hover .session-delete-btn {
  opacity: 1;
}

.session-rename-btn:hover {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
}

.session-delete-btn:hover {
  background: hsl(var(--af-error) / 0.1);
  color: hsl(var(--af-error));
}

.session-rename-input {
  width: 100%;
  font-size: 0.8rem;
  color: var(--af-fg);
  background: hsl(var(--muted-foreground) / 0.06);
  border: 1px solid hsl(var(--primary) / 0.35);
  border-radius: 4px;
  padding: 0.2rem 0.4rem;
  outline: none;
  font-family: inherit;
  line-height: 1.4;
}

.session-count {
  font-size: 0.65rem;
  color: var(--af-muted);
}

.session-status,
.session-phase {
  font-size: 0.6rem;
  font-weight: 500;
  color: var(--af-muted);
}

.session-status.idle { color: hsl(var(--af-success)); }
.session-status.thinking { color: hsl(var(--af-warning)); }
.session-status.tool_call { color: hsl(var(--af-info)); }
.session-status.error { color: hsl(var(--af-error)); }

.session-phase.intake { color: hsl(var(--af-info)); }
.session-phase.spec_draft { color: hsl(var(--af-warning)); }
.session-phase.spec_review { color: hsl(var(--af-furnace)); }
.session-phase.execution { color: hsl(var(--af-success)); }
.session-phase.verification { color: hsl(var(--af-accent)); }

.session-empty {
  font-size: 0.8rem;
  color: var(--af-muted);
  text-align: center;
  padding: 1rem 0;
}

/* ─── Chat Area ───────────────────────────────────────────────────────────── */

.furnace-body {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  min-height: 0;
}

.furnace-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  flex-shrink: 0;
  height: 48px;
}

.header-title-row {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  flex-shrink: 0;
  width: 80px;
}

.furnace-header h2 {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--af-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  line-height: 1;
}

.header-center {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.15rem;
  flex: 1;
  max-width: 520px;
  margin: 0 auto;
}

.header-project {
  font-size: 0.7rem;
  font-weight: 500;
  color: var(--af-primary);
  line-height: 1;
}

.header-search {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  width: 100%;
  padding: 0.35rem 0.75rem;
  background: hsl(var(--muted-foreground) / 0.06);
  border: 1px solid hsl(var(--muted-foreground) / 0.12);
  border-radius: 6px;
  color: var(--af-muted);
  transition: border-color 0.15s, background 0.15s;
}

.header-search:focus-within {
  border-color: hsl(var(--primary) / 0.35);
  background: hsl(var(--muted-foreground) / 0.04);
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--af-fg);
  font-size: 0.85rem;
  font-family: inherit;
  min-width: 0;
}

.search-input::placeholder {
  color: var(--af-muted);
  font-size: 0.8rem;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.session-badge {
  font-size: 0.7rem;
  font-weight: 500;
  color: var(--af-muted);
}

.session-badge.idle { color: hsl(var(--af-success)); }
.session-badge.thinking { color: hsl(var(--af-warning)); }
.session-badge.tool_call { color: hsl(var(--af-info)); }
.session-badge.waiting_approval { color: var(--af-primary); }
.session-badge.error { color: hsl(var(--af-error)); }

.chat-canvas {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem 1rem;
  display: flex;
  flex-direction: column;
}

.chat-inner {
  max-width: 960px;
  width: 100%;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.message {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.message.user {
  align-self: flex-end;
  max-width: 85%;
}

.message.assistant,
.message.system {
  align-self: flex-start;
  max-width: 100%;
}

.message.error {
  align-self: center;
  max-width: 100%;
}

.message-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0 0.25rem;
}

.role-badge {
  font-size: 0.7rem;
  font-weight: 500;
  color: var(--af-muted);
}

.role-badge.user {
  color: var(--af-primary);
  font-weight: 600;
}

.msg-time {
  font-size: 0.65rem;
  color: var(--af-muted);
}

.message-content {
  font-size: 0.9rem;
  line-height: 1.6;
  color: var(--af-fg);
  white-space: pre-wrap;
  word-break: break-word;
  padding: 0.25rem 0;
}

/* Override markstream-vue heading sizes to match body text scale */
.message-content :deep(.markstream-vue) {
  --ms-text-body: 0.9rem;
}

.message-content :deep(h1),
.message-content :deep(h2),
.message-content :deep(h3),
.message-content :deep(h4) {
  font-size: 0.95rem;
  font-weight: 600;
  margin: 0.75rem 0 0.35rem;
  line-height: 1.4;
}

.message-content :deep(p) {
  margin: 0.35rem 0;
}

.message-content :deep(ul),
.message-content :deep(ol) {
  margin: 0.35rem 0;
  padding-left: 1.25rem;
}

.message-content :deep(li) {
  margin: 0.15rem 0;
}

.message-content :deep(pre) {
  margin: 0.5rem 0;
}

.message-content :deep(hr) {
  margin: 0.75rem 0;
  border: none;
  border-top: 1px solid var(--af-border);
}

.message-content.has-border {
  border-top: 1px solid var(--af-border);
  border-bottom: 1px solid var(--af-border);
  padding: 0.5rem 0;
  margin: 0.25rem 0;
}

.message-content.error {
  color: hsl(var(--af-error));
  font-size: 0.85rem;
}

.message.user .message-content {
  background: hsl(var(--primary) / 0.06);
  border-radius: 12px;
  padding: 0.6rem 0.9rem;
  max-width: 100%;
  font-size: 0.9rem;
}

.message.system .message-content {
  font-size: 0.85rem;
}

.system-welcome {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  color: var(--af-muted);
  line-height: 1.5;
}

.welcome-icon {
  font-size: 1.1rem;
  flex-shrink: 0;
}

/* ─── Message Toolbar ─────────────────────────────────────────────────────── */

.message-toolbar {
  display: flex;
  align-items: center;
  gap: 0.15rem;
  padding: 0.1rem 0.25rem;
  opacity: 0;
  transition: opacity 0.15s;
}

.message:hover .message-toolbar {
  opacity: 1;
}

.toolbar-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--af-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.toolbar-btn:hover {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
}

/* ─── Tool Cards ──────────────────────────────────────────────────────────── */

.tool-calls {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  margin-top: 0.15rem;
}

.tool-card {
  background: transparent;
  border: 1px solid var(--af-border);
  border-radius: 8px;
  overflow: hidden;
}

.tool-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.6rem;
  cursor: pointer;
  user-select: none;
}

.tool-header:hover {
  background: hsl(var(--muted-foreground) / 0.03);
}

.tool-icon {
  font-size: 0.85rem;
}

.tool-name {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--af-fg);
  flex: 1;
}

.tool-status {
  font-size: 0.65rem;
  font-weight: 500;
  color: var(--af-muted);
}

.tool-status.pending { color: hsl(var(--af-warning)); }
.tool-status.running { color: hsl(var(--af-info)); }
.tool-status.success { color: hsl(var(--af-success)); }
.tool-status.error { color: hsl(var(--af-error)); }

.tool-chevron {
  color: var(--af-muted);
}

.tool-body {
  border-top: 1px solid var(--af-border);
  padding: 0.5rem 0.6rem;
}

.tool-section {
  margin-bottom: 0.4rem;
}

.tool-section:last-child {
  margin-bottom: 0;
}

.tool-section-title {
  font-size: 0.65rem;
  font-weight: 500;
  text-transform: uppercase;
  color: var(--af-muted);
  margin-bottom: 0.2rem;
  letter-spacing: 0.02em;
}

.tool-code {
  font-size: 0.75rem;
  color: var(--af-muted);
  background: hsl(var(--muted-foreground) / 0.04);
  padding: 0.35rem 0.5rem;
  border-radius: 4px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-word;
}

.tool-code.result {
  color: hsl(var(--af-success));
}

.typing {
  color: var(--af-muted);
  font-size: 0.85rem;
}

.typing-dots {
  animation: blink 1.4s infinite both;
}

@keyframes blink {
  0%, 80%, 100% { opacity: 0; }
  40% { opacity: 1; }
}

/* ─── Input Bar ───────────────────────────────────────────────────────────── */

.furnace-input-bar {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0.6rem 1rem 0.75rem;
  flex-shrink: 0;
}

.input-inner {
  width: 100%;
  max-width: 960px;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.input-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.furnace-input {
  flex: 1;
  background: hsl(var(--muted-foreground) / 0.04);
  border: 1px solid hsl(var(--primary) / 0.18);
  border-radius: 20px;
  padding: 0.55rem 1rem;
  color: var(--af-fg);
  font-size: 0.9rem;
  resize: none;
  min-height: 80px;
  max-height: 180px;
  outline: none;
  font-family: inherit;
  transition: border-color 0.15s, background 0.15s, box-shadow 0.15s;
}

.furnace-input:focus {
  border-color: hsl(var(--primary) / 0.45);
  background: var(--af-bg);
  box-shadow: 0 0 0 3px hsl(var(--primary) / 0.08);
}

.furnace-input::placeholder {
  color: var(--af-muted);
}

.furnace-input:disabled {
  opacity: 0.5;
}

.send-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: linear-gradient(135deg, var(--vp-c-brand-1) 0%, var(--vp-c-brand-2) 100%);
  border: none;
  border-radius: 50%;
  color: #fff;
  cursor: pointer;
  transition: opacity 0.15s, transform 0.1s;
  flex-shrink: 0;
}

.send-btn:hover:not(:disabled) {
  opacity: 0.85;
}

.send-btn:active:not(:disabled) {
  transform: scale(0.95);
}

.send-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.input-extras {
  display: flex;
  align-items: center;
  padding: 0 0.5rem;
}

.thinking-toggle {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.2rem 0.5rem;
  background: transparent;
  border: 1px solid var(--af-border);
  border-radius: 12px;
  color: var(--af-muted);
  font-size: 0.7rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.thinking-toggle:hover {
  background: hsl(var(--muted-foreground) / 0.05);
  color: var(--af-fg);
}

.thinking-toggle.active {
  background: hsl(var(--primary) / 0.08);
  border-color: hsl(var(--primary) / 0.25);
  color: var(--af-primary);
}

.model-select-wrap {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.2rem 0.5rem;
  background: transparent;
  border: 1px solid var(--af-border);
  border-radius: 12px;
  color: var(--af-muted);
  font-size: 0.7rem;
  cursor: pointer;
  transition: all 0.15s;
}

.model-select-wrap:hover {
  background: hsl(var(--muted-foreground) / 0.05);
  color: var(--af-fg);
}

.model-select {
  appearance: none;
  background: transparent;
  border: none;
  color: inherit;
  font-size: inherit;
  font-family: inherit;
  font-weight: 500;
  cursor: pointer;
  outline: none;
  padding-right: 0.2rem;
}

/* ─── Phase Badge ─────────────────────────────────────────────────────────── */

.session-badge.phase {
  text-transform: capitalize;
  font-weight: 500;
}

.session-badge.phase.intake { color: hsl(var(--af-info)); }
.session-badge.phase.spec_draft { color: hsl(var(--af-warning)); }
.session-badge.phase.spec_review { color: hsl(var(--af-furnace)); }
.session-badge.phase.execution { color: hsl(var(--af-success)); }
.session-badge.phase.verification { color: hsl(var(--af-accent)); }

/* ─── Approval Gate ───────────────────────────────────────────────────────── */

.approval-gate {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  padding: 0.75rem 1.25rem;
  border-top: 1px solid var(--af-border);
  flex-shrink: 0;
}

.approval-message {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.85rem;
  color: var(--af-fg);
}

.approval-icon {
  font-size: 1.1rem;
}

.approval-actions {
  display: flex;
  gap: 0.5rem;
}

.approve-btn,
.reject-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.4rem 0.9rem;
  border: none;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s;
}

.approve-btn {
  background: linear-gradient(135deg, var(--vp-c-brand-1) 0%, var(--vp-c-brand-2) 100%);
  color: #fff;
}

.reject-btn {
  background: transparent;
  color: var(--af-fg);
  border: 1px solid var(--af-border);
}

.approve-btn:hover,
.reject-btn:hover {
  opacity: 0.85;
}

/* ─── Approval Diff View ──────────────────────────────────────────────────── */

.approval-diff-list {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  max-height: 300px;
  overflow-y: auto;
}

.diff-card {
  border: 1px solid var(--af-border);
  border-radius: 8px;
  overflow: hidden;
}

.diff-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.6rem;
  cursor: pointer;
  user-select: none;
}

.diff-header:hover {
  background: hsl(var(--muted-foreground) / 0.03);
}

.diff-title {
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--af-fg);
  text-transform: capitalize;
  flex: 1;
}

.diff-status {
  font-size: 0.65rem;
  font-weight: 500;
  color: hsl(var(--af-warning));
}

.diff-status.approved {
  color: hsl(var(--af-success));
}

.diff-chevron {
  color: var(--af-muted);
}

.diff-body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.5rem;
  padding: 0.5rem 0.6rem;
  background: hsl(var(--muted-foreground) / 0.02);
  border-top: 1px solid var(--af-border);
}

.diff-side {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.diff-label {
  font-size: 0.65rem;
  font-weight: 500;
  text-transform: uppercase;
  color: var(--af-muted);
  letter-spacing: 0.02em;
}

.diff-content {
  font-size: 0.75rem;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  background: var(--af-bg);
  border: 1px solid var(--af-border);
  border-radius: 4px;
  padding: 0.35rem;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--af-fg);
  margin: 0;
}

.diff-content.old {
  color: var(--af-muted);
}

.diff-editor {
  font-size: 0.75rem;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  background: var(--af-bg);
  border: 1px solid var(--af-border);
  border-radius: 4px;
  padding: 0.35rem;
  color: var(--af-fg);
  resize: vertical;
  outline: none;
  width: 100%;
  box-sizing: border-box;
}

.diff-editor:focus {
  border-color: hsl(var(--primary) / 0.4);
}
</style>
