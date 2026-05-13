<template>
  <div class="tests-cards">
    <div
      v-for="item in items"
      :key="item.id"
      class="test-card"
      :class="{ expanded: expandedId === item.id, passing: isPassing(item), failing: isFailing(item) }"
      @click="$emit('toggle', item.id)"
    >
      <div class="test-header">
        <span class="test-indicator" :class="item.status">
          {{ isPassing(item) ? '✓' : isFailing(item) ? '✗' : '○' }}
        </span>
        <span class="test-id">{{ item.id }}</span>
        <span class="test-title">{{ item.title }}</span>
        <StatusBadge :status="item.status" size="sm" />
      </div>
      <div v-if="item.test_file" class="test-file">📄 {{ item.test_file }}</div>
      <div v-if="expandedId === item.id" class="item-detail">
        <RelationsPanel :item="item" :project="project" @jump="$emit('jump', $event)" />
        <div class="detail-content">{{ item.content }}</div>
        <div class="detail-actions">
          <button class="btn-sm" @click.stop="$emit('edit', item)">Edit</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SpecItem, Status } from '@/types/specs'
import StatusBadge from '@/components/StatusBadge.vue'
import RelationsPanel from '@/components/RelationsPanel.vue'

defineProps<{
  items: SpecItem[]
  project: string
  expandedId: string | null
}>()

defineEmits<{
  toggle: [id: string]
  jump: [id: string]
  edit: [item: SpecItem]
}>()

function isPassing(item: SpecItem): boolean {
  return item.status === 'done' || item.status === 'verified'
}
function isFailing(item: SpecItem): boolean {
  return item.status === 'blocked'
}
</script>

<style scoped>
.tests-cards {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
.test-card {
  padding: 0.75rem 1rem;
  border-radius: 10px;
  background: hsl(var(--muted-foreground) / 0.02);
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.12s;
}
.test-card:hover {
  background: hsl(var(--muted-foreground) / 0.05);
  border-color: var(--af-border);
}
.test-card.expanded {
  background: hsl(var(--muted-foreground) / 0.06);
  border-color: hsl(var(--primary) / 0.2);
}
.test-card.passing {
  border-left: 3px solid #10b981;
}
.test-card.failing {
  border-left: 3px solid #ef4444;
}
.test-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.test-indicator {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 0.75rem;
  font-weight: 700;
  flex-shrink: 0;
}
.test-indicator.done,
.test-indicator.verified {
  background: hsl(160 84% 39% / 0.15);
  color: #10b981;
}
.test-indicator.blocked {
  background: hsl(0 84% 60% / 0.15);
  color: #ef4444;
}
.test-indicator.draft,
.test-indicator.implemented {
  background: hsl(215 16% 62% / 0.15);
  color: #94a3b8;
}
.test-id {
  font-family: monospace;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--af-muted);
}
.test-title {
  font-size: 0.85rem;
  color: var(--af-fg);
  flex: 1;
  line-height: 1.3;
}
.test-file {
  font-size: 0.7rem;
  color: var(--af-muted);
  font-family: monospace;
  margin-top: 0.3rem;
  margin-left: 2.2rem;
}
.item-detail {
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--af-border);
}
.detail-content {
  font-size: 0.8rem;
  color: var(--af-fg);
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  background: hsl(var(--muted-foreground) / 0.04);
  border-radius: 8px;
  padding: 0.75rem 1rem;
  margin-top: 0.5rem;
  max-height: 300px;
  overflow-y: auto;
}
.detail-actions {
  margin-top: 0.5rem;
  display: flex;
  gap: 0.5rem;
}
.btn-sm {
  padding: 0.3rem 0.7rem;
  font-size: 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--af-border);
  background: hsl(var(--primary) / 0.08);
  color: hsl(var(--primary));
  cursor: pointer;
}
.btn-sm:hover {
  background: hsl(var(--primary) / 0.15);
}
</style>
