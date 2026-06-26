<script setup lang="ts">
import { ref } from 'vue'
import { Checkbox } from '@auto-ui/widgets/registry/checkbox'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const checked = ref(true)
</script>

<template>
  <h2 class="text-2xl font-bold tracking-tight mb-6">Checkbox</h2>

  <DemoBlock title="Controlled">
    <Checkbox v-model="checked" />
    <span class="text-sm text-muted-foreground">checked: {{ checked }}</span>
  </DemoBlock>

  <DemoBlock title="States">
    <Checkbox :model-value="true" />
    <Checkbox :model-value="false" />
    <Checkbox disabled />
  </DemoBlock>

  <PropTable :rows="[
    { prop: 'modelValue', type: 'boolean', default: '—', desc: 'Checked state (v-model).' },
    { prop: 'disabled', type: 'boolean', default: 'false', desc: 'Disable interaction (reka-ui).' },
    { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
  ]" />
</template>
