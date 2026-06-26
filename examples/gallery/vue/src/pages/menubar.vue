<script setup lang="ts">
import { Menubar } from '@/components/ui/menubar'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const menubarBasicAutoCode = `row (gap: "4") {
    menubar {}
}
`
const menubarBasicVueCode = `<div class="flex flex-row gap-4">
  <Menubar />
</div>
`
</script>

<template>
  <ComponentDocPage title="Menubar" description="A horizontal menu bar with dropdown menus for navigation." installCommand="npx shadcn-vue@latest add menubar">
    <DemoSection title="Simple" id="menubar-basic" :autoCode="menubarBasicAutoCode" :vueCode="menubarBasicVueCode">
      <template #preview>
        <div class="flex flex-row gap-4">
              <Menubar />
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
            <TableCell>Menu label text</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the menu item</TableCell>
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
