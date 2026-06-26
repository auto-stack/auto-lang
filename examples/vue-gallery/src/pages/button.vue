<script setup lang="ts">
import { Button } from '@auto-ui/widgets/registry/button'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const variants = ['default', 'destructive', 'outline', 'secondary', 'ghost', 'link'] as const
const sizes = ['default', 'sm', 'lg', 'icon'] as const

const codeVariants = `<script setup lang="ts">
import { Button } from '@auto-ui/widgets/registry/button'
<\/script>

<template>
  <Button variant="default">Default</Button>
  <Button variant="destructive">Destructive</Button>
  <Button variant="outline">Outline</Button>
  <Button variant="secondary">Secondary</Button>
  <Button variant="ghost">Ghost</Button>
  <Button variant="link">Link</Button>
</template>`

const codeSizes = `<script setup lang="ts">
import { Button } from '@auto-ui/widgets/registry/button'
<\/script>

<template>
  <Button size="default">Default</Button>
  <Button size="sm">Small</Button>
  <Button size="lg">Large</Button>
  <Button size="icon" aria-label="Next">
    <svg viewBox="0 0 24 24" class="h-4 w-4"><!-- icon --></svg>
  </Button>
</template>`

const codeStates = `<script setup lang="ts">
import { Button } from '@auto-ui/widgets/registry/button'
<\/script>

<template>
  <Button>Normal</Button>
  <Button disabled>Disabled</Button>
</template>`

const rows = [
  { prop: 'variant', type: "'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link'", default: "'default'", desc: 'Visual style.' },
  { prop: 'size', type: "'default' | 'sm' | 'lg' | 'icon'", default: "'default'", desc: 'Size preset.' },
  { prop: 'as', type: 'string', default: "'button'", desc: 'Underlying element/tag (via reka-ui Primitive).' },
  { prop: 'asChild', type: 'boolean', default: 'false', desc: 'Render as the single slot child instead of a <button>.' },
  { prop: 'class', type: 'string', default: '—', desc: 'Extra classes merged via cn().' },
]
</script>

<template>
  <h2 class="page-title">Button</h2>

  <DemoBlock title="Variants" :code="codeVariants">
    <Button v-for="v in variants" :key="v" :variant="v">{{ v }}</Button>
  </DemoBlock>

  <DemoBlock title="Sizes" :code="codeSizes">
    <Button v-for="s in sizes" :key="s" :size="s" :aria-label="s === 'icon' ? s : undefined">
      <span v-if="s !== 'icon'">{{ s }}</span>
      <svg v-else xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="h-4 w-4"><polyline points="9 18 15 12 9 6" /></svg>
    </Button>
  </DemoBlock>

  <DemoBlock title="States" :code="codeStates">
    <Button>Normal</Button>
    <Button disabled>Disabled</Button>
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
