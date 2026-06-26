<script setup lang="ts">
import { ref } from 'vue'
import { Input } from '@auto-ui/widgets/registry/input'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const value = ref('Hello')
</script>

<template>
  <h2 class="page-title">Input</h2>

  <DemoBlock title="Default (controlled)" description="Two-way bound via v-model.">
    <Input v-model="value" />
    <span class="text-sm muted">value: {{ value }}</span>
  </DemoBlock>

  <DemoBlock title="Placeholder">
    <Input placeholder="you@example.com" />
  </DemoBlock>

  <DemoBlock title="States">
    <Input model-value="readonly-ish" />
    <Input disabled placeholder="disabled" />
  </DemoBlock>

  <PropTable :rows="[
    { prop: 'modelValue', type: 'string | number', default: '—', desc: 'Bound value (v-model).' },
    { prop: 'defaultValue', type: 'string | number', default: '—', desc: 'Uncontrolled initial value.' },
    { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
  ]" />
</template>
