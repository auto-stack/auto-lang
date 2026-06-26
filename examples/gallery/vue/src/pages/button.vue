<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const buttonBasicAutoCode = `button (text: "Button") {}
`
const buttonBasicVueCode = `<Button>Button</Button>
`
const buttonVariantsAutoCode = `row (gap: "2") {
    button (text: "Default") {}
    button (text: "Secondary", variant: "secondary") {}
    button (text: "Destructive", variant: "destructive") {}
    button (text: "Outline", variant: "outline") {}
    button (text: "Ghost", variant: "ghost") {}
    button (text: "Link", variant: "link") {}
}
`
const buttonVariantsVueCode = `<div class="flex flex-row gap-2">
  <Button>Default</Button>
  <Button variant="secondary">Secondary</Button>
  <Button variant="destructive">Destructive</Button>
  <Button variant="outline">Outline</Button>
  <Button variant="ghost">Ghost</Button>
  <Button variant="link">Link</Button>
</div>
`
const buttonSizesAutoCode = `row (gap: "2") {
    button (text: "Small", size: "sm") {}
    button (text: "Default") {}
    button (size: "lg", text: "Large") {}
}
`
const buttonSizesVueCode = `<div class="flex flex-row gap-2">
  <Button size="sm">Small</Button>
  <Button>Default</Button>
  <Button size="lg">Large</Button>
</div>
`
</script>

<template>
  <ComponentDocPage title="Button" description="A clickable button component with multiple variants." installCommand="npx shadcn-vue@latest add button">
    <DemoSection title="Simple" id="button-basic" :autoCode="buttonBasicAutoCode" :vueCode="buttonBasicVueCode">
      <template #preview>
        <Button>Button</Button>
      </template>
    </DemoSection>
    <DemoSection title="Variants" id="button-variants" :autoCode="buttonVariantsAutoCode" :vueCode="buttonVariantsVueCode">
      <template #preview>
        <div class="flex flex-row gap-2">
              <Button>Default</Button>
              <Button variant="secondary">Secondary</Button>
              <Button variant="destructive">Destructive</Button>
              <Button variant="outline">Outline</Button>
              <Button variant="ghost">Ghost</Button>
              <Button variant="link">Link</Button>
            </div>
      </template>
    </DemoSection>
    <DemoSection title="Sizes" id="button-sizes" :autoCode="buttonSizesAutoCode" :vueCode="buttonSizesVueCode">
      <template #preview>
        <div class="flex flex-row gap-2">
              <Button size="sm">Small</Button>
              <Button>Default</Button>
              <Button size="lg">Large</Button>
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
            <TableCell>Button label text (slot content)</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>variant</TableCell>
            <TableCell>string</TableCell>
            <TableCell>default</TableCell>
            <TableCell>default, secondary, destructive, outline, ghost, link</TableCell>
            <TableCell>Visual style variant</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>size</TableCell>
            <TableCell>string</TableCell>
            <TableCell>default</TableCell>
            <TableCell>sm, default, lg, icon</TableCell>
            <TableCell>Button size</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the button</TableCell>
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
