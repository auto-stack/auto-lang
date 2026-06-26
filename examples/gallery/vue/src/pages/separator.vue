<script setup lang="ts">
import { Separator } from '@/components/ui/separator'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const separatorHorizontalAutoCode = `col (gap: "4") {
    "Content above"
    separator (orientation: "horizontal") {}
    "Content below"
}
`
const separatorHorizontalVueCode = `<div class="flex flex-col gap-4">
  Content above
  <Separator orientation="horizontal" />
  Content below
</div>
`
const separatorVerticalAutoCode = `row (gap: "4", class: "h-8 items-center") {
    "Left"
    separator (orientation: "vertical") {}
    "Right"
}
`
const separatorVerticalVueCode = `<div class="flex flex-row gap-4 h-8 items-center">
  Left
  <Separator orientation="vertical" />
  Right
</div>
`
</script>

<template>
  <ComponentDocPage title="Separator" description="Visually or semantically separates content." installCommand="npx shadcn-vue@latest add separator">
    <DemoSection title="Horizontal" id="separator-horizontal" :autoCode="separatorHorizontalAutoCode" :vueCode="separatorHorizontalVueCode">
      <template #preview>
        <div class="flex flex-col gap-4">
              Content above
              <Separator orientation="horizontal" />
              Content below
            </div>
      </template>
    </DemoSection>
    <DemoSection title="Vertical" id="separator-vertical" :autoCode="separatorVerticalAutoCode" :vueCode="separatorVerticalVueCode">
      <template #preview>
        <div class="flex flex-row gap-4 h-8 items-center">
              Left
              <Separator orientation="vertical" />
              Right
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
            <TableCell>orientation</TableCell>
            <TableCell>string</TableCell>
            <TableCell>horizontal</TableCell>
            <TableCell>horizontal, vertical</TableCell>
            <TableCell>Separator orientation</TableCell>
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
