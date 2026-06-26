<script setup lang="ts">
import { ref } from 'vue'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@auto-ui/widgets/registry/tabs'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const active = ref('account')
</script>

<template>
  <h2 class="text-2xl font-bold tracking-tight mb-6">Tabs</h2>

  <DemoBlock title="Controlled">
    <Tabs v-model="active" class="w-full max-w-sm">
      <TabsList>
        <TabsTrigger value="account">Account</TabsTrigger>
        <TabsTrigger value="password">Password</TabsTrigger>
      </TabsList>
      <TabsContent value="account" class="text-sm mt-2">Account settings.</TabsContent>
      <TabsContent value="password" class="text-sm mt-2">Password settings.</TabsContent>
    </Tabs>
    <span class="text-xs text-muted-foreground">active: {{ active }}</span>
  </DemoBlock>

  <PropTable :rows="[
    { prop: '(Tabs) modelValue', type: 'string | number', default: '—', desc: 'Active tab value (v-model, reka-ui TabsRootProps).' },
    { prop: '(TabsList)', type: 'TabsListProps', default: '—', desc: 'Tab trigger container.' },
    { prop: '(TabsTrigger) value', type: 'string | number', default: '—', desc: 'Value this tab activates.' },
    { prop: '(TabsContent) value', type: 'string | number', default: '—', desc: 'Shown when its value is active.' },
  ]" />
</template>
