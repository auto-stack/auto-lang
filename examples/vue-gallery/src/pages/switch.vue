<script setup lang="ts">
import { ref } from 'vue'
import { Switch } from '@auto-ui/widgets/registry/switch'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const on = ref(true)

const codeControlled = `<script setup lang="ts">
import { ref } from 'vue'
import { Switch } from '@auto-ui/widgets/registry/switch'

const on = ref(true)
<\/script>

<template>
  <Switch v-model="on" />
</template>`

const codeStates = `<script setup lang="ts">
import { Switch } from '@auto-ui/widgets/registry/switch'
<\/script>

<template>
  <Switch :model-value="true" />
  <Switch :model-value="false" />
  <Switch disabled />
</template>`

const rows = [
  { prop: 'modelValue', type: 'boolean', default: '—', desc: 'On/off state (v-model).' },
  { prop: 'disabled', type: 'boolean', default: 'false', desc: 'Disable interaction (reka-ui).' },
  { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
]
</script>

<template>
  <h2 class="page-title">Switch</h2>

  <DemoBlock title="Controlled" :code="codeControlled">
    <Switch v-model="on" />
    <span class="text-sm muted">on: {{ on }}</span>
  </DemoBlock>

  <DemoBlock title="States" :code="codeStates">
    <Switch :model-value="true" />
    <Switch :model-value="false" />
    <Switch disabled />
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
