<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Textarea } from '@/components/ui/textarea'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const textareaBasicAutoCode = `textarea (placeholder: "Type your message here.", class: "w-64") {}
`
const textareaBasicVueCode = `<Textarea placeholder="Type your message here." />
`

const textareaDisabledAutoCode = `textarea (class: "w-64", disabled: "true", placeholder: "Disabled textarea") {}
`
const textareaDisabledVueCode = `<Textarea placeholder="Disabled textarea" disabled />
`

const textareaLabelAutoCode = `col (gap: "1.5", class: "w-64") {
    label (for: "message", text: "Your message") {}
    textarea (id: "message", placeholder: "Type your message here.") {}
}
`
const textareaLabelVueCode = `<div class="flex flex-col gap-1.5 w-64">
  <Label for="message">Your message</Label>
  <Textarea id="message" placeholder="Type your message here." />
</div>
`

const textareaWithButtonAutoCode = `col (gap: "2", class: "w-64") {
    textarea (placeholder: "Type your message here.") {}
    button (text: "Send Message") {}
}
`
const textareaWithButtonVueCode = `<div class="flex flex-col gap-2 w-64">
  <Textarea placeholder="Type your message here." />
  <Button>Send Message</Button>
</div>
`
</script>

<template>
  <ComponentDocPage title="Textarea" description="Displays a form textarea or a component that looks like a textarea." installCommand="npx shadcn-vue@latest add textarea">
    <DemoSection title="Simple" id="textarea-basic" :autoCode="textareaBasicAutoCode" :vueCode="textareaBasicVueCode">
      <template #preview>
        <Textarea placeholder="Type your message here." />
      </template>
    </DemoSection>
    <DemoSection title="Disabled" id="textarea-disabled" :autoCode="textareaDisabledAutoCode" :vueCode="textareaDisabledVueCode">
      <template #preview>
        <Textarea placeholder="Disabled textarea" disabled />
      </template>
    </DemoSection>
    <DemoSection title="With Label" id="textarea-label" :autoCode="textareaLabelAutoCode" :vueCode="textareaLabelVueCode">
      <template #preview>
        <div class="flex flex-col gap-1.5 w-64">
          <Label for="message">Your message</Label>
          <Textarea id="message" placeholder="Type your message here." />
        </div>
      </template>
    </DemoSection>
    <DemoSection title="With Button" id="textarea-button" :autoCode="textareaWithButtonAutoCode" :vueCode="textareaWithButtonVueCode">
      <template #preview>
        <div class="flex flex-col gap-2 w-64">
          <Textarea placeholder="Type your message here." />
          <Button>Send Message</Button>
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
            <TableCell>placeholder</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Placeholder text</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the textarea</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>id</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>ID for label association</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>rows</TableCell>
            <TableCell>number</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Number of visible text lines</TableCell>
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
