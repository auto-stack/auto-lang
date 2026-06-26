<script setup lang="ts">
import { ref } from 'vue'
import { Textarea } from '@auto-ui/widgets/registry/textarea'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const value = ref('Multi-line\ncontent')

const codeControlled = `<script setup lang="ts">
import { ref } from 'vue'
import { Textarea } from '@auto-ui/widgets/registry/textarea'

const value = ref('Multi-line\\ncontent')
<\/script>

<template>
  <Textarea v-model="value" />
</template>`

const codePlaceholder = `<script setup lang="ts">
import { Textarea } from '@auto-ui/widgets/registry/textarea'
<\/script>

<template>
  <Textarea placeholder="Write something…" />
</template>`

const codeStates = `<script setup lang="ts">
import { Textarea } from '@auto-ui/widgets/registry/textarea'
<\/script>

<template>
  <Textarea disabled placeholder="disabled" />
</template>`

const rows = [
  { prop: 'modelValue', type: 'string | number', default: '—', desc: 'Bound value (v-model).' },
  { prop: 'defaultValue', type: 'string | number', default: '—', desc: 'Uncontrolled initial value.' },
  { prop: 'class', type: 'HTMLAttributes[class]', default: '—', desc: 'Extra classes merged via cn().' },
]
</script>

<template>
  <h2 class="page-title">Textarea</h2>

  <DemoBlock title="Default (controlled)" :code="codeControlled">
    <Textarea v-model="value" class="half" />
    <pre class="text-sm muted whitespace-pre">{{ value }}</pre>
  </DemoBlock>

  <DemoBlock title="Placeholder" :code="codePlaceholder">
    <Textarea placeholder="Write something…" class="half" />
  </DemoBlock>

  <DemoBlock title="States" :code="codeStates">
    <Textarea disabled placeholder="disabled" class="half" />
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
