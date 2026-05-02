<script setup lang="ts">
import { Calendar } from '@/components/ui/calendar'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const calendarBasicAutoCode = `calendar {}
`
const calendarBasicVueCode = `<Calendar />
`
</script>

<template>
  <ComponentDocPage title="Calendar" description="A date field component that allows users to select a date or date range." installCommand="npx shadcn-vue@latest add calendar">
    <DemoSection title="Simple" id="calendar-basic" :autoCode="calendarBasicAutoCode" :vueCode="calendarBasicVueCode">
      <template #preview>
        <Calendar />
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
            <TableCell>v-model</TableCell>
            <TableCell>Date</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Selected date value</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>locale</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"en"</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Locale for formatting</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the calendar</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>mode</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"single"</TableCell>
            <TableCell>"single", "range"</TableCell>
            <TableCell>Selection mode</TableCell>
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
