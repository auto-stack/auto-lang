<script setup lang="ts">
import { Label } from '@/components/ui/label'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const radiogroupBasicAutoCode = `radiogroup (default: "option-one") {
    row (gap: "2", class: "items-center") {
        radioitem (id: "option-one", value: "option-one") {}
        label (text: "Option One", for: "option-one") {}
    }
    row (gap: "2", class: "items-center") {
        radioitem (value: "option-two", id: "option-two") {}
        label (text: "Option Two", for: "option-two") {}
    }
}
`
const radiogroupBasicVueCode = `<RadioGroup default-value="option-one">
  <div class="flex items-center space-x-2">
    <RadioGroupItem id="option-one" value="option-one" />
    <Label for="option-one">Option One</Label>
  </div>
  <div class="flex items-center space-x-2">
    <RadioGroupItem id="option-two" value="option-two" />
    <Label for="option-two">Option Two</Label>
  </div>
</RadioGroup>
`
</script>

<template>
  <ComponentDocPage title="RadioGroup" description="A set of checkable buttons, known as radio buttons, where no more than one of the buttons can be checked at a time." installCommand="npx shadcn-vue@latest add radio-group">
    <DemoSection title="Simple" id="radiogroup-basic" :autoCode="radiogroupBasicAutoCode" :vueCode="radiogroupBasicVueCode">
      <template #preview>
        <RadioGroup default-value="option-one">
          <div class="flex items-center space-x-2">
            <RadioGroupItem id="option-one" value="option-one" />
            <Label for="option-one">Option One</Label>
          </div>
          <div class="flex items-center space-x-2">
            <RadioGroupItem id="option-two" value="option-two" />
            <Label for="option-two">Option Two</Label>
          </div>
        </RadioGroup>
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
            <TableCell>value</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Selected radio value</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the group</TableCell>
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
