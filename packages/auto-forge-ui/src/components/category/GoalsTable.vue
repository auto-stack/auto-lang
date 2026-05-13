<template>
  <div class="goals-table-wrapper">
    <table class="goals-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>Goal</th>
          <th>Priority</th>
          <th>Status</th>
          <th>Children</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="item in items"
          :key="item.id"
          :class="{ expanded: expandedId === item.id }"
          @click="toggle(item.id)"
        >
          <td class="col-id">{{ item.id }}</td>
          <td class="col-goal">{{ item.title }}</td>
          <td class="col-priority">
            <span class="priority-badge" :class="item.priority">{{ item.priority || '-' }}</span>
          </td>
          <td class="col-status">
            <StatusBadge :status="item.status" size="sm" />
          </td>
          <td class="col-children">
            <span v-if="item.related?.length" class="children-count">
              {{ item.related.length }}
            </span>
            <span v-else class="children-empty">—</span>
          </td>
        </tr>
        <tr v-if="expandedItem" class="detail-row">
          <td :colspan="5">
            <RelationsPanel :item="expandedItem" :project="project" @jump="$emit('jump', $event)" />
            <div class="detail-actions">
              <button class="btn-sm" @click.stop="$emit('edit', expandedItem)">Edit</button>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { SpecItem } from '@/types/specs'
import StatusBadge from '@/components/StatusBadge.vue'
import RelationsPanel from '@/components/RelationsPanel.vue'

const props = defineProps<{
  items: SpecItem[]
  project: string
  expandedId: string | null
}>()

const emit = defineEmits<{
  toggle: [id: string]
  jump: [id: string]
  edit: [item: SpecItem]
}>()

const expandedItem = computed(() =>
  props.expandedId ? props.items.find((i) => i.id === props.expandedId) ?? null : null
)

function toggle(id: string) {
  emit('toggle', id)
}
</script>

<style scoped>
.goals-table-wrapper {
  overflow-x: auto;
}
.goals-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.85rem;
}
.goals-table th {
  text-align: left;
  padding: 0.6rem 0.75rem;
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  color: var(--af-muted);
  border-bottom: 1px solid var(--af-border);
}
.goals-table td {
  padding: 0.55rem 0.75rem;
  border-bottom: 1px solid var(--af-border);
  cursor: pointer;
}
.goals-table tbody tr:hover {
  background: hsl(var(--muted-foreground) / 0.03);
}
.goals-table tbody tr.expanded {
  background: hsl(var(--primary) / 0.04);
}
.col-id {
  font-family: monospace;
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--af-muted);
  white-space: nowrap;
}
.col-goal {
  color: var(--af-fg);
  line-height: 1.4;
}
.priority-badge {
  font-size: 0.7rem;
  font-weight: 700;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  background: hsl(var(--muted-foreground) / 0.08);
}
.priority-badge.P0 {
  color: #ef4444;
  background: hsl(0 84% 60% / 0.12);
}
.priority-badge.P1 {
  color: #f59e0b;
  background: hsl(38 92% 50% / 0.12);
}
.priority-badge.P2 {
  color: #3b82f6;
  background: hsl(217 91% 60% / 0.12);
}
.children-count {
  font-size: 0.75rem;
  font-weight: 600;
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 0.08);
  padding: 0.1rem 0.4rem;
  border-radius: 999px;
}
.children-empty {
  color: var(--af-muted);
  font-size: 0.8rem;
}
.detail-row td {
  padding: 0.75rem;
  background: hsl(var(--muted-foreground) / 0.02);
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
