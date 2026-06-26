<script setup lang="ts">
import { ref } from 'vue'
import { Textarea } from '@auto-ui/widgets/registry/textarea'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const value = ref('Multi-line\ncontent')
</script>

<template>
  <h2 class="page-title">Textarea</h2>

  <DemoBlock title="Default (controlled)">
    <Textarea v-model="value" class="half" />
    <pre class="text-sm muted whitespace-pre">{{ value }}</pre>
  </DemoBlock>

  <DemoBlock title="Placeholder">
    <Textarea placeholder="Write something…" class="half" />
  </DemoBlock>

  <DemoBlock title="States">
    <Textarea disabled placeholder="disabled" class="half" />
  </DemoBlock>

  <PropTable :rows="[
    { prop: 'modelValue', type: 'string | number', default: '—', desc: 'Bound value (v-model).' },
    { prop: 'defaultValue', type: 'string | number', default: '—', desc: 'Uncontrolled initial value.' },
    { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
  ]" />
</template>
