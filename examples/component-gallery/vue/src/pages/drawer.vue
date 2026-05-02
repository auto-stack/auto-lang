<script setup lang="ts">
import { Drawer, DrawerTrigger } from '@/components/ui/drawer'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const drawerBasicAutoCode = `row (gap: "4") {
    drawer {}
    drawer-trigger (text: "Open Drawer") {}
}
`
const drawerBasicVueCode = `<div class="flex flex-row gap-4">
  <Drawer />
  <DrawerTrigger />
</div>
`
</script>

<template>
  <ComponentDocPage title="Drawer" description="A drawer that slides in from the side of the screen." installCommand="npx shadcn-vue@latest add drawer">
    <DemoSection title="Simple" id="drawer-basic" :autoCode="drawerBasicAutoCode" :vueCode="drawerBasicVueCode">
      <template #preview>
        <div class="flex flex-row gap-4">
              <Drawer />
              <DrawerTrigger />
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
            <TableCell>open</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Controls drawer visibility</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>direction</TableCell>
            <TableCell>string</TableCell>
            <TableCell>right</TableCell>
            <TableCell>left, right, top, bottom</TableCell>
            <TableCell>Drawer slide direction</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>title</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Drawer title</TableCell>
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
