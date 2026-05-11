<template>
  <div class="furnace-view">
    <div class="furnace-header">
      <h2>The Furnace · 丹炉</h2>
      <span class="session-badge" :class="sessionStatus">
        {{ sessionStatus }}
      </span>
    </div>
    <div class="furnace-body">
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
import { Send, ChevronDown, ChevronUp } from 'lucide-vue-next'
import { useForge } from '@/composables/useForge'
import MarkdownRenderer from '@/components/MarkdownRenderer.vue'

const {
  messages,
  isLoading,
  error,
  sessionStatus,
  createSession,
  sendMessage: forgeSendMessage,
} = useForge()

const inputText = ref('')
const chatRef = ref<HTMLDivElement>()

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
  await createSession()
})
</script>

<style scoped>
.furnace-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.furnace-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: #181825;
  border-bottom: 1px solid #313244;
  flex-shrink: 0;
}

.furnace-header h2 {
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

.session-badge.tool_call {
  background: #89b4fa22;
  color: #89b4fa;
}

.session-badge.waiting_approval {
  background: #cba6f722;
  color: #cba6f7;
}

.session-badge.error {
  background: #f38ba822;
  color: #f38ba8;
}

.furnace-body {
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

.message-content.error {
  background: #f38ba822;
  border-color: #f38ba844;
  color: #f38ba8;
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
  overflow: hidden;
}

.tool-card.pending {
  border-color: #f9e2af44;
}

.tool-card.running {
  border-color: #89b4fa44;
}

.tool-card.success {
  border-color: #a6e3a144;
}

.tool-card.error {
  border-color: #f38ba844;
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
  background: #1e1e2e;
}

.tool-icon {
  font-size: 0.9rem;
}

.tool-name {
  font-size: 0.75rem;
  font-weight: 600;
  color: #f9e2af;
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
  background: #f9e2af22;
  color: #f9e2af;
}

.tool-status.running {
  background: #89b4fa22;
  color: #89b4fa;
}

.tool-status.success {
  background: #a6e3a122;
  color: #a6e3a1;
}

.tool-status.error {
  background: #f38ba822;
  color: #f38ba8;
}

.tool-chevron {
  color: #6c7086;
}

.tool-body {
  border-top: 1px solid #313244;
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
  color: #6c7086;
  margin-bottom: 0.25rem;
}

.tool-code {
  font-size: 0.75rem;
  color: #a6adc8;
  background: #0f0f14;
  padding: 0.4rem;
  border-radius: 4px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-word;
}

.tool-code.result {
  color: #a6e3a1;
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

.furnace-input-bar {
  display: flex;
  align-items: flex-end;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: #181825;
  border-top: 1px solid #313244;
  flex-shrink: 0;
}

.furnace-input {
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

.furnace-input:focus {
  border-color: #fab387;
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
