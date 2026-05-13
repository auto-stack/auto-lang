<template>
  <div class="relations-panel">
    <div v-if="loading" class="relations-loading">Loading…</div>
    <template v-else>
      <div v-if="parents.length" class="relations-group">
        <div class="relations-label">▲ Parents</div>
        <div
          v-for="p in parents"
          :key="p.id"
          class="relations-row"
          @click="$emit('jump', p.id)"
        >
          <StatusBadge :status="p.status as Status" size="sm" />
          <span class="relations-id">{{ p.id }}</span>
          <span class="relations-title">{{ p.title }}</span>
        </div>
      </div>
      <div v-if="children.length" class="relations-group">
        <div class="relations-label">▼ Children</div>
        <div
          v-for="c in children"
          :key="c.id"
          class="relations-row"
          @click="$emit('jump', c.id)"
        >
          <StatusBadge :status="c.status as Status" size="sm" />
          <span class="relations-id">{{ c.id }}</span>
          <span class="relations-title">{{ c.title }}</span>
        </div>
      </div>
      <div v-if="!parents.length && !children.length" class="relations-empty">
        No relations
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { watch } from 'vue'
import type { SpecItem, Status } from '@/types/specs'
import StatusBadge from './StatusBadge.vue'
import { useItemRelations } from '@/composables/useItemRelations'

const props = defineProps<{
  item: SpecItem
  project: string
}>()

const emit = defineEmits<{
  jump: [id: string]
}>()

const { loading, parents, children, loadRelations } = useItemRelations(props.project)

watch(
  () => props.item.id,
  (id) => {
    if (id) loadRelations(id)
  },
  { immediate: true }
)
</script>

<style scoped>
.relations-panel {
  background: hsl(var(--muted-foreground) / 0.04);
  border-radius: 8px;
  padding: 0.75rem 1rem;
  margin-bottom: 0.75rem;
}
.relations-loading {
  font-size: 0.75rem;
  color: var(--af-muted);
}
.relations-group {
  margin-bottom: 0.5rem;
}
.relations-group:last-child {
  margin-bottom: 0;
}
.relations-label {
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--af-muted);
  margin-bottom: 0.35rem;
}
.relations-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.3rem 0.5rem;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.12s;
}
.relations-row:hover {
  background: hsl(var(--muted-foreground) / 0.06);
}
.relations-id {
  font-family: monospace;
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--af-muted);
  flex-shrink: 0;
}
.relations-title {
  font-size: 0.75rem;
  color: var(--af-fg);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.relations-empty {
  font-size: 0.75rem;
  color: var(--af-muted);
  font-style: italic;
}
</style>
