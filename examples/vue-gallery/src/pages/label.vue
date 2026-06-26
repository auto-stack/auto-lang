<script setup lang="ts">
import { Label } from '@auto-ui/widgets/registry/label'
import { Input } from '@auto-ui/widgets/registry/input'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const codeAssociated = `<script setup lang="ts">
import { Label } from '@auto-ui/widgets/registry/label'
import { Input } from '@auto-ui/widgets/registry/input'
<\/script>

<template>
  <Label for="email">Email</Label>
  <Input id="email" placeholder="you@example.com" />
</template>`

const codeStandalone = `<script setup lang="ts">
import { Label } from '@auto-ui/widgets/registry/label'
<\/script>

<template>
  <Label>A standalone label</Label>
</template>`

const rows = [
  { prop: 'for', type: 'string', default: '—', desc: 'ID of the associated form control (reka-ui LabelProps).' },
  { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
]
</script>

<template>
  <h2 class="page-title">Label</h2>

  <DemoBlock title="Associated with an input" description="Click the label to focus the input (reka-ui Label `for`)." :code="codeAssociated">
    <div class="stack">
      <Label for="email">Email</Label>
      <Input id="email" placeholder="you@example.com" />
    </div>
  </DemoBlock>

  <DemoBlock title="Standalone" :code="codeStandalone">
    <Label>A standalone label</Label>
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
