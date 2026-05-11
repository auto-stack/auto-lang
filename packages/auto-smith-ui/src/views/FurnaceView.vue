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
          <div class="session-preview">{{ s.preview || 'New session' }}</div>
          <div class="session-meta">
            <span class="session-count">{{ s.message_count }} msgs</span>
            <span class="session-status" :class="s.status">{{ s.status }}</span>
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
        <h2>The Furnace · 丹炉</h2>
        <div class="header-actions">
          <button v-if="sidebarCollapsed" class="sidebar-toggle-btn" @click="sidebarCollapsed = false" title="Show sessions">
            <PanelLeft :size="16" />
          </button>
          <span class="session-badge" :class="sessionStatus">
            {{ sessionStatus }}
          </span>
        </div>
      </div>
      <div class="chat-canvas" ref="chatRef">
        <div
          v-for="msg in messages"
          :key="msg.id"
          class="message"
          :class="msg.role"
        >
          <div class="message-header">
            <span class="role-badge" :class="msg.role">{{ msg.role }}</span>
            <span class="msg-time">{{ formatTime(msg.timestamp) }}</span>
          </div>
          <div class="message-content">
            <MarkdownRenderer v-if="msg.role === 'assistant' || msg.role === 'system'" :source="msg.content" />
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
      <div class="furnace-input-bar">
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
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import { Send, ChevronDown, ChevronUp, Plus, PanelLeft } from 'lucide-vue-next'
import { useForge } from '@/composables/useForge'
import MarkdownRenderer from '@/components/MarkdownRenderer.vue'

const {
  session,
  messages,
  isLoading,
  error,
  sessionList,
  sessionId,
  sessionStatus,
  resume,
  switchSession,
  clearSession,
  loadSessionList,
  sendMessage: forgeSendMessage,
} = useForge()

const inputText = ref('')
const chatRef = ref<HTMLDivElement>()
const sidebarCollapsed = ref(false)

const hasPendingAssistant = computed(() => {
  return messages.value.some((m) => m.role === 'assistant' && m.content === '' && !m.tool_calls?.length)
})

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
  width: 240px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: var(--af-card);
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
  padding: 0.6rem 0.75rem;
  border-bottom: 1px solid var(--af-border);
  flex-shrink: 0;
}

.sidebar-title {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--af-fg);
  flex: 1;
}

.sidebar-new-btn,
.sidebar-collapse-btn,
.sidebar-toggle-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: 1px solid var(--af-border);
  border-radius: 6px;
  color: var(--af-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.sidebar-new-btn:hover,
.sidebar-collapse-btn:hover,
.sidebar-toggle-btn:hover {
  background: var(--af-secondary);
  color: var(--af-fg);
}

.session-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.session-item {
  padding: 0.5rem 0.6rem;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
  border: 1px solid transparent;
}

.session-item:hover {
  background: var(--af-secondary);
}

.session-item.active {
  background: var(--af-primary-soft);
  border-color: hsl(var(--primary) / 0.2);
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
  margin-top: 0.25rem;
}

.session-count {
  font-size: 0.65rem;
  color: var(--af-muted);
}

.session-status {
  font-size: 0.6rem;
  font-weight: 700;
  text-transform: uppercase;
  padding: 0.05rem 0.3rem;
  border-radius: 4px;
}

.session-status.idle {
  background: hsl(var(--af-success) / 0.15);
  color: hsl(var(--af-success));
}

.session-status.thinking {
  background: hsl(var(--af-warning) / 0.15);
  color: hsl(var(--af-warning));
}

.session-status.tool_call {
  background: hsl(var(--af-info) / 0.15);
  color: hsl(var(--af-info));
}

.session-status.error {
  background: hsl(var(--af-error) / 0.15);
  color: hsl(var(--af-error));
}

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
  background: var(--af-card);
  border-bottom: 1px solid var(--af-border);
  flex-shrink: 0;
}

