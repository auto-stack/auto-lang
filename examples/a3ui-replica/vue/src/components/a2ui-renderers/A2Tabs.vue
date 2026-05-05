<template>
  <div>
    <div class="flex border-b border-border-default mb-3">
      <button
        v-for="(tab, i) in tabs"
        :key="i"
        :class="[
          'px-4 py-2 text-sm font-medium transition-colors cursor-pointer border-b-2',
          activeIdx === i
            ? 'text-text-primary border-text-primary font-semibold'
            : 'text-text-secondary border-transparent hover:text-text-primary'
        ]"
        @click="activeIdx = i"
      >
        {{ tab }}
      </button>
    </div>
    <div class="tab-content">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { A2UIComponent } from '@/types/a2ui'

const props = defineProps<{
  component: A2UIComponent
  allComponents: Record<string, A2UIComponent>
  dataModel?: Record<string, any>
}>()

const tabs = computed(() => props.component.tabs || [])
const activeIdx = ref(props.component.activeTab || 0)
</script>
