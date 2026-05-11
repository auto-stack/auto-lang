<template>
  <div class="forge-view">
    <div class="forge-header">
      <h2>The Forge</h2>
      <span class="session-badge" :class="sessionStatus">
        {{ sessionStatus }}
      </span>
    </div>
    <div class="forge-body">
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
            <pre v-if="msg.role === 'tool'">{{ msg.content }}</pre>
            <div v-else>{{ msg.content }}</div>
          </div>
          <div v-if="msg.tool_calls" class="tool-calls">
            <div
              v-for="tc in msg.tool_calls"
              :key="tc.id"
              class="tool-card"
              :class="tc.status"
            >
              <div class="tool-name">🔧 {{ tc.name }}</div>
              <pre class="tool-args">{{ JSON.stringify(tc.arguments, null, 2) }}</pre>
              <div v-if="tc.result" class="tool-result">{{ tc.result }}</div>
            </div>
          </div>
        </div>
        <div v-if="isLoading" class="message assistant pending">
          <div class="message-header">
            <span class="role-badge assistant">assistant</span>
          </div>
          <div class="message-content">
            <span class="typing">Thinking</span>
            <span class="typing-dots">...</span>
          </div>
        </div>
      </div>
      <div class="forge-input-bar">
        <textarea
          v-model="inputText"
          class="forge-input"
          placeholder="Describe what you want to build... (Shift+Enter to send)"
          @keydown.shift.enter.prevent="sendMessage"
        />
        <button class="send-btn" :disabled="!inputText.trim() || isLoading" @click="sendMessage">
          <Send :size="16" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { Send } from 'lucide-vue-next'
import type { ForgeMessage } from '@/types/forge'

const messages = ref<ForgeMessage[]>([
  {
    id: 'welcome',
    role: 'system',
    content: 'Welcome to AutoSmith. I\'m your serial agent forge. Describe what you want to build, and I\'ll break it down into specs, generate code, and test it — one step at a time.',
    timestamp: Date.now(),
  },
])

const inputText = ref('')
const isLoading = ref(false)
const sessionStatus = ref<'idle' | 'thinking' | 'waiting'>('idle')
const chatRef = ref<HTMLDivElement>()

function formatTime(ts: number): string {
  return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

async function scrollToBottom() {
  await nextTick()
  if (chatRef.value) {
    chatRef.value.scrollTop = chatRef.value.scrollHeight
  }
}

async function sendMessage() {
  const text = inputText.value.trim()
  if (!text || isLoading.value) return

  const userMsg: ForgeMessage = {
    id: `u-${Date.now()}`,
    role: 'user',
    content: text,
    timestamp: Date.now(),
  }
  messages.value.push(userMsg)
  inputText.value = ''
  isLoading.value = true
  sessionStatus.value = 'thinking'
  await scrollToBottom()

  // TODO: wire to backend SSE endpoint
  setTimeout(() => {
    const assistantMsg: ForgeMessage = {
      id: `a-${Date.now()}`,
      role: 'assistant',
      content: 'I understand you want to build something. In the full implementation, I would analyze your request, update the Ledger with specs, and start generating code. For now, this is a scaffold.',
      timestamp: Date.now(),
    }
    messages.value.push(assistantMsg)
    isLoading.value = false
    sessionStatus.value = 'idle'
    scrollToBottom()
  }, 1500)
}
</script>

<style scoped>
.forge-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.forge-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: #181825;
  border-bottom: 1px solid #313244;
  flex-shrink: 0;
}

.forge-header h2 {
  font-size: 1rem;
  font-weight: 600;
  color: #fab387;
}

.session-badge {
  font-size: 0.7rem;
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  text-transform: uppercase;
  font-weight: 600;
}

.session-badge.idle {
  background: #27c93f22;
  color: #27c93f;
}

.session-badge.thinking {
  background: #f9e2af22;
  color: #f9e2af;
}

.session-badge.waiting {
  background: #cba6f722;
  color: #cba6f7;
}

.forge-body {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
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
  background: #6366f122;
  color: #6366f1;
}

.role-badge.assistant {
  background: #cba6f722;
  color: #cba6f7;
}

.role-badge.system {
  background: #45475a;
  color: #a6adc8;
}

.role-badge.tool {
  background: #f9e2af22;
  color: #f9e2af;
}

.msg-time {
  font-size: 0.65rem;
  color: #45475a;
}

.message-content {
  background: #1e1e2e;
  border: 1px solid #313244;
  border-radius: 8px;
  padding: 0.75rem 1rem;
  font-size: 0.9rem;
  line-height: 1.5;
  color: #cdd6f4;
  white-space: pre-wrap;
  word-break: break-word;
}

.message.user .message-content {
  background: #6366f122;
  border-color: #6366f133;
}

.message.system .message-content {
  background: #181825;
  border-color: #313244;
  font-style: italic;
  color: #a6adc8;
}

.tool-calls {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-top: 0.25rem;
}

.tool-card {
  background: #181825;
  border: 1px solid #313244;
  border-radius: 6px;
  padding: 0.5rem 0.75rem;
}

.tool-card.pending {
  border-color: #f9e2af44;
}

.tool-card.success {
  border-color: #27c93f44;
}

.tool-card.error {
  border-color: #f38ba844;
}

.tool-name {
  font-size: 0.75rem;
  font-weight: 600;
  color: #f9e2af;
  margin-bottom: 0.25rem;
}

.tool-args {
  font-size: 0.75rem;
  color: #6c7086;
  background: #0f0f14;
  padding: 0.35rem;
  border-radius: 4px;
  overflow-x: auto;
}

.tool-result {
  font-size: 0.8rem;
  color: #a6e3a1;
  margin-top: 0.35rem;
  padding-top: 0.35rem;
  border-top: 1px solid #313244;
}

.typing {
  color: #a6adc8;
}

.typing-dots {
  animation: blink 1.4s infinite both;
}

@keyframes blink {
  0%, 80%, 100% { opacity: 0; }
  40% { opacity: 1; }
}

.forge-input-bar {
  display: flex;
  align-items: flex-end;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: #181825;
  border-top: 1px solid #313244;
  flex-shrink: 0;
}

.forge-input {
  flex: 1;
  background: #1e1e2e;
  border: 1px solid #313244;
  border-radius: 8px;
  padding: 0.6rem 0.75rem;
  color: #cdd6f4;
  font-size: 0.9rem;
  resize: none;
  min-height: 40px;
  max-height: 120px;
  outline: none;
  font-family: inherit;
}

.forge-input:focus {
  border-color: #fab387;
}

.send-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: #fab387;
  border: none;
  border-radius: 8px;
  color: #0f0f14;
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
