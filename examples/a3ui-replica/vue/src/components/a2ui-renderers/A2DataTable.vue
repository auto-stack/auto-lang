<template>
  <table class="w-full text-sm border-collapse">
    <thead>
      <tr class="border-b border-border-default">
        <th
          v-for="col in columns"
          :key="col"
          class="text-left px-3 py-2 text-text-secondary font-medium text-xs uppercase tracking-wider"
        >
          {{ col }}
        </th>
      </tr>
    </thead>
    <tbody>
      <tr
        v-for="(row, i) in rows"
        :key="i"
        class="border-b border-border-light"
      >
        <td
          v-for="col in columns"
          :key="col"
          class="px-3 py-2 text-text-primary"
        >
          {{ row[col] }}
        </td>
      </tr>
    </tbody>
  </table>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { A2UIComponent } from '@/types/a2ui'

const props = defineProps<{
  component: A2UIComponent
  allComponents: Record<string, A2UIComponent>
  dataModel?: Record<string, any>
}>()

const columns = computed(() => props.component.columns || [])
const rows = computed(() => props.component.rows || [])
</script>
