<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Calendar } from '@/components/ui/calendar'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const datepickerBasicAutoCode = `popover {
    popover-trigger {
        button (text: "Pick a date", variant: "outline") {}
    }
    popover-content {
        calendar {}
    }
}
`
const datepickerBasicVueCode = `<Popover>
  <PopoverTrigger>
    <Button variant="outline">Pick a date</Button>
  </PopoverTrigger>
  <PopoverContent>
    <Calendar />
  </PopoverContent>
</Popover>
`
</script>

<template>
  <ComponentDocPage title="DatePicker" description="A date picker component with a calendar popover for selecting dates." installCommand="npx shadcn-vue@latest add date-picker">
    <DemoSection title="Simple" id="datepicker-basic" :autoCode="datepickerBasicAutoCode" :vueCode="datepickerBasicVueCode">
      <template #preview>
        <Popover>
              <PopoverTrigger>
                <Button variant="outline">Pick a date</Button>
              </PopoverTrigger>
              <PopoverContent>
                <Calendar />
              </PopoverContent>
            </Popover>
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
            <TableCell>format</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"PPP"</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Date display format</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>placeholder</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"Pick a date"</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Placeholder text</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the picker</TableCell>
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
