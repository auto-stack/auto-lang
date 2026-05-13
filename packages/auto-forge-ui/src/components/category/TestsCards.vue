<template>
  <div class="tests-cards">
    <SpecItemRow
      v-for="item in items"
      :key="item.id"
      :item="item"
      section-type="tests"
      :project="project"
      :is-expanded="expandedId === item.id"
      :summary="extractTestSummary(item)"
      @toggle="$emit('toggle', $event)"
      @jump="$emit('jump', $event)"
      @edit="$emit('edit', $event)"
      @status-change="$emit('status-change', $event)"
      @delete="$emit('delete', $event)"
    >
      <template #detail="{ item: rowItem }">
        <SpecItemDetail
          :item="rowItem"
          section-type="tests"
          :project="project"
          @jump="$emit('jump', $event)"
          @edit="$emit('edit', rowItem)"
          @status-change="$emit('status-change', $event)"
          @delete="$emit('delete', rowItem.id)"
        />
      </template>
    </SpecItemRow>
  </div>
</template>

<script setup lang="ts">
import type { SpecItem } from '@/types/specs'
import SpecItemRow from '@/components/SpecItemRow.vue'
import SpecItemDetail from '@/components/SpecItemDetail.vue'
import { extractTestSummary } from '@/utils/categorySummary'

defineProps<{
  items: SpecItem[]
  project: string
  expandedId: string | null
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
.tests-cards {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}
</style>
