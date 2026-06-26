<script setup lang="ts">
import { Pagination } from '@/components/ui/pagination'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const paginationBasicAutoCode = `row (class: "items-center", gap: "2") {
    pagination {}
}
`
const paginationBasicVueCode = `<div class="flex flex-row gap-2 items-center">
  <Pagination :total="100" :itemsPerPage="10" />
</div>
`
</script>

<template>
  <ComponentDocPage title="Pagination" description="A navigation component for paginating through content." installCommand="npx shadcn-vue@latest add pagination">
    <DemoSection title="Simple" id="pagination-basic" :autoCode="paginationBasicAutoCode" :vueCode="paginationBasicVueCode">
      <template #preview>
        <div class="flex flex-row gap-2 items-center">
              <Pagination :total="100" :itemsPerPage="10" />
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
            <TableCell>total</TableCell>
            <TableCell>number</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Total number of pages</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>page</TableCell>
            <TableCell>number</TableCell>
            <TableCell>1</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Current page number</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>sibling-count</TableCell>
            <TableCell>number</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Number of visible page buttons</TableCell>
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

