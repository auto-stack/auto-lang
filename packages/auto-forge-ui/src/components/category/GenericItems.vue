<template>
  <div class="generic-items">
    <div
      v-for="item in items"
      :key="item.id"
      class="item-row"
      :class="{ expanded: expandedId === item.id }"
      @click="$emit('toggle', item.id)"
    >
      <div class="item-header">
        <span class="item-id">{{ item.id }}</span>
        <span class="item-title">{{ item.title }}</span>
        <StatusBadge :status="item.status" size="sm" />
      </div>
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
import type { SpecItem } from '@/types/specs'
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
</script>

<style scoped>
.generic-items {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.item-row {
  padding: 0.65rem 0.85rem;
  border-radius: 8px;
  background: hsl(var(--muted-foreground) / 0.02);
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.12s;
}
.item-row:hover {
  background: hsl(var(--muted-foreground) / 0.05);
  border-color: var(--af-border);
}
.item-row.expanded {
  background: hsl(var(--muted-foreground) / 0.06);
  border-color: hsl(var(--primary) / 0.2);
}
.item-header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}
.item-id {
  font-family: monospace;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--af-muted);
  flex-shrink: 0;
}
.item-title {
  font-size: 0.85rem;
  color: var(--af-fg);
  flex: 1;
  line-height: 1.3;
}
.item-detail {
  margin-top: 0.6rem;
  padding-top: 0.6rem;
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
