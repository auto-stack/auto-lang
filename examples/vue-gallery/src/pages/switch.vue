<script setup lang="ts">
import { ref } from 'vue'
import { Switch } from '@auto-ui/widgets/registry/switch'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const on = ref(true)
</script>

<template>
  <h2 class="text-2xl font-bold tracking-tight mb-6">Switch</h2>

  <DemoBlock title="Controlled">
    <Switch v-model="on" />
    <span class="text-sm text-muted-foreground">on: {{ on }}</span>
  </DemoBlock>

  <DemoBlock title="States">
    <Switch :model-value="true" />
    <Switch :model-value="false" />
    <Switch disabled />
  </DemoBlock>

  <PropTable :rows="[
    { prop: 'modelValue', type: 'boolean', default: '—', desc: 'On/off state (v-model).' },
    { prop: 'disabled', type: 'boolean', default: 'false', desc: 'Disable interaction (reka-ui).' },
    { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
  ]" />
</template>
