<script setup lang="ts">
import { Progress } from '@/components/ui/progress'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const progressBasicAutoCode = `progress (value: "33", class: "w-60") {}
`
const progressBasicVueCode = `<Progress :model-value="33" />
`
</script>

<template>
  <ComponentDocPage title="Progress" description="Displays an indicator showing the completion progress of a task." installCommand="npx shadcn-vue@latest add progress">
    <DemoSection title="Simple" id="progress-basic" :autoCode="progressBasicAutoCode" :vueCode="progressBasicVueCode">
      <template #preview>
        <Progress :model-value="33" />
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
            <TableCell>number</TableCell>
            <TableCell>0</TableCell>
            <TableCell>0-100</TableCell>
            <TableCell>Progress percentage</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>max</TableCell>
            <TableCell>number</TableCell>
            <TableCell>100</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Maximum value</TableCell>
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
