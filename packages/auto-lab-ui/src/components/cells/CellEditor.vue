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
        :error-lines="errorLines"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { CodeEditor } from 'auto-playground-vue'
import type { Cell } from '@/types/cell'

const props = defineProps<{
  cell: Cell
}>()

const emit = defineEmits<{
  (e: 'update', source: string): void
}>()

const errorLines = computed(() => {
  if (!props.cell.output?.diagnostics) return []
  return props.cell.output.diagnostics
    .map((d) => d.line)
    .filter((l): l is number => l !== undefined && l > 0)
})
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
