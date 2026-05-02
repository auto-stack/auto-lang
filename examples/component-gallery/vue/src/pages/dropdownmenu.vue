<script setup lang="ts">
import { DropdownMenu } from '@/components/ui/dropdown-menu'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const dropdownmenuBasicAutoCode = `row (gap: "4") {
    dropdown-menu {}
}
`
const dropdownmenuBasicVueCode = `<div class="flex flex-row gap-4">
  <DropdownMenu />
</div>
`
</script>

<template>
  <ComponentDocPage title="DropdownMenu" description="A menu that drops down from a trigger element." installCommand="npx shadcn-vue@latest add dropdown-menu">
    <DemoSection title="Simple" id="dropdownmenu-basic" :autoCode="dropdownmenuBasicAutoCode" :vueCode="dropdownmenuBasicVueCode">
      <template #preview>
        <div class="flex flex-row gap-4">
              <DropdownMenu />
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
            <TableCell>Menu item text</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the menu item</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>shortcut</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Keyboard shortcut hint</TableCell>
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
