<template>
  <div
    class="spec-item-row"
    :class="{ expanded: isExpanded, [`cat-${sectionType}`]: true }"
    @click="$emit('toggle', item.id)"
  >
    <div class="row-main">
      <!-- ID badge -->
      <span class="id-badge">{{ item.id }}</span>

      <!-- Title + summary -->
      <div class="row-body">
        <span class="row-title">{{ item.title }}</span>
        <span v-if="summary" class="row-summary">{{ summary }}</span>
      </div>

      <!-- Right side -->
      <div class="row-meta">
        <StatusBadge :status="item.status" size="sm" />
        <component
          :is="isExpanded ? ChevronUp : ChevronDown"
          :size="14"
          class="expand-icon"
        />
      </div>
    </div>

    <!-- Expanded detail -->
    <div v-if="isExpanded" class="row-detail" @click.stop>
      <slot name="detail" :item="item">
        <SpecItemDetail
          :item="item"
          :section-type="sectionType"
          :project="project"
          @jump="$emit('jump', $event)"
          @edit="$emit('edit', item)"
          @status-change="$emit('status-change', $event)"
          @delete="$emit('delete', item.id)"
        />
      </slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SpecItem, SectionType } from '@/types/specs'
import StatusBadge from './StatusBadge.vue'
import SpecItemDetail from './SpecItemDetail.vue'
import { ChevronDown, ChevronUp } from 'lucide-vue-next'

const props = defineProps<{
  item: SpecItem
  sectionType: SectionType
  project: string
  isExpanded: boolean
  summary?: string
}>()

defineEmits<{
  toggle: [id: string]
  jump: [id: string]
  edit: [item: SpecItem]
  'status-change': [payload: { id: string; status: string }]
  delete: [id: string]
}>()
</script>

<style scoped>
.spec-item-row {
  border-radius: 10px;
  background: var(--af-card);
  border: 1px solid var(--af-border);
  transition: all 0.15s ease;
  overflow: hidden;
}

.spec-item-row:hover {
  border-color: hsl(var(--muted-foreground) / 0.25);
  box-shadow: 0 1px 3px hsl(var(--muted-foreground) / 0.06);
}

.spec-item-row.expanded {
  border-color: hsl(var(--primary) / 0.25);
  box-shadow: 0 2px 8px hsl(var(--muted-foreground) / 0.08);
}

/* Category accent borders on left */
.spec-item-row.cat-goals { border-left: 3px solid #10b981; }
.spec-item-row.cat-architecture { border-left: 3px solid #8b5cf6; }
.spec-item-row.cat-designs { border-left: 3px solid #ec4899; }
.spec-item-row.cat-plans { border-left: 3px solid #f59e0b; }
.spec-item-row.cat-tests { border-left: 3px solid #06b6d4; }
.spec-item-row.cat-reviews { border-left: 3px solid #6366f1; }
.spec-item-row.cat-reports { border-left: 3px solid #14b8a6; }


.row-main {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.7rem 1rem;
  cursor: pointer;
}

.id-badge {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.72rem;
  font-weight: 700;
  padding: 0.2rem 0.5rem;
  border-radius: 6px;
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-muted);
  flex-shrink: 0;
}

.row-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.row-title {
  font-size: 0.88rem;
  font-weight: 500;
  color: var(--af-fg);
  line-height: 1.35;
}

.row-summary {
  font-size: 0.75rem;
  color: var(--af-muted);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-shrink: 0;
}

.expand-icon {
  color: var(--af-muted);
  transition: transform 0.2s;
}

.row-detail {
  padding: 0 1rem 1rem;
  border-top: 1px solid var(--af-border);
  margin-top: 0;
}
</style>
