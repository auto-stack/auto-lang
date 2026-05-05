<template>
  <div class="w-full">
    <div class="flex items-end gap-3 h-32 px-2">
      <div
        v-for="(item, i) in data"
        :key="i"
        class="flex-1 flex flex-col items-center gap-1"
      >
        <div
          class="w-full bg-accent-purple/80 rounded-t transition-all"
          :style="{ height: `${(item.value / maxValue) * 100}%` }"
        />
        <span class="text-[10px] text-text-secondary truncate max-w-full">{{ item.label }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { A2UIComponent } from '@/types/a2ui'

const props = defineProps<{
  component: A2UIComponent
  allComponents: Record<string, A2UIComponent>
  dataModel?: Record<string, any>
}>()

const data = computed(() => props.component.data || [])

const maxValue = computed(() => {
  const values = data.value.map((d: any) => d.value || 0)
  return Math.max(...values, 1)
})
</script>
