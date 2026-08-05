<script setup lang="ts">
import { ref } from 'vue'

// Plan 337 Task 4.1: per-widget install command with copy button.
const props = defineProps<{
  widget: string
}>()

const copied = ref(false)
const cmd = `npx @auto-ui/widgets add ${props.widget}`

async function copy() {
  try {
    await navigator.clipboard.writeText(cmd)
    copied.value = true
    setTimeout(() => (copied.value = false), 1500)
  } catch {
    /* clipboard unavailable */
  }
}
</script>

<template>
  <div class="install-hint">
    <code class="install-cmd">{{ cmd }}</code>
    <button type="button" class="install-copy" @click="copy">
      {{ copied ? '✓ Copied' : 'Copy' }}
    </button>
  </div>
</template>

<style scoped>
.install-hint {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 0.5rem 0 1.5rem;
  padding: 0.5rem 0.75rem;
  background: #1a1a2e;
  border: 1px solid #333;
  border-radius: 6px;
  font-size: 0.85rem;
}
.install-cmd {
  color: #7dd3fc;
  font-family: monospace;
}
.install-copy {
  margin-left: auto;
  padding: 0.2rem 0.6rem;
  background: #333;
  color: #ccc;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8rem;
}
.install-copy:hover {
  background: #444;
}
</style>
