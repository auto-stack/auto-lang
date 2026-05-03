<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const inputBasicAutoCode = `input (placeholder: "Email", type: "email") {}
`
const inputBasicVueCode = `<Input type="email" placeholder="Email" />
`
const inputFileAutoCode = `col (gap: "1.5", class: "w-full max-w-sm") {
    label (text: "Picture", for: "picture") {}
    input (id: "picture", type: "file") {}
}
`
const inputFileVueCode = `<div class="flex flex-col gap-1.5 w-full max-w-sm">
  <Label for="picture">Picture</Label>
  <Input type="file" />
</div>
`
const inputDisabledAutoCode = `input (type: "email", disabled: "true", placeholder: "Email") {}
`
const inputDisabledVueCode = `<Input type="email" placeholder="Email" disabled />
`
const inputLabelAutoCode = `col (class: "w-full max-w-sm", gap: "1.5") {
    label (for: "email", text: "Email") {}
    input (placeholder: "Email", id: "email", type: "email") {}
}
`
const inputLabelVueCode = `<div class="flex flex-col gap-1.5 w-full max-w-sm">
  <Label for="email">Email</Label>
  <Input type="email" placeholder="Email" />
</div>
`
const inputButtonAutoCode = `row (gap: "2", class: "w-full max-w-sm items-center") {
    input (placeholder: "Email", type: "email") {}
    button (text: "Subscribe", type: "submit") {}
}
`
const inputButtonVueCode = `<div class="flex flex-row gap-2 w-full max-w-sm items-center">
  <Input type="email" placeholder="Email" />
  <Button>Subscribe</Button>
</div>
`
</script>

<template>
  <ComponentDocPage title="Input" description="Displays a form input field or a component that looks like an input field." installCommand="npx shadcn-vue@latest add input">
    <DemoSection title="Simple" id="input-basic" :autoCode="inputBasicAutoCode" :vueCode="inputBasicVueCode">
      <template #preview>
        <Input type="email" placeholder="Email" />
      </template>
    </DemoSection>
    <DemoSection title="File" id="input-file" :autoCode="inputFileAutoCode" :vueCode="inputFileVueCode">
      <template #preview>
        <div class="flex flex-col gap-1.5 w-full max-w-sm">
          <Label for="picture">Picture</Label>
          <Input type="file" />
        </div>
      </template>
    </DemoSection>
    <DemoSection title="Disabled" id="input-disabled" :autoCode="inputDisabledAutoCode" :vueCode="inputDisabledVueCode">
      <template #preview>
        <Input type="email" placeholder="Email" disabled />
      </template>
    </DemoSection>
    <DemoSection title="With Label" id="input-label" :autoCode="inputLabelAutoCode" :vueCode="inputLabelVueCode">
      <template #preview>
        <div class="flex flex-col gap-1.5 w-full max-w-sm">
          <Label for="email">Email</Label>
          <Input type="email" placeholder="Email" />
        </div>
      </template>
    </DemoSection>
    <DemoSection title="With Button" id="input-button" :autoCode="inputButtonAutoCode" :vueCode="inputButtonVueCode">
      <template #preview>
        <div class="flex flex-row gap-2 w-full max-w-sm items-center">
          <Input type="email" placeholder="Email" />
          <Button>Subscribe</Button>
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
            <TableCell>type</TableCell>
            <TableCell>string</TableCell>
            <TableCell>text</TableCell>
            <TableCell>text, email, password, file, number, etc.</TableCell>
            <TableCell>Input type attribute</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>placeholder</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Placeholder text shown when input is empty</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the input</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>id</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Unique identifier for associating with labels</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>modelValue</TableCell>
            <TableCell>string | number</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>The value of the input (v-model binding)</TableCell>
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
