<template>
  <label class="flex items-center gap-2 cursor-pointer">
    <input
      type="checkbox"
      :checked="component.checked || false"
      class="w-4 h-4 rounded border-border-default text-accent-purple focus:ring-accent-purple"
      disabled
    />
    <span class="text-sm">{{ component.label }}</span>
  </label>
</template>

<script setup lang="ts">
import type { A2UIComponent } from '@/types/a2ui'

defineProps<{
  component: A2UIComponent
  allComponents: Record<string, A2UIComponent>
  dataModel?: Record<string, any>
}>()
</script>
