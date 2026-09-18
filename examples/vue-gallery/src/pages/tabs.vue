<script setup lang="ts">
import { ref } from 'vue'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@auto-ui/widgets/registry/tabs'
import DemoBlock from '../components/DemoBlock.vue'
import PropTable from '../components/PropTable.vue'

const active = ref('account')
const enclosedFlat = ref('account')
const enclosedRounded = ref('account')

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

const codeEnclosedFlat = `<template>
  <!-- PLAN-641 enclosed（IDE 直角观感）：激活 tab 与内容面板连通 -->
  <Tabs v-model="active" variant="enclosed">
    <TabsList>
      <TabsTrigger value="account">Account</TabsTrigger>
      <TabsTrigger value="password">Password</TabsTrigger>
    </TabsList>
    <TabsContent value="account">Account settings.</TabsContent>
    <TabsContent value="password">Password settings.</TabsContent>
  </Tabs>
</template>`

const codeEnclosedRounded = `<template>
  <!-- PLAN-641 enclosed + 顶部圆角（Chrome 观感）：装饰走圆角 token -->
  <Tabs v-model="active" variant="enclosed">
    <TabsList>
      <TabsTrigger value="account" class="rounded-t-lg">Account</TabsTrigger>
      <TabsTrigger value="password" class="rounded-t-lg">Password</TabsTrigger>
    </TabsList>
    <TabsContent value="account">Account settings.</TabsContent>
    <TabsContent value="password">Password settings.</TabsContent>
  </Tabs>
</template>`

const rows = [
  { prop: '(Tabs) modelValue', type: 'string | number', default: '—', desc: 'Active tab value (v-model, reka-ui TabsRootProps).' },
  { prop: '(Tabs) variant', type: "'default' | 'enclosed'", default: "'default'", desc: 'PLAN-641: enclosed = active tab merges with the content panel; inactive tabs are flat cells (IDE look; add rounded-t-* for Chrome look).' },
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

  <DemoBlock title="Enclosed (flat, IDE look)" :code="codeEnclosedFlat">
    <Tabs v-model="enclosedFlat" variant="enclosed" class="half">
      <TabsList>
        <TabsTrigger value="account">Account</TabsTrigger>
        <TabsTrigger value="password">Password</TabsTrigger>
      </TabsList>
      <TabsContent value="account" class="text-sm p-4">Account settings.</TabsContent>
      <TabsContent value="password" class="text-sm p-4">Password settings.</TabsContent>
    </Tabs>
    <span class="text-sm muted">active: {{ enclosedFlat }}</span>
  </DemoBlock>

  <DemoBlock title="Enclosed + rounded top (Chrome look)" :code="codeEnclosedRounded">
    <Tabs v-model="enclosedRounded" variant="enclosed" class="half">
      <TabsList>
        <TabsTrigger value="account" class="rounded-t-lg">Account</TabsTrigger>
        <TabsTrigger value="password" class="rounded-t-lg">Password</TabsTrigger>
      </TabsList>
      <TabsContent value="account" class="text-sm p-4">Account settings.</TabsContent>
      <TabsContent value="password" class="text-sm p-4">Password settings.</TabsContent>
    </Tabs>
    <span class="text-sm muted">active: {{ enclosedRounded }}</span>
  </DemoBlock>

  <PropTable :rows="rows" />
</template>
