<script setup lang="ts">
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const skeletonBasicAutoCode = `skeleton (class: "w-48 h-4") {}
`
const skeletonBasicVueCode = `<div class="w-48 h-4" />
`
const skeletonCardAutoCode = `col (class: "w-64", gap: "4") {
    skeleton (class: "h-12 w-12 rounded-full") {}
    col (gap: "2") {
        skeleton (class: "h-4 w-full") {}
        skeleton (class: "h-4 w-3/4") {}
    }
}
`
const skeletonCardVueCode = `<div class="flex flex-col gap-4 w-64">
  <div class="h-12 w-12 rounded-full" />
  <div class="flex flex-col gap-2">
    <div class="h-4 w-full" />
    <div class="h-4 w-3/4" />
  </div>
</div>
`
</script>

<template>
  <ComponentDocPage title="Skeleton" description="Use to show a placeholder while content is loading." installCommand="npx shadcn-vue@latest add skeleton">
    <DemoSection title="Simple" id="skeleton-basic" :autoCode="skeletonBasicAutoCode" :vueCode="skeletonBasicVueCode">
      <template #preview>
        <div class="w-48 h-4" />
      </template>
    </DemoSection>
    <DemoSection title="Card Skeleton" id="skeleton-card" :autoCode="skeletonCardAutoCode" :vueCode="skeletonCardVueCode">
      <template #preview>
        <div class="flex flex-col gap-4 w-64">
              <div class="h-12 w-12 rounded-full" />
              <div class="flex flex-col gap-2">
                <div class="h-4 w-full" />
                <div class="h-4 w-3/4" />
              </div>
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
            <TableCell>class</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Tailwind classes for sizing</TableCell>
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
