<script setup lang="ts">
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const switchBasicAutoCode = `switch {}
`
const switchBasicVueCode = `<Switch />
`
const switchLabelAutoCode = `row (gap: "2", class: "items-center") {
    switch (id: "airplane-mode") {}
    label (text: "Airplane Mode", for: "airplane-mode") {}
}
`
const switchLabelVueCode = `<div class="flex items-center space-x-2">
  <Switch id="airplane-mode" />
  <Label for="airplane-mode">Airplane Mode</Label>
</div>
`
const switchDisabledAutoCode = `switch (disabled: "true") {}
`
const switchDisabledVueCode = `<Switch disabled />
`
</script>

<template>
  <ComponentDocPage title="Switch" description="A control that allows the user to toggle between checked and not checked." installCommand="npx shadcn-vue@latest add switch">
    <DemoSection title="Simple" id="switch-basic" :autoCode="switchBasicAutoCode" :vueCode="switchBasicVueCode">
      <template #preview>
        <Switch />
      </template>
    </DemoSection>
    <DemoSection title="With Label" id="switch-label" :autoCode="switchLabelAutoCode" :vueCode="switchLabelVueCode">
      <template #preview>
        <div class="flex items-center space-x-2">
          <Switch id="airplane-mode" />
          <Label for="airplane-mode">Airplane Mode</Label>
        </div>
      </template>
    </DemoSection>
    <DemoSection title="Disabled" id="switch-disabled" :autoCode="switchDisabledAutoCode" :vueCode="switchDisabledVueCode">
      <template #preview>
        <Switch disabled />
      </template>
    </DemoSection>
    <template #properties>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Property</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Default</TableHead>
            <TableHead>Values</TableHead>
            <TableHead>Description</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>checked</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Switch state</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the switch</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </template>
  </ComponentDocPage>
</template>

<style scoped>
pre[class*="language-"] {
  margin: 0;
}
</style>
