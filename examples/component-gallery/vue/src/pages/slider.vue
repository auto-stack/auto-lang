<script setup lang="ts">
import { Slider } from '@/components/ui/slider'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const sliderBasicAutoCode = `slider (min: "0", value: "50", class: "w-60", max: "100") {}
`
const sliderBasicVueCode = `<Slider :min="0" :max="100" />
`
</script>

<template>
  <ComponentDocPage title="Slider" description="An input where the user selects a value from within a given range." installCommand="npx shadcn-vue@latest add slider">
    <DemoSection title="Simple" id="slider-basic" :autoCode="sliderBasicAutoCode" :vueCode="sliderBasicVueCode">
      <template #preview>
        <Slider :min="0" :max="100" />
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
            <TableCell>min</TableCell>
            <TableCell>number</TableCell>
            <TableCell>0</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Minimum value</TableCell>
            <TableCell>100</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Maximum value</TableCell>
            <TableCell>1</TableCell>
            <TableCell>Step increment</TableCell>
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
