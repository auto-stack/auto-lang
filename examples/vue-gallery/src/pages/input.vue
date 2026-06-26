<script setup lang="ts">
import { ref } from 'vue'
import { Input } from '@auto-ui/widgets/registry/input'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const value = ref('Hello')

const codeControlled = `<script setup lang="ts">
import { ref } from 'vue'
import { Input } from '@auto-ui/widgets/registry/input'

const value = ref('Hello')
<\/script>

<template>
  <Input v-model="value" />
</template>`

const codePlaceholder = `<script setup lang="ts">
import { Input } from '@auto-ui/widgets/registry/input'
<\/script>

<template>
  <Input placeholder="you@example.com" />
</template>`

const codeStates = `<script setup lang="ts">
import { Input } from '@auto-ui/widgets/registry/input'
<\/script>

<template>
  <Input model-value="readonly-ish" />
  <Input disabled placeholder="disabled" />
</template>`

const rows = [
  { prop: 'modelValue', type: 'string | number', default: '—', desc: 'Bound value (v-model).' },
  { prop: 'defaultValue', type: 'string | number', default: '—', desc: 'Uncontrolled initial value.' },
  { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
]
</script>

<template>
  <h2 class="page-title">Input</h2>

  <DemoBlock title="Default (controlled)" description="Two-way bound via v-model." :code="codeControlled">
    <Input v-model="value" />
    <span class="text-sm muted">value: {{ value }}</span>
  </DemoBlock>

  <DemoBlock title="Placeholder" :code="codePlaceholder">
    <Input placeholder="you@example.com" />
  </DemoBlock>

  <DemoBlock title="States" :code="codeStates">
    <Input model-value="readonly-ish" />
    <Input disabled placeholder="disabled" />
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
