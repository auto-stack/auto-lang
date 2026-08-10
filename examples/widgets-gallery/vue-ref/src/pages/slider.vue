<script setup lang="ts">
import { Slider } from '@/components/ui/slider'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'
import { ref } from 'vue'

const value = ref([50])
const rangeValue = ref([25, 75])

const sliderBasicAutoCode = `slider (min: "0", value: "50", class: "w-60", max: "100") {}
`
const sliderBasicVueCode = `<Slider :min="0" :max="100" class="w-60" />
`

const sliderControlledAutoCode = `slider (min: "0", value: "50", class: "w-60", max: "100") {}
`
const sliderControlledVueCode = `<Slider v-model="value" :min="0" :max="100" class="w-60" />
<p class="text-sm text-muted-foreground">Value: {{ value[0] }}</p>
`

const sliderRangeAutoCode = `slider (min: "0", value: "25,75", class: "w-60", max: "100") {}
`
const sliderRangeVueCode = `<Slider v-model="rangeValue" :min="0" :max="100" class="w-60" />
<p class="text-sm text-muted-foreground">Range: {{ rangeValue[0] }} - {{ rangeValue[1] }}</p>
`

const sliderStepAutoCode = `slider (min: "0", step: "10", value: "50", class: "w-60", max: "100") {}
`
const sliderStepVueCode = `<Slider :min="0" :max="100" :step="10" class="w-60" />
`
</script>

<template>
  <ComponentDocPage title="Slider" description="An input where the user selects a value from within a given range." installCommand="npx shadcn-vue@latest add slider">
    <DemoSection title="Simple" id="slider-basic" :autoCode="sliderBasicAutoCode" :vueCode="sliderBasicVueCode">
      <template #preview>
        <Slider :min="0" :max="100" class="w-60" />
      </template>
    </DemoSection>

    <DemoSection title="Controlled" id="slider-controlled" :autoCode="sliderControlledAutoCode" :vueCode="sliderControlledVueCode">
      <template #preview>
        <div class="flex flex-col gap-2 w-60">
          <Slider v-model="value" :min="0" :max="100" />
          <p class="text-sm text-muted-foreground">Value: {{ value[0] }}</p>
        </div>
      </template>
    </DemoSection>

    <DemoSection title="Range" id="slider-range" :autoCode="sliderRangeAutoCode" :vueCode="sliderRangeVueCode">
      <template #preview>
        <div class="flex flex-col gap-2 w-60">
          <Slider v-model="rangeValue" :min="0" :max="100" />
          <p class="text-sm text-muted-foreground">Range: {{ rangeValue[0] }} - {{ rangeValue[1] }}</p>
        </div>
      </template>
    </DemoSection>

    <DemoSection title="With Step" id="slider-step" :autoCode="sliderStepAutoCode" :vueCode="sliderStepVueCode">
      <template #preview>
        <Slider :min="0" :max="100" :step="10" class="w-60" />
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
          </TableRow>
          <TableRow>
            <TableCell>max</TableCell>
            <TableCell>number</TableCell>
            <TableCell>100</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Maximum value</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>step</TableCell>
            <TableCell>number</TableCell>
            <TableCell>1</TableCell>
            <TableCell>-</TableCell>
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
