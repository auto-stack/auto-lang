<script setup lang="ts">
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const checkboxBasicAutoCode = `row (gap: "2", class: "items-center") {
    checkbox (id: "terms1") {}
    label (text: "Accept terms and conditions", for: "terms1") {}
}
`
const checkboxBasicVueCode = `<div class="flex flex-row gap-2 items-center">
  <Checkbox />
  <Label for="terms1">Accept terms and conditions</Label>
</div>
`
const checkboxDisabledAutoCode = `row (class: "items-center", gap: "2") {
    checkbox (disabled: "true", id: "terms2") {}
    label (for: "terms2", text: "Accept terms and conditions") {}
}
`
const checkboxDisabledVueCode = `<div class="flex flex-row gap-2 items-center">
  <Checkbox disabled />
  <Label for="terms2">Accept terms and conditions</Label>
</div>
`
const checkboxCheckedAutoCode = `row (gap: "2", class: "items-center") {
    checkbox (id: "terms3", checked: "true") {}
    label (text: "Accept terms and conditions", for: "terms3") {}
}
`
const checkboxCheckedVueCode = `<div class="flex flex-row gap-2 items-center">
  <Checkbox :default-checked="true" />
  <Label for="terms3">Accept terms and conditions</Label>
</div>
`
</script>

<template>
  <ComponentDocPage title="Checkbox" description="A control that allows the user to toggle between checked and not checked." installCommand="npx shadcn-vue@latest add checkbox">
    <DemoSection title="Simple" id="checkbox-basic" :autoCode="checkboxBasicAutoCode" :vueCode="checkboxBasicVueCode">
      <template #preview>
        <div class="flex flex-row gap-2 items-center">
              <Checkbox />
              <Label for="terms1">Accept terms and conditions</Label>
            </div>
      </template>
    </DemoSection>
    <DemoSection title="Disabled" id="checkbox-disabled" :autoCode="checkboxDisabledAutoCode" :vueCode="checkboxDisabledVueCode">
      <template #preview>
        <div class="flex flex-row gap-2 items-center">
              <Checkbox disabled />
              <Label for="terms2">Accept terms and conditions</Label>
            </div>
      </template>
    </DemoSection>
    <DemoSection title="Checked" id="checkbox-checked" :autoCode="checkboxCheckedAutoCode" :vueCode="checkboxCheckedVueCode">
      <template #preview>
        <div class="flex flex-row gap-2 items-center">
              <Checkbox :default-checked="true" />
              <Label for="terms3">Accept terms and conditions</Label>
            </div>
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
            <TableCell>Checkbox state</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the checkbox</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>id</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>ID for label association</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </template>
  </ComponentDocPage>
</template>

<style scoped>
/* Override Prism.js default styles */
pre[class*="language-"] {
  margin: 0;
}

/* Component styles */

</style>
