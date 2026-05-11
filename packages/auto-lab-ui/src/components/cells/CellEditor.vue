<template>
  <div class="cell-editor">
    <div v-if="cell.type === 'markdown'" class="markdown-editor">
      <textarea
        :value="cell.source"
        @input="$emit('update', ($event.target as HTMLTextAreaElement).value)"
        placeholder="Write markdown here..."
        rows="4"
      />
    </div>
    <div v-else-if="cell.type === 'ai'" class="ai-editor">
      <textarea
        :value="cell.source"
        @input="$emit('update', ($event.target as HTMLTextAreaElement).value)"
        placeholder="AI conversation..."
        rows="2"
      />
    </div>
    <div v-else class="code-editor-wrapper">
      <CodeEditor
        :model-value="cell.source"
        @update:model-value="$emit('update', $event)"
        :on-run="() => {}"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { CodeEditor } from 'auto-playground-vue'
import type { Cell } from '@/types/cell'

defineProps<{
  cell: Cell
}>()

defineEmits<{
  (e: 'update', source: string): void
}>()
</script>

<style scoped>
.cell-editor {
  min-height: 0;
}

.markdown-editor textarea,
.ai-editor textarea {
  width: 100%;
  background: #1e1e2e;
  color: #cdd6f4;
  border: none;
  padding: 0.75rem;
  font-family: inherit;
  font-size: 0.9rem;
  resize: vertical;
  outline: none;
  line-height: 1.5;
}

.markdown-editor textarea::placeholder,
.ai-editor textarea::placeholder {
  color: #6c7086;
}

.code-editor-wrapper {
  min-height: 120px;
}

.code-editor-wrapper :deep(.cm-editor) {
  background: #1e1e2e;
}
</style>
