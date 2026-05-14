<template>
  <div class="relations-panel">
    <div v-if="loading" class="relations-loading">
      <Loader2 :size="14" class="spin" />
      Loading relations…
    </div>
    <template v-else>
      <!-- Parents -->
      <div v-if="parents.length" class="relations-group">
        <div class="relations-label">
          <ArrowUp :size="12" />
          Parents
          <span class="relations-count">{{ parents.length }}</span>
        </div>
        <div class="relations-list">
          <div
            v-for="p in parents"
            :key="p.id"
            class="relations-chip"
            @click="$emit('jump', p.id)"
          >
            <span class="chip-icon">{{ sectionIcon(p.section_type) }}</span>
            <span class="chip-id">{{ p.id }}</span>
            <span class="chip-title">{{ p.title }}</span>
            <StatusBadge :status="p.status as Status" size="sm" />
          </div>
        </div>
      </div>

      <!-- Children -->
      <div v-if="children.length" class="relations-group">
        <div class="relations-label">
          <ArrowDown :size="12" />
          Children
          <span class="relations-count">{{ children.length }}</span>
        </div>
        <div class="relations-list">
          <div
            v-for="c in children"
            :key="c.id"
            class="relations-chip"
            @click="$emit('jump', c.id)"
          >
            <span class="chip-icon">{{ sectionIcon(c.section_type) }}</span>
            <span class="chip-id">{{ c.id }}</span>
            <span class="chip-title">{{ c.title }}</span>
            <StatusBadge :status="c.status as Status" size="sm" />
          </div>
        </div>
      </div>

      <div v-if="!parents.length && !children.length" class="relations-empty">
        <Unlink :size="14" />
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
import { ArrowUp, ArrowDown, Loader2, Unlink } from 'lucide-vue-next'

const props = defineProps<{
  item: SpecItem
  project: string
}>()

const emit = defineEmits<{
  jump: [id: string]
}>()

const { loading, parents, children, loadRelations } = useItemRelations(props.project)

function sectionIcon(type: string): string {
  const map: Record<string, string> = {
    goals: '🎯',
    architecture: '🏗️',
    designs: '🎨',
    plans: '📅',
    tests: '🧪',
    reviews: '📝',
    reports: '📊',

  }
  return map[type] || '📄'
}

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
  border-radius: 10px;
  padding: 0.75rem 1rem;
  margin-bottom: 0.75rem;
}

.relations-loading {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.75rem;
  color: var(--af-muted);
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.relations-group {
  margin-bottom: 0.6rem;
}
.relations-group:last-child {
  margin-bottom: 0;
}

.relations-label {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--af-muted);
  margin-bottom: 0.4rem;
}

.relations-count {
  margin-left: auto;
  font-size: 0.6rem;
  padding: 0.05rem 0.35rem;
  border-radius: 999px;
  background: hsl(var(--muted-foreground) / 0.1);
  color: var(--af-muted);
}

.relations-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}

.relations-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.3rem 0.55rem;
  border-radius: 8px;
  background: var(--af-card);
  border: 1px solid var(--af-border);
  cursor: pointer;
  transition: all 0.12s;
  font-size: 0.78rem;
}

.relations-chip:hover {
  border-color: hsl(var(--primary) / 0.3);
  background: hsl(var(--primary) / 0.04);
}

.chip-icon {
  font-size: 0.85rem;
  line-height: 1;
}

.chip-id {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-weight: 600;
  color: hsl(var(--primary));
  font-size: 0.75rem;
}

.chip-title {
  color: var(--af-fg);
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.relations-empty {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.75rem;
  color: var(--af-muted);
  padding: 0.5rem 0;
}
</style>
