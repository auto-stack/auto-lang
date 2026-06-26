<script setup lang="ts">
import { Label } from '@auto-ui/widgets/registry/label'
import { Input } from '@auto-ui/widgets/registry/input'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'
</script>

<template>
  <h2 class="page-title">Label</h2>

  <DemoBlock title="Associated with an input" description="Click the label to focus the input (reka-ui Label `for`).">
    <div class="stack">
      <Label for="email">Email</Label>
      <Input id="email" placeholder="you@example.com" />
    </div>
  </DemoBlock>

  <DemoBlock title="Standalone">
    <Label>A standalone label</Label>
  </DemoBlock>

  <PropTable :rows="[
    { prop: 'for', type: 'string', default: '—', desc: 'ID of the associated form control (reka-ui LabelProps).' },
    { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
  ]" />
</template>
