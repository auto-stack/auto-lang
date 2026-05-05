<template>
  <input
    :type="component.type || 'text'"
    :placeholder="component.placeholder || ''"
    :value="component.value || ''"
    class="w-full px-3 py-2 bg-bg-input border border-border-default rounded-md text-sm focus:outline-none focus:border-accent-purple focus:ring-2 focus:ring-accent-purple/10"
    readonly
  />
</template>

<script setup lang="ts">
import type { A2UIComponent } from '@/types/a2ui'

defineProps<{
  component: A2UIComponent
  allComponents: Record<string, A2UIComponent>
  dataModel?: Record<string, any>
}>()
</script>
