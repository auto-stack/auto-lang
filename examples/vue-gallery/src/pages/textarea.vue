<script setup lang="ts">
import { ref } from 'vue'
import { Textarea } from '@auto-ui/widgets/registry/textarea'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const value = ref('Multi-line\ncontent')
</script>

<template>
  <h2 class="text-2xl font-bold tracking-tight mb-6">Textarea</h2>

  <DemoBlock title="Default (controlled)">
    <Textarea v-model="value" class="w-full max-w-md" />
    <pre class="text-xs text-muted-foreground whitespace-pre-wrap">{{ value }}</pre>
  </DemoBlock>

  <DemoBlock title="Placeholder">
    <Textarea placeholder="Write something…" class="w-full max-w-md" />
  </DemoBlock>

  <DemoBlock title="States">
    <Textarea disabled placeholder="disabled" class="w-full max-w-md" />
  </DemoBlock>

  <PropTable :rows="[
    { prop: 'modelValue', type: 'string | number', default: '—', desc: 'Bound value (v-model).' },
    { prop: 'defaultValue', type: 'string | number', default: '—', desc: 'Uncontrolled initial value.' },
    { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
  ]" />
</template>
