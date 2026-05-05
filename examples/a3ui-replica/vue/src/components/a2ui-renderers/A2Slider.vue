<template>
  <div class="w-full">
    <input
      type="range"
      :min="component.min || 0"
      :max="component.max || 100"
      :value="component.value || 50"
      class="w-full h-2 bg-bg-input rounded-lg appearance-none cursor-pointer accent-accent-purple"
      disabled
    />
  </div>
</template>

<script setup lang="ts">
import type { A2UIComponent } from '@/types/a2ui'

defineProps<{
  component: A2UIComponent
  allComponents: Record<string, A2UIComponent>
  dataModel?: Record<string, any>
}>()
</script>
