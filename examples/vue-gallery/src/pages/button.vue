<script setup lang="ts">
import { Button } from '@auto-ui/widgets/registry/button'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const variants = ['default', 'destructive', 'outline', 'secondary', 'ghost', 'link'] as const
const sizes = ['default', 'sm', 'lg', 'icon'] as const

const rows = [
  { prop: 'variant', type: "'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link'", default: "'default'", desc: 'Visual style.' },
  { prop: 'size', type: "'default' | 'sm' | 'lg' | 'icon'", default: "'default'", desc: 'Size preset.' },
  { prop: 'as', type: 'string', default: "'button'", desc: 'Underlying element/tag (via reka-ui Primitive).' },
  { prop: 'asChild', type: 'boolean', default: 'false', desc: 'Render as the single slot child instead of a <button>.' },
  { prop: 'class', type: 'string', default: '—', desc: 'Extra classes merged via cn().' },
]
</script>

<template>
  <h2 class="text-2xl font-bold tracking-tight mb-6">Button</h2>

  <DemoBlock title="Variants">
    <Button v-for="v in variants" :key="v" :variant="v">{{ v }}</Button>
  </DemoBlock>

  <DemoBlock title="Sizes">
    <Button v-for="s in sizes" :key="s" :size="s" :aria-label="s === 'icon' ? s : undefined">
      <span v-if="s !== 'icon'">{{ s }}</span>
      <svg v-else xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="h-4 w-4"><polyline points="9 18 15 12 9 6" /></svg>
    </Button>
  </DemoBlock>

  <DemoBlock title="States">
    <Button>Normal</Button>
    <Button disabled>Disabled</Button>
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
