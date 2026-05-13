<template>
  <CategoryList
    :items="items"
    :project="project"
    :expanded-id="expandedId"
    :editing-id="editingId"
    section-type="reports"
    :summary-fn="extractReportSummary"
    @toggle="$emit('toggle', $event)"
    @jump="$emit('jump', $event)"
    @edit="$emit('edit', $event)"
    @status-change="$emit('status-change', $event)"
    @delete="$emit('delete', $event)"
    @save="$emit('save', $event)"
  />
</template>

<script setup lang="ts">
import type { SpecItem } from '@/types/specs'
import CategoryList from './CategoryList.vue'
import { extractReportSummary } from '@/utils/categorySummary'

defineProps<{
  items: SpecItem[]
  project: string
  expandedId: string | null
  editingId: string | null
}>()

defineEmits<{
  toggle: [id: string]
  jump: [id: string]
  edit: [item: SpecItem]
  'status-change': [payload: { id: string; status: string }]
  delete: [id: string]
  save: [item: SpecItem]
  'cancel-edit': []
}>()
</script>
