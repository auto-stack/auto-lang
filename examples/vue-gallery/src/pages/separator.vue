<script setup lang="ts">
import { Separator } from '@auto-ui/widgets/registry/separator'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const codeHorizontal = `<script setup lang="ts">
import { Separator } from '@auto-ui/widgets/registry/separator'
<\/script>

<template>
  <div>
    <p>Above</p>
    <Separator />
    <p>Below</p>
  </div>
</template>`

const codeVertical = `<script setup lang="ts">
import { Separator } from '@auto-ui/widgets/registry/separator'
<\/script>

<template>
  <div style="display:flex;align-items:center;gap:0.75rem">
    <span>Left</span>
    <Separator orientation="vertical" />
    <span>Right</span>
  </div>
</template>`

const rows = [
  { prop: 'orientation', type: "'horizontal' | 'vertical'", default: "'horizontal'", desc: 'Axis of the divider (reka-ui SeparatorProps).' },
  { prop: 'decorative', type: 'boolean', default: 'true', desc: 'If true, excluded from a11y tree.' },
  { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
]
</script>

<template>
  <h2 class="page-title">Separator</h2>

  <DemoBlock title="Horizontal" :code="codeHorizontal">
    <div class="half">
      <p class="text-sm">Above</p>
      <Separator class="myblock" />
      <p class="text-sm">Below</p>
    </div>
  </DemoBlock>

  <DemoBlock title="Vertical" :code="codeVertical">
    <div class="vsep">
      <span class="text-sm">Left</span>
      <Separator orientation="vertical" />
      <span class="text-sm">Right</span>
    </div>
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
