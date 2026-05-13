<template>
  <div class="markdown-editor">
    <div class="editor-split">
      <div class="editor-pane">
        <div class="pane-label">Markdown</div>
        <textarea
          v-model="draftContent"
          class="editor-textarea"
          spellcheck="false"
        />
      </div>
      <div class="preview-pane">
        <div class="pane-label">Preview</div>
        <MarkdownContent :content="draftContent" @link-click="$emit('linkClick', $event)" />
      </div>
    </div>
    <div class="editor-actions">
      <button class="save-btn" @click="$emit('save', draftContent)">
        <Check :size="13" />
        Save
      </button>
      <button class="cancel-btn" @click="$emit('cancel')">
        <X :size="13" />
        Cancel
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import MarkdownContent from '@/components/MarkdownContent.vue'
import { Check, X } from 'lucide-vue-next'

const props = defineProps<{
  content: string
}>()

const emit = defineEmits<{
  save: [content: string]
  cancel: []
  linkClick: [id: string]
}>()

const draftContent = ref(props.content)
</script>

<style scoped>
.markdown-editor {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.editor-split {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  min-height: 320px;
}

.editor-pane,
.preview-pane {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--af-border);
  border-radius: 8px;
  overflow: hidden;
  background: var(--af-card);
}

.pane-label {
  padding: 0.35rem 0.6rem;
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--af-muted);
  background: hsl(var(--muted-foreground) / 0.04);
  border-bottom: 1px solid var(--af-border);
}

.editor-textarea {
  flex: 1;
  padding: 0.6rem 0.75rem;
  border: none;
  background: transparent;
  color: var(--af-fg);
  font-size: 0.82rem;
  line-height: 1.6;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  resize: none;
  outline: none;
}

.preview-pane :deep(.markdown-content) {
  flex: 1;
  padding: 0.6rem 0.75rem;
  font-size: 0.82rem;
  line-height: 1.6;
  overflow-y: auto;
}

.editor-actions {
  display: flex;
  gap: 0.5rem;
}

.save-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.4rem 0.8rem;
  font-size: 0.78rem;
  font-weight: 600;
  border-radius: 6px;
  border: none;
  background: hsl(var(--primary));
  color: white;
  cursor: pointer;
}

.save-btn:hover {
  opacity: 0.9;
}

.cancel-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.4rem 0.8rem;
  font-size: 0.78rem;
  border-radius: 6px;
  border: 1px solid var(--af-border);
  background: transparent;
  color: var(--af-muted);
  cursor: pointer;
}

.cancel-btn:hover {
  color: var(--af-fg);
  background: hsl(var(--muted-foreground) / 0.05);
}
</style>
