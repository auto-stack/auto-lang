<template>
  <div class="ai-chat-bar">
    <div class="chat-input-wrapper">
      <Bot :size="18" class="chat-icon" />
      <input
        v-model="input"
        type="text"
        class="chat-input"
        placeholder="Ask AI anything... (Shift+Enter to submit)"
        @keydown.shift.enter.prevent="submit"
        @keydown.enter.exact.prevent
      />
      <button
        class="chat-submit"
        :disabled="!input.trim()"
        @click="submit"
      >
        <Send :size="16" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Bot, Send } from 'lucide-vue-next'

const emit = defineEmits<{
  (e: 'submit', content: string): void
}>()

const input = ref('')

function submit() {
  const text = input.value.trim()
  if (!text) return
  emit('submit', text)
  input.value = ''
}
</script>

<style scoped>
.ai-chat-bar {
  flex-shrink: 0;
  padding: 0.75rem 1rem;
  background: #181825;
  border-top: 1px solid #313244;
}

.chat-input-wrapper {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: #1e1e2e;
  border: 1px solid #313244;
  border-radius: 8px;
  padding: 0.5rem 0.75rem;
  transition: border-color 0.2s;
}

.chat-input-wrapper:focus-within {
  border-color: #6366f1;
}

.chat-icon {
  color: #cba6f7;
  flex-shrink: 0;
}

.chat-input {
  flex: 1;
  background: transparent;
  border: none;
  color: #cdd6f4;
  font-size: 0.9rem;
  outline: none;
  min-width: 0;
}

.chat-input::placeholder {
  color: #6c7086;
}

.chat-submit {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: #6366f1;
  color: #fff;
  border: none;
  border-radius: 6px;
  padding: 0.4rem;
  cursor: pointer;
  transition: all 0.15s;
  flex-shrink: 0;
}

.chat-submit:hover:not(:disabled) {
  background: #7c7ff0;
}

.chat-submit:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
