<script setup lang="ts">
import { Progress } from '@/components/ui/progress'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const progressBasicAutoCode = `progress (value: "33", class: "w-60") {}
`
const progressBasicVueCode = `<Progress :model-value="33" class="w-60" />
`

const progressValuesAutoCode = `col (gap: "4") {
    progress (value: "13", class: "w-60") {}
    progress (value: "45", class: "w-60") {}
    progress (value: "78", class: "w-60") {}
    progress (value: "100", class: "w-60") {}
}
`
const progressValuesVueCode = `<div class="flex flex-col gap-4 w-60">
  <Progress :model-value="13" />
  <Progress :model-value="45" />
  <Progress :model-value="78" />
  <Progress :model-value="100" />
</div>
`

const progressIndeterminateAutoCode = `progress (value: "0", class: "w-60 animate-pulse") {}
`
const progressIndeterminateVueCode = `<Progress :model-value="0" class="w-60 animate-pulse" />
`
</script>

<template>
  <ComponentDocPage title="Progress" description="Displays an indicator showing the completion progress of a task." installCommand="npx shadcn-vue@latest add progress">
    <DemoSection title="Simple" id="progress-basic" :autoCode="progressBasicAutoCode" :vueCode="progressBasicVueCode">
      <template #preview>
        <Progress :model-value="33" class="w-60" />
      </template>
    </DemoSection>

    <DemoSection title="Different Values" id="progress-values" :autoCode="progressValuesAutoCode" :vueCode="progressValuesVueCode">
      <template #preview>
        <div class="flex flex-col gap-4 w-60">
          <Progress :model-value="13" />
          <Progress :model-value="45" />
          <Progress :model-value="78" />
          <Progress :model-value="100" />
        </div>
      </template>
    </DemoSection>

    <DemoSection title="Indeterminate" id="progress-indeterminate" :autoCode="progressIndeterminateAutoCode" :vueCode="progressIndeterminateVueCode">
      <template #preview>
        <Progress :model-value="0" class="w-60 animate-pulse" />
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
