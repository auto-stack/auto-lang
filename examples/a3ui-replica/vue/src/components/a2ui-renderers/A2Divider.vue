<template>
  <div
    class="w-full border-t border-border-light"
    :style="{ margin: `${component.spacing || 16}px 0` }"
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
