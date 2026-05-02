<script setup lang="ts">
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const labelBasicAutoCode = `label (text: "Email") {}
`
const labelBasicVueCode = `<Label>Email</Label>
`
const labelInputAutoCode = `col (gap: "1.5", class: "w-64") {
    label (text: "Email", for: "email") {}
    input (placeholder: "Enter your email", id: "email") {}
}
`
const labelInputVueCode = `<div class="flex flex-col gap-1.5 w-64">
  <Label for="email">Email</Label>
  <Input placeholder="Enter your email" />
</div>
`
</script>

<template>
  <ComponentDocPage title="Label" description="Renders an accessible label associated with controls." installCommand="npx shadcn-vue@latest add label">
    <DemoSection title="Simple" id="label-basic" :autoCode="labelBasicAutoCode" :vueCode="labelBasicVueCode">
      <template #preview>
        <Label>Email</Label>
      </template>
    </DemoSection>
    <DemoSection title="With Input" id="label-input" :autoCode="labelInputAutoCode" :vueCode="labelInputVueCode">
      <template #preview>
        <div class="flex flex-col gap-1.5 w-64">
              <Label for="email">Email</Label>
              <Input placeholder="Enter your email" />
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
            <TableCell>text</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Label text content (slot)</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>for</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>ID of the associated form control</TableCell>
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
