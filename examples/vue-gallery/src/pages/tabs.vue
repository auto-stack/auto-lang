<script setup lang="ts">
import { ref } from 'vue'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@auto-ui/widgets/registry/tabs'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const active = ref('account')

const codeControlled = `<script setup lang="ts">
import { ref } from 'vue'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@auto-ui/widgets/registry/tabs'

const active = ref('account')
<\/script>

<template>
  <Tabs v-model="active">
    <TabsList>
      <TabsTrigger value="account">Account</TabsTrigger>
      <TabsTrigger value="password">Password</TabsTrigger>
    </TabsList>
    <TabsContent value="account">Account settings.</TabsContent>
    <TabsContent value="password">Password settings.</TabsContent>
  </Tabs>
</template>`

const rows = [
  { prop: '(Tabs) modelValue', type: 'string | number', default: '—', desc: 'Active tab value (v-model, reka-ui TabsRootProps).' },
  { prop: '(TabsList)', type: 'TabsListProps', default: '—', desc: 'Tab trigger container.' },
  { prop: '(TabsTrigger) value', type: 'string | number', default: '—', desc: 'Value this tab activates.' },
  { prop: '(TabsContent) value', type: 'string | number', default: '—', desc: 'Shown when its value is active.' },
]
</script>

<template>
  <h2 class="page-title">Tabs</h2>

  <DemoBlock title="Controlled" :code="codeControlled">
    <Tabs v-model="active" class="half">
      <TabsList>
        <TabsTrigger value="account">Account</TabsTrigger>
        <TabsTrigger value="password">Password</TabsTrigger>
      </TabsList>
      <TabsContent value="account" class="text-sm mt-2">Account settings.</TabsContent>
      <TabsContent value="password" class="text-sm mt-2">Password settings.</TabsContent>
    </Tabs>
    <span class="text-sm muted">active: {{ active }}</span>
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
