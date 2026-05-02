<script setup lang="ts">
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const navigationmenuBasicAutoCode = `row (gap: "4") {
    navigation-menu {}
}
`
const navigationmenuBasicVueCode = `<div class="flex flex-row gap-4">
  <div />
</div>
`
</script>

<template>
  <ComponentDocPage title="NavigationMenu" description="A collection of navigation links with dropdown support." installCommand="npx shadcn-vue@latest add navigation-menu">
    <DemoSection title="Simple" id="navigationmenu-basic" :autoCode="navigationmenuBasicAutoCode" :vueCode="navigationmenuBasicVueCode">
      <template #preview>
        <div class="flex flex-row gap-4">
              <div />
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
            <TableCell>Link text</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>href</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Link destination</TableCell>
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
