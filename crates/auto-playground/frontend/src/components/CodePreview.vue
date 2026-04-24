<template>
  <div class="code-preview">
    <div class="toolbar">
      <button class="copy-btn" @click="copyCode" :title="'Copy'">
        {{ copied ? 'Copied!' : 'Copy' }}
      </button>
    </div>
    <div class="lines-container">
      <div
        v-for="(line, index) in lines"
        :key="index"
        :class="['code-line', { highlighted: isHighlighted(index + 1) }]"
      >
        <span class="line-number">{{ index + 1 }}</span>
        <span class="line-content">{{ line }}</span>
      </div>
      <div v-if="lines.length === 0" class="code-line">
        <span class="line-number">1</span>
        <span class="line-content"></span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

const props = defineProps<{
  code: string;
  language?: string;
  highlightLines?: number[];
}>();

const copied = ref(false);

const lines = computed(() => {
  if (!props.code) return [''];
  return props.code.split('\n');
});

function isHighlighted(lineNum: number): boolean {
  return props.highlightLines?.includes(lineNum) ?? false;
}

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
.lines-container {
  padding: 0;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: #d4d4d4;
}
.code-line {
  display: flex;
  padding: 0 12px;
  min-height: 20px;
  line-height: 20px;
}
.code-line.highlighted {
  background: rgba(255, 255, 0, 0.12);
}
.line-number {
  flex-shrink: 0;
  width: 48px;
  text-align: right;
  padding-right: 16px;
  color: #6e7681;
  user-select: none;
}
.line-content {
  white-space: pre;
}
</style>
