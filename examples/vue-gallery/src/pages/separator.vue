<script setup lang="ts">
import { Separator } from '@auto-ui/widgets/registry/separator'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const rows = [
  { prop: 'orientation', type: "'horizontal' | 'vertical'", default: "'horizontal'", desc: 'Axis of the divider (reka-ui SeparatorProps).' },
  { prop: 'decorative', type: 'boolean', default: 'true', desc: 'If true, excluded from a11y tree.' },
  { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
]
</script>

<template>
  <h2 class="text-2xl font-bold tracking-tight mb-6">Separator</h2>

  <DemoBlock title="Horizontal">
    <div class="w-full max-w-sm">
      <p class="text-sm">Above</p>
      <Separator class="my-2" />
      <p class="text-sm">Below</p>
    </div>
  </DemoBlock>

  <DemoBlock title="Vertical">
    <div class="flex h-8 items-center gap-3">
      <span class="text-sm">Left</span>
      <Separator orientation="vertical" />
      <span class="text-sm">Right</span>
    </div>
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
