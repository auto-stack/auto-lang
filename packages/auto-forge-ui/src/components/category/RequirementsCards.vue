<template>
  <div class="requirements-cards">
    <div
      v-for="item in items"
      :key="item.id"
      class="req-card"
      :class="{ expanded: expandedId === item.id }"
      @click="$emit('toggle', item.id)"
    >
      <div class="req-header">
        <span class="req-id">{{ item.id }}</span>
        <StatusBadge :status="item.status" size="sm" />
        <span v-if="item.priority" class="req-priority" :class="item.priority">{{ item.priority }}</span>
      </div>
      <h4 class="req-title">{{ item.title }}</h4>
      <div v-if="checklists[item.id]?.length" class="req-checklist-preview">
        <div
          v-for="(check, i) in checklists[item.id].slice(0, 3)"
          :key="i"
          class="req-check"
        >
          <span class="check-box">{{ check.done ? '☑' : '☐' }}</span>
          <span :class="{ done: check.done }">{{ check.text }}</span>
        </div>
        <div v-if="checklists[item.id].length > 3" class="req-more">
          +{{ checklists[item.id].length - 3 }} more
        </div>
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

interface CheckItem {
  done: boolean
  text: string
}

const CHECK_RE = /^\s*-\s*\[(.?)\]\s*(.+)$/gm

const checklists = computed(() => {
  const map: Record<string, CheckItem[]> = {}
  for (const item of props.items) {
    const list: CheckItem[] = []
    let m: RegExpExecArray | null
    while ((m = CHECK_RE.exec(item.content)) !== null) {
      list.push({ done: m[1] === 'x' || m[1] === 'X', text: m[2].trim() })
    }
    map[item.id] = list
  }
  return map
})
</script>

<style scoped>
.requirements-cards {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
.req-card {
  padding: 0.85rem 1rem;
  border-radius: 10px;
  background: hsl(var(--muted-foreground) / 0.02);
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.12s;
}
.req-card:hover {
  background: hsl(var(--muted-foreground) / 0.05);
  border-color: var(--af-border);
}
.req-card.expanded {
  background: hsl(var(--muted-foreground) / 0.06);
  border-color: hsl(var(--primary) / 0.2);
}
.req-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.4rem;
}
.req-id {
  font-family: monospace;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--af-muted);
}
.req-title {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--af-fg);
  margin: 0 0 0.5rem 0;
  line-height: 1.3;
}
.req-priority {
  font-size: 0.65rem;
  font-weight: 700;
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
  margin-left: auto;
}
.req-priority.P0 {
  color: #ef4444;
  background: hsl(0 84% 60% / 0.12);
}
.req-priority.P1 {
  color: #f59e0b;
  background: hsl(38 92% 50% / 0.12);
}
.req-checklist-preview {
  margin-top: 0.4rem;
}
.req-check {
  display: flex;
  align-items: flex-start;
  gap: 0.4rem;
  font-size: 0.78rem;
  color: var(--af-fg);
  line-height: 1.4;
  margin-bottom: 0.2rem;
}
.req-check .done {
  text-decoration: line-through;
  color: var(--af-muted);
}
.check-box {
  flex-shrink: 0;
  font-size: 0.85rem;
}
.req-more {
  font-size: 0.7rem;
  color: var(--af-muted);
  margin-top: 0.2rem;
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
