<template>
  <div class="category-list">
    <div v-if="items.length === 0" class="empty-state">
      <Inbox :size="28" />
      <span>No items yet</span>
      <span class="empty-hint">Click "Add" above to create one</span>
    </div>
    <SpecItemRow
      v-for="item in items"
      :key="item.id"
      :item="item"
      :section-type="sectionType"
      :project="project"
      :is-expanded="expandedId === item.id"
      :summary="summaryFn(item)"
      @toggle="$emit('toggle', $event)"
      @jump="$emit('jump', $event)"
      @edit="$emit('edit', $event)"
      @status-change="$emit('status-change', $event)"
      @delete="$emit('delete', $event)"
    >
      <template #detail="{ item: rowItem }">
        <slot name="detail" :item="rowItem" :project="project">
          <SpecItemDetail
            :item="rowItem"
            :section-type="sectionType"
            :project="project"
            @jump="$emit('jump', $event)"
            @edit="$emit('edit', rowItem)"
            @status-change="$emit('status-change', $event)"
            @delete="$emit('delete', rowItem.id)"
          />
        </slot>
      </template>
    </SpecItemRow>
  </div>
</template>

<script setup lang="ts">
import type { SpecItem, SectionType } from '@/types/specs'
import SpecItemRow from '@/components/SpecItemRow.vue'
import SpecItemDetail from '@/components/SpecItemDetail.vue'
import { Inbox } from 'lucide-vue-next'

defineProps<{
  items: SpecItem[]
  project: string
  expandedId: string | null
  sectionType: SectionType
  summaryFn: (item: SpecItem) => string
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
.category-list {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
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
