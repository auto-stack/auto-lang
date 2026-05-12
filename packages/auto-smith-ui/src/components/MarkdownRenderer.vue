<template>
  <MarkdownRender
    :content="source"
    :final="!streaming"
    :max-live-nodes="streaming ? 0 : 320"
    :batch-rendering="streaming"
    :render-batch-size="16"
    :render-batch-delay="8"
    :typewriter="streaming"
    :fade="false"
  />
</template>

<script setup lang="ts">
import { MarkdownRender } from 'markstream-vue'

const props = defineProps<{
  source: string
  streaming?: boolean
}>()
</script>

<style>
/* Override markstream-vue container font to match AutoForge theme */
.markstream-vue {
  font-size: 0.9rem;
  line-height: 1.6;
  color: var(--af-fg);
}

/* Heading colours */
.markstream-vue h1,
.markstream-vue h2,
.markstream-vue h3,
.markstream-vue h4 {
  color: hsl(var(--af-furnace));
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}

.markstream-vue h1 { font-size: 1.2rem; }
.markstream-vue h2 { font-size: 1.1rem; }
.markstream-vue h3 { font-size: 1rem; }

/* Paragraphs */
.markstream-vue p {
  margin-bottom: 0.75rem;
}

/* Code blocks */
.markstream-vue pre {
  background: var(--af-bg);
  border: 1px solid var(--af-border);
  border-radius: 6px;
  padding: 0.75rem;
  overflow-x: auto;
  margin: 0.5rem 0;
}

.markstream-vue code {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.8rem;
}

.markstream-vue pre code {
  color: var(--af-fg);
  background: transparent;
  padding: 0;
}

/* Inline code */
.markstream-vue p code,
.markstream-vue li code {
  background: var(--af-secondary);
  padding: 0.1rem 0.3rem;
  border-radius: 4px;
  color: hsl(var(--af-warning));
}

/* Lists */
.markstream-vue ul,
.markstream-vue ol {
  margin-left: 1.25rem;
  margin-bottom: 0.75rem;
}

.markstream-vue li {
  margin-bottom: 0.25rem;
}

/* Blockquotes */
.markstream-vue blockquote {
  border-left: 3px solid var(--af-border);
  padding-left: 0.75rem;
  margin-left: 0;
  color: var(--af-muted);
  font-style: italic;
}

/* Horizontal rules */
.markstream-vue hr {
  border: none;
  border-top: 1px solid var(--af-border);
  margin: 1rem 0;
}

/* Links */
.markstream-vue a {
  color: hsl(var(--af-info));
  text-decoration: none;
}

.markstream-vue a:hover {
  text-decoration: underline;
}

/* Tables */
.markstream-vue table {
  border-collapse: collapse;
  width: 100%;
  margin: 0.5rem 0;
}

.markstream-vue th,
.markstream-vue td {
  border: 1px solid var(--af-border);
  padding: 0.4rem 0.6rem;
  text-align: left;
}

.markstream-vue th {
  background: var(--af-card);
  font-weight: 600;
}

/* Syntax highlighting colours (mapped to theme palette) */
.markstream-vue .hljs-keyword,
.markstream-vue .hljs-selector-tag,
.markstream-vue .hljs-literal,
.markstream-vue .hljs-section,
.markstream-vue .hljs-link {
  color: var(--af-primary);
}

.markstream-vue .hljs-string,
.markstream-vue .hljs-title,
.markstream-vue .hljs-name,
.markstream-vue .hljs-type,
.markstream-vue .hljs-attribute,
.markstream-vue .hljs-symbol,
.markstream-vue .hljs-bullet,
.markstream-vue .hljs-addition,
.markstream-vue .hljs-variable,
.markstream-vue .hljs-template-tag,
.markstream-vue .hljs-template-variable {
  color: hsl(var(--af-success));
}

.markstream-vue .hljs-comment,
.markstream-vue .hljs-quote,
.markstream-vue .hljs-deletion,
.markstream-vue .hljs-meta {
  color: var(--af-muted);
}

.markstream-vue .hljs-number,
.markstream-vue .hljs-regexp,
.markstream-vue .hljs-literal,
.markstream-vue .hljs-built_in,
.markstream-vue .hljs-builtin-name {
  color: hsl(var(--af-furnace));
}

.markstream-vue .hljs-function,
.markstream-vue .hljs-params {
  color: hsl(var(--af-info));
}

.markstream-vue .hljs-tag,
.markstream-vue .hljs-name {
  color: hsl(var(--af-error));
}
</style>
