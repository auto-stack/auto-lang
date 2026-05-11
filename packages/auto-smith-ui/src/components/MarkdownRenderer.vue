<template>
  <div class="markdown-body" v-html="rendered"></div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { marked } from 'marked'

const props = defineProps<{
  source: string
}>()

marked.setOptions({
  gfm: true,
  breaks: true,
})

const rendered = computed(() => {
  return marked.parse(props.source || '')
})
</script>

<style>
.markdown-body {
  font-size: 0.9rem;
  line-height: 1.6;
  color: #cdd6f4;
}

.markdown-body h1,
.markdown-body h2,
.markdown-body h3,
.markdown-body h4 {
  color: #fab387;
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}

.markdown-body h1 { font-size: 1.2rem; }
.markdown-body h2 { font-size: 1.1rem; }
.markdown-body h3 { font-size: 1rem; }

.markdown-body p {
  margin-bottom: 0.75rem;
}

.markdown-body pre {
  background: #11111b;
  border: 1px solid #313244;
  border-radius: 6px;
  padding: 0.75rem;
  overflow-x: auto;
  margin: 0.5rem 0;
}

.markdown-body code {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.8rem;
}

.markdown-body pre code {
  color: #cdd6f4;
  background: transparent;
  padding: 0;
}

.markdown-body p code,
.markdown-body li code {
  background: #313244;
  padding: 0.1rem 0.3rem;
  border-radius: 4px;
  color: #f9e2af;
}

.markdown-body ul,
.markdown-body ol {
  margin-left: 1.25rem;
  margin-bottom: 0.75rem;
}

.markdown-body li {
  margin-bottom: 0.25rem;
}

.markdown-body blockquote {
  border-left: 3px solid #45475a;
  padding-left: 0.75rem;
  margin-left: 0;
  color: #a6adc8;
  font-style: italic;
}

.markdown-body hr {
  border: none;
  border-top: 1px solid #313244;
  margin: 1rem 0;
}

.markdown-body a {
  color: #89b4fa;
  text-decoration: none;
}

.markdown-body a:hover {
  text-decoration: underline;
}

.markdown-body table {
  border-collapse: collapse;
  width: 100%;
  margin: 0.5rem 0;
}

.markdown-body th,
.markdown-body td {
  border: 1px solid #313244;
  padding: 0.4rem 0.6rem;
  text-align: left;
}

.markdown-body th {
  background: #181825;
  font-weight: 600;
}

/* Syntax highlighting colors (simple Catppuccin Mocha-like) */
.markdown-body .hljs-keyword,
.markdown-body .hljs-selector-tag,
.markdown-body .hljs-literal,
.markdown-body .hljs-section,
.markdown-body .hljs-link {
  color: #cba6f7;
}

.markdown-body .hljs-string,
.markdown-body .hljs-title,
.markdown-body .hljs-name,
.markdown-body .hljs-type,
.markdown-body .hljs-attribute,
.markdown-body .hljs-symbol,
.markdown-body .hljs-bullet,
.markdown-body .hljs-addition,
.markdown-body .hljs-variable,
.markdown-body .hljs-template-tag,
.markdown-body .hljs-template-variable {
  color: #a6e3a1;
}

.markdown-body .hljs-comment,
.markdown-body .hljs-quote,
.markdown-body .hljs-deletion,
.markdown-body .hljs-meta {
  color: #6c7086;
}

.markdown-body .hljs-number,
.markdown-body .hljs-regexp,
.markdown-body .hljs-literal,
.markdown-body .hljs-built_in,
.markdown-body .hljs-builtin-name {
  color: #fab387;
}

.markdown-body .hljs-function,
.markdown-body .hljs-params {
  color: #89b4fa;
}

.markdown-body .hljs-tag,
.markdown-body .hljs-name {
  color: #f38ba8;
}
</style>
