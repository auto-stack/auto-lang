<template>
  <div class="spec-item-detail">
    <!-- Relations -->
    <RelationsPanel
      :item="item"
      :project="project"
      @jump="$emit('jump', $event)"
    />

    <!-- Content -->
    <div class="detail-content">
      <slot name="content" :item="item">
        <MarkdownContent
          :content="item.content || '(No content)'"
          @link-click="$emit('jump', $event)"
        />
      </slot>
    </div>

    <!-- Metadata chips -->
    <div v-if="hasMeta" class="detail-meta">
      <span v-if="item.priority" class="meta-chip priority">
        Priority: {{ item.priority }}
      </span>
      <span v-if="item.assignee" class="meta-chip assignee">
        👤 {{ item.assignee }}
      </span>
      <span v-if="item.test_file" class="meta-chip file">
        📄 {{ item.test_file }}
      </span>
      <span v-if="item.module" class="meta-chip module">
        📦 {{ item.module }}
      </span>
      <span v-if="item.milestone" class="meta-chip milestone">
        🚩 {{ item.milestone }}
      </span>
      <span v-if="item.depends_on?.length" class="meta-chip deps">
        Depends: {{ item.depends_on.join(', ') }}
      </span>
    </div>

    <!-- Actions -->
    <div class="detail-actions">
      <StatusTransition
        :status="item.status"
        :section-type="sectionType"
        @change="$emit('status-change', { id: item.id, status: $event })"
      />
      <button class="action-btn" @click="$emit('edit', item)">
        <Pencil :size="13" />
        Edit
      </button>
      <button class="action-btn danger" @click="$emit('delete', item.id)">
        <Trash2 :size="13" />
        Delete
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { SpecItem, SectionType } from '@/types/specs'
import RelationsPanel from './RelationsPanel.vue'
import MarkdownContent from './MarkdownContent.vue'
import StatusTransition from './StatusTransition.vue'
import { Pencil, Trash2 } from 'lucide-vue-next'

const props = defineProps<{
  item: SpecItem
  sectionType: SectionType
  project: string
}>()

defineEmits<{
  jump: [id: string]
  edit: [item: SpecItem]
  'status-change': [payload: { id: string; status: string }]
  delete: [id: string]
}>()

const hasMeta = computed(() =>
  props.item.priority ||
  props.item.assignee ||
  props.item.test_file ||
  props.item.module ||
  props.item.milestone ||
  (props.item.depends_on && props.item.depends_on.length > 0)
)
</script>

<style scoped>
.spec-item-detail {
  padding-top: 0.75rem;
}

.detail-content {
  margin-bottom: 0.75rem;
}

.detail-content :deep(.markdown-content) {
  font-size: 0.85rem;
  line-height: 1.65;
}

.detail-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  margin-bottom: 0.75rem;
}

.meta-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  padding: 0.2rem 0.5rem;
  font-size: 0.7rem;
  border-radius: 6px;
  background: hsl(var(--muted-foreground) / 0.06);
  color: var(--af-muted);
  border: 1px solid var(--af-border);
}

.meta-chip.priority {
  background: hsl(var(--primary) / 0.08);
  color: hsl(var(--primary));
  border-color: hsl(var(--primary) / 0.2);
}

.detail-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding-top: 0.5rem;
  border-top: 1px solid var(--af-border);
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.35rem 0.65rem;
  font-size: 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--af-border);
  background: transparent;
  color: var(--af-muted);
  cursor: pointer;
  transition: all 0.12s;
}

.action-btn:hover {
  background: hsl(var(--muted-foreground) / 0.06);
  color: var(--af-fg);
  border-color: hsl(var(--muted-foreground) / 0.2);
}

.action-btn.danger:hover {
  background: hsl(var(--destructive) / 0.08);
  color: hsl(var(--destructive));
  border-color: hsl(var(--destructive) / 0.3);
}
</style>
