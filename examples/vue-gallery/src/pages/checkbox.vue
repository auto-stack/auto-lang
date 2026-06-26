<script setup lang="ts">
import { ref } from 'vue'
import { Checkbox } from '@auto-ui/widgets/registry/checkbox'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const checked = ref(true)

const codeControlled = `<script setup lang="ts">
import { ref } from 'vue'
import { Checkbox } from '@auto-ui/widgets/registry/checkbox'

const checked = ref(true)
<\/script>

<template>
  <Checkbox v-model="checked" />
</template>`

const codeStates = `<script setup lang="ts">
import { Checkbox } from '@auto-ui/widgets/registry/checkbox'
<\/script>

<template>
  <Checkbox :model-value="true" />
  <Checkbox :model-value="false" />
  <Checkbox disabled />
</template>`

const rows = [
  { prop: 'modelValue', type: 'boolean', default: '—', desc: 'Checked state (v-model).' },
  { prop: 'disabled', type: 'boolean', default: 'false', desc: 'Disable interaction (reka-ui).' },
  { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
]
</script>

<template>
  <h2 class="page-title">Checkbox</h2>

  <DemoBlock title="Controlled" :code="codeControlled">
    <Checkbox v-model="checked" />
    <span class="text-sm muted">checked: {{ checked }}</span>
  </DemoBlock>

  <DemoBlock title="States" :code="codeStates">
    <Checkbox :model-value="true" />
    <Checkbox :model-value="false" />
    <Checkbox disabled />
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
