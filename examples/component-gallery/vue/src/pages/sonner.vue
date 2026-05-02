<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const sonnerBasicAutoCode = `col (gap: "4") {
    row (gap: "2") {
        button (variant: "outline", text: "Show Toast") {}
        button (text: "Show Success", variant: "outline") {}
    }
    sonner-position {}
}
`
const sonnerBasicVueCode = `<div class="flex flex-col gap-4">
  <div class="flex flex-row gap-2">
    <Button variant="outline">Show Toast</Button>
    <Button variant="outline">Show Success</Button>
  </div>
  <div />
</div>
`
const codeblock2Code = `import { toast } from 'vue-sonner'
toast('Event has been created')
toast.success('Event created successfully')
toast.error('Failed to create event')`
</script>

<template>
  <ComponentDocPage title="Sonner" description="An opinionated toast component built on top of Sonner library." installCommand="npx shadcn-vue@latest add sonner">
    <DemoSection title="Simple" id="sonner-basic" :autoCode="sonnerBasicAutoCode" :vueCode="sonnerBasicVueCode">
      <template #preview>
        <div class="flex flex-col gap-4">
              <div class="flex flex-row gap-2">
                <Button variant="outline">Show Toast</Button>
                <Button variant="outline">Show Success</Button>
              </div>
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
            <TableCell>position</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"bottom-right"</TableCell>
            <TableCell>"top-left", "top-center", "top-right", "bottom-left", "bottom-center", "bottom-right"</TableCell>
            <TableCell>Toast position</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>richColors</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Enable rich colors</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>expand</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Expand toasts on hover</TableCell>
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
