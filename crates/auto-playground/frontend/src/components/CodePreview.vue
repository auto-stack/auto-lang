<template>
  <div class="code-preview">
    <div class="toolbar">
      <button class="copy-btn" @click="copyCode" :title="'Copy'">
        {{ copied ? 'Copied!' : 'Copy' }}
      </button>
    </div>
    <pre><code>{{ code }}</code></pre>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const props = defineProps<{
  code: string;
  language?: string;
}>();

const copied = ref(false);

async function copyCode() {
  try {
    await navigator.clipboard.writeText(props.code);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  } catch { /* ignore */ }
}
</script>

<style scoped>
.code-preview {
  position: relative;
  height: 100%;
  overflow: auto;
  background: #1e1e1e;
}
.toolbar {
  position: sticky;
  top: 0;
  display: flex;
  justify-content: flex-end;
  padding: 4px 8px;
  background: #2d2d2d;
  z-index: 1;
}
.copy-btn {
  background: #3c3c3c;
  color: #ccc;
  border: 1px solid #555;
  border-radius: 4px;
  padding: 2px 10px;
  cursor: pointer;
  font-size: 12px;
}
.copy-btn:hover {
  background: #4c4c4c;
}
pre {
  margin: 0;
  padding: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: #d4d4d4;
  white-space: pre-wrap;
}
</style>
