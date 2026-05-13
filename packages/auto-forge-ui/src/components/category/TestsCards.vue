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
    @save="$emit('save', $event)"
    >
      <template #detail="{ item: rowItem }">
        <TestEditor
          v-if="editingId === rowItem.id"
          :item="rowItem"
          @save="onTestSave(rowItem, $event)"
          @cancel="$emit('cancel-edit')"
        />
        <SpecItemDetail
          v-else
          :item="rowItem"
          section-type="tests"
          :project="project"
          @jump="$emit('jump', $event)"
          @edit="$emit('edit', rowItem)"
          @status-change="$emit('status-change', $event)"
          @delete="$emit('delete', rowItem.id)"
        >
          <template #content="{ item }">
            <TestDetail :content="item.content" :test-file="item.test_file" @link-click="$emit('jump', $event)" />
          </template>
        </SpecItemDetail>
      </template>
    </SpecItemRow>
  </div>
</template>

<script setup lang="ts">
import type { SpecItem } from '@/types/specs'
import SpecItemRow from '@/components/SpecItemRow.vue'
import SpecItemDetail from '@/components/SpecItemDetail.vue'
import TestEditor from '@/components/editors/TestEditor.vue'
import TestDetail from '@/components/detail/TestDetail.vue'
import { extractTestSummary } from '@/utils/categorySummary'

const props = defineProps<{
  items: SpecItem[]
  project: string
  expandedId: string | null
  editingId: string | null
}>()

const emit = defineEmits<{
  toggle: [id: string]
  jump: [id: string]
  edit: [item: SpecItem]
  'status-change': [payload: { id: string; status: string }]
  delete: [id: string]
  save: [item: SpecItem]
  'cancel-edit': []
}>()

function onTestSave(item: SpecItem, payload: { title: string; content: string; test_file: string }) {
  emit('save', { ...item, title: payload.title, content: payload.content, test_file: payload.test_file, modified_at: Date.now() })
  emit('cancel-edit')
}
</script>

<style scoped>
.tests-cards {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}
</style>
