<template>
  <div class="goals-table-wrapper">
    <div v-if="items.length === 0" class="empty-state">
      <Inbox :size="28" />
      <span>No goals yet</span>
      <span class="empty-hint">Click "Add" above to create one</span>
    </div>
    <table v-else class="goals-table">
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
        <template v-for="item in items" :key="item.id">
          <tr
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
          <tr v-if="expandedId === item.id" class="detail-row">
            <td :colspan="5">
              <SpecItemDetail
                :item="item"
                section-type="goals"
                :project="project"
                @jump="$emit('jump', $event)"
                @edit="$emit('edit', item)"
                @status-change="$emit('status-change', $event)"
                @delete="$emit('delete', item.id)"
              />
            </td>
          </tr>
        </template>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import type { SpecItem } from '@/types/specs'
import StatusBadge from '@/components/StatusBadge.vue'
import SpecItemDetail from '@/components/SpecItemDetail.vue'
import { Inbox } from 'lucide-vue-next'

const props = defineProps<{
  items: SpecItem[]
  project: string
  expandedId: string | null
}>()

const emit = defineEmits<{
  toggle: [id: string]
  jump: [id: string]
  edit: [item: SpecItem]
  'status-change': [payload: { id: string; status: string }]
  delete: [id: string]
}>()

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
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-weight: 600;
  color: var(--af-muted);
  font-size: 0.75rem;
}
.col-goal {
  color: var(--af-fg);
  font-weight: 500;
}
.priority-badge {
  display: inline-flex;
  padding: 0.1rem 0.4rem;
  font-size: 0.65rem;
  font-weight: 700;
  border-radius: 4px;
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-muted);
}
.priority-badge.P0 {
  background: hsl(0 72% 51% / 0.12);
  color: #ef4444;
}
.priority-badge.P1 {
  background: hsl(38 92% 50% / 0.12);
  color: #f59e0b;
}
.priority-badge.P2 {
  background: hsl(217 91% 60% / 0.12);
  color: #3b82f6;
}
.children-count {
  display: inline-flex;
  padding: 0.1rem 0.4rem;
  font-size: 0.65rem;
  font-weight: 600;
  border-radius: 999px;
  background: hsl(var(--primary) / 0.08);
  color: hsl(var(--primary));
}
.children-empty {
  color: var(--af-muted);
  font-size: 0.8rem;
}
.detail-row td {
  padding: 0.75rem;
  background: hsl(var(--muted-foreground) / 0.02);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.4rem;
  padding: 2.5rem 1rem;
  color: var(--af-muted);
  font-size: 0.85rem;
}
.empty-state svg {
  color: hsl(var(--muted-foreground) / 0.3);
  margin-bottom: 0.3rem;
}
.empty-hint {
  font-size: 0.75rem;
  color: hsl(var(--muted-foreground) / 0.6);
}
</style>