.furnace-header h2 {
  font-size: 1rem;
  font-weight: 600;
  color: hsl(var(--af-furnace));
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.session-badge {
  font-size: 0.7rem;
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  text-transform: uppercase;
  font-weight: 600;
}

.session-badge.idle {
  background: hsl(var(--af-success) / 0.15);
  color: hsl(var(--af-success));
}

.session-badge.thinking {
  background: hsl(var(--af-warning) / 0.15);
  color: hsl(var(--af-warning));
}

.session-badge.tool_call {
  background: hsl(var(--af-info) / 0.15);
  color: hsl(var(--af-info));
}

.session-badge.waiting_approval {
  background: var(--af-primary-soft);
  color: var(--af-primary);
}

.session-badge.error {
  background: hsl(var(--af-error) / 0.15);
  color: hsl(var(--af-error));
}

.chat-canvas {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.message {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  max-width: 80%;
}

.message.user {
  align-self: flex-end;
}

.message.assistant,
.message.system {
  align-self: flex-start;
}

.message.error {
  align-self: center;
  max-width: 100%;
}

.message-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.role-badge {
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
}

.role-badge.user {
  background: var(--af-primary-soft);
  color: var(--af-primary);
}

.role-badge.assistant {
  background: var(--af-primary-soft);
  color: var(--af-primary);
}

.role-badge.system {
  background: var(--af-secondary);
  color: var(--af-muted);
}

.msg-time {
  font-size: 0.65rem;
  color: var(--af-muted);
}

.message-content {
  background: var(--af-card);
  border: 1px solid var(--af-border);
  border-radius: 8px;
  padding: 0.75rem 1rem;
  font-size: 0.9rem;
  line-height: 1.5;
  color: var(--af-fg);
  white-space: pre-wrap;
  word-break: break-word;
}

.message-content.error {
  background: hsl(var(--af-error) / 0.1);
  border-color: hsl(var(--af-error) / 0.3);
  color: hsl(var(--af-error));
}

.message.user .message-content {
  background: var(--af-primary-soft);
  border-color: hsl(var(--primary) / 0.2);
}

.message.system .message-content {
  background: var(--af-secondary);
  border-color: var(--af-border);
  font-style: italic;
  color: var(--af-muted);
}

.tool-calls {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-top: 0.25rem;
}

.tool-card {
  background: var(--af-card);
  border: 1px solid var(--af-border);
  border-radius: 6px;
  overflow: hidden;
}

.tool-card.pending {
  border-color: hsl(var(--af-warning) / 0.4);
}

.tool-card.running {
  border-color: hsl(var(--af-info) / 0.4);
}

.tool-card.success {
  border-color: hsl(var(--af-success) / 0.4);
}

.tool-card.error {
  border-color: hsl(var(--af-error) / 0.4);
}

.tool-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  cursor: pointer;
  user-select: none;
}

.tool-header:hover {
  background: var(--af-secondary);
}

.tool-icon {
  font-size: 0.9rem;
}

.tool-name {
  font-size: 0.75rem;
  font-weight: 600;
  color: hsl(var(--af-warning));
  flex: 1;
}

.tool-status {
  font-size: 0.6rem;
  font-weight: 700;
  text-transform: uppercase;
  padding: 0.1rem 0.3rem;
  border-radius: 4px;
}

.tool-status.pending {
  background: hsl(var(--af-warning) / 0.15);
  color: hsl(var(--af-warning));
}

.tool-status.running {
  background: hsl(var(--af-info) / 0.15);
  color: hsl(var(--af-info));
}

.tool-status.success {
  background: hsl(var(--af-success) / 0.15);
  color: hsl(var(--af-success));
}

.tool-status.error {
  background: hsl(var(--af-error) / 0.15);
  color: hsl(var(--af-error));
}

.tool-chevron {
  color: var(--af-muted);
}

.tool-body {
  border-top: 1px solid var(--af-border);
  padding: 0.5rem 0.75rem;
}

.tool-section {
  margin-bottom: 0.5rem;
}

.tool-section:last-child {
  margin-bottom: 0;
}

.tool-section-title {
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--af-muted);
  margin-bottom: 0.25rem;
}

.tool-code {
  font-size: 0.75rem;
  color: var(--af-muted);
  background: var(--af-bg);
  padding: 0.4rem;
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
}

.typing-dots {
  animation: blink 1.4s infinite both;
}

@keyframes blink {
  0%, 80%, 100% { opacity: 0; }
  40% { opacity: 1; }
}

.furnace-input-bar {
  display: flex;
  align-items: flex-end;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: var(--af-card);
  border-top: 1px solid var(--af-border);
  flex-shrink: 0;
}

.furnace-input {
  flex: 1;
  background: var(--af-bg);
  border: 1px solid var(--af-border);
  border-radius: 8px;
  padding: 0.6rem 0.75rem;
  color: var(--af-fg);
  font-size: 0.9rem;
  resize: none;
  min-height: 40px;
  max-height: 120px;
  outline: none;
  font-family: inherit;
}

.furnace-input:focus {
  border-color: hsl(var(--af-furnace));
}

.furnace-input:disabled {
  opacity: 0.5;
}

.send-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: linear-gradient(135deg, var(--vp-c-brand-1) 0%, var(--vp-c-brand-2) 100%);
  border: none;
  border-radius: 8px;
  color: #fff;
  cursor: pointer;
  transition: opacity 0.15s;
}

.send-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.send-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
