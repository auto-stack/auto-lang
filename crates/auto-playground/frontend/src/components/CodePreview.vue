<template>
  <div class="code-preview">
    <div class="lines-container">
      <div
        v-for="(line, index) in highlightedLines"
        :key="index"
        :class="['code-line', { highlighted: isHighlighted(index + 1) }]"
      >
        <span class="line-number">{{ index + 1 }}</span>
        <span class="line-content" v-html="line || ' '"></span>
      </div>
      <div v-if="highlightedLines.length === 0" class="code-line">
        <span class="line-number">1</span>
        <span class="line-content"></span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import hljs from 'highlight.js/lib/core';
import rust from 'highlight.js/lib/languages/rust';
import python from 'highlight.js/lib/languages/python';
import typescript from 'highlight.js/lib/languages/typescript';
import c from 'highlight.js/lib/languages/c';
import 'highlight.js/styles/atom-one-dark.css';

hljs.registerLanguage('rust', rust);
hljs.registerLanguage('python', python);
hljs.registerLanguage('typescript', typescript);
hljs.registerLanguage('c', c);

const props = defineProps<{
  code: string;
  language?: string;
  highlightLines?: number[];
}>();

const hljsLanguageMap: Record<string, string> = {
  rust: 'rust',
  python: 'python',
  typescript: 'typescript',
  c: 'c',
};

const highlightedLines = computed(() => {
  if (!props.code) return [''];
  const lang = props.language ? hljsLanguageMap[props.language] : undefined;
  if (!lang) {
    return props.code.split('\n');
  }
  try {
    const result = hljs.highlight(props.code, { language: lang });
    // Split highlighted HTML by newlines while preserving per-line markup
    return result.value.split('\n');
  } catch {
    return props.code.split('\n');
  }
});

function isHighlighted(lineNum: number): boolean {
  return props.highlightLines?.includes(lineNum) ?? false;
}
</script>

<style scoped>
.code-preview {
  position: relative;
  height: 100%;
  overflow: auto;
  background: #1e1e1e;
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
.line-content :deep(.hljs) {
  background: transparent;
  padding: 0;
}
</style>
