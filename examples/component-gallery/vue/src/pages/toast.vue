<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Toaster } from '@/components/ui/sonner'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const toastBasicAutoCode = `col (gap: "4") {
    button (text: "Show Toast", variant: "outline") {}
    toast-provider {
        toast {
            toast-title (text: "Scheduled: Catch up") {}
            toast-description (text: "Friday, February 10, 2023 at 5:57 PM") {}
        }
    }
}
`
const toastBasicVueCode = `<div class="flex flex-col gap-4">
  <Button variant="outline">Show Toast</Button>
  <div>
    <Toaster>
      <div>Scheduled: Catch up</div>
      <div>Friday, February 10, 2023 at 5:57 PM</div>
    </Toaster>
  </div>
</div>
`
</script>

<template>
  <ComponentDocPage title="Toast" description="A brief message that appears temporarily and disappears automatically." installCommand="npx shadcn-vue@latest add toast">
    <DemoSection title="Simple" id="toast-basic" :autoCode="toastBasicAutoCode" :vueCode="toastBasicVueCode">
      <template #preview>
        <div class="flex flex-col gap-4">
              <Button variant="outline">Show Toast</Button>
              <div>
                <Toaster>
                  <div>Scheduled: Catch up</div>
                  <div>Friday, February 10, 2023 at 5:57 PM</div>
                </Toaster>
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
            <TableCell>variant</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"default"</TableCell>
            <TableCell>"default", "destructive"</TableCell>
            <TableCell>Toast style variant</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>duration</TableCell>
            <TableCell>number</TableCell>
            <TableCell>5000</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Duration in ms</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>open</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Controls open state</TableCell>
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
