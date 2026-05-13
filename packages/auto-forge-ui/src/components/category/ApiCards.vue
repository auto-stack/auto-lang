<template>
  <CategoryList
    :items="items"
    :project="project"
    :expanded-id="expandedId"
    section-type="apis"
    :summary-fn="extractApiSummary"
    @toggle="$emit('toggle', $event)"
    @jump="$emit('jump', $event)"
    @edit="$emit('edit', $event)"
    @status-change="$emit('status-change', $event)"
    @delete="$emit('delete', $event)"
  />
</template>

<script setup lang="ts">
import type { SpecItem } from '@/types/specs'
import CategoryList from './CategoryList.vue'
import { extractApiSummary } from '@/utils/categorySummary'

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
