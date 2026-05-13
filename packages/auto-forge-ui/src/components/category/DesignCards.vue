<template>
  <CategoryList
    :items="items"
    :project="project"
    :expanded-id="expandedId"
    :editing-id="editingId"
    section-type="designs"
    :summary-fn="extractDesignSummary"
    @toggle="$emit('toggle', $event)"
    @jump="$emit('jump', $event)"
    @edit="$emit('edit', $event)"
    @status-change="$emit('status-change', $event)"
    @delete="$emit('delete', $event)"
    @save="$emit('save', $event)"
    @cancel-edit="$emit('cancel-edit')"
  >
    <template #detail="{ item: rowItem, project }">
      <template v-if="editingId === rowItem.id">
        <MarkdownEditor
          :content="rowItem.content"
          @save="$emit('save', { ...rowItem, content: $event, modified_at: Date.now() })"
          @cancel="$emit('cancel-edit')"
          @link-click="$emit('jump', $event)"
        />
      </template>
      <SpecItemDetail
        v-else
        :item="rowItem"
        section-type="designs"
        :project="project"
        @jump="$emit('jump', $event)"
        @edit="$emit('edit', rowItem)"
        @status-change="$emit('status-change', $event)"
        @delete="$emit('delete', rowItem.id)"
      />
    </template>
  </CategoryList>
</template>

<script setup lang="ts">
import type { SpecItem } from '@/types/specs'
import CategoryList from './CategoryList.vue'
import SpecItemDetail from '@/components/SpecItemDetail.vue'
import MarkdownEditor from '@/components/editors/MarkdownEditor.vue'
import { extractDesignSummary } from '@/utils/categorySummary'

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
