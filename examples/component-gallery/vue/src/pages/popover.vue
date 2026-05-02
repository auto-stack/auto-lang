<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const popoverBasicAutoCode = `row (gap: "4") {
    popover {
        popover-trigger {
            button (variant: "outline", text: "Open Popover") {}
        }
        popover-content {
            col (gap: "4") {
                h4 (class: "font-medium", text: "Dimensions") {}
                "Set the dimensions for the layer."
            }
        }
    }
}
`
const popoverBasicVueCode = `<div class="flex flex-row gap-4">
  <Popover>
    <PopoverTrigger>
      <Button variant="outline">Open Popover</Button>
    </PopoverTrigger>
    <PopoverContent>
      <div class="flex flex-col gap-4">
        <h4 class="font-medium">Dimensions</h4>
        Set the dimensions for the layer.
      </div>
    </PopoverContent>
  </Popover>
</div>
`
</script>

<template>
  <ComponentDocPage title="Popover" description="A floating content that appears near a trigger element." installCommand="npx shadcn-vue@latest add popover">
    <DemoSection title="Simple" id="popover-basic" :autoCode="popoverBasicAutoCode" :vueCode="popoverBasicVueCode">
      <template #preview>
        <div class="flex flex-row gap-4">
              <Popover>
                <PopoverTrigger>
                  <Button variant="outline">Open Popover</Button>
                </PopoverTrigger>
                <PopoverContent>
                  <div class="flex flex-col gap-4">
                    <h4 class="font-medium">Dimensions</h4>
                    Set the dimensions for the layer.
                  </div>
                </PopoverContent>
              </Popover>
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
            <TableCell>side</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"bottom"</TableCell>
            <TableCell>"top", "right", "bottom", "left"</TableCell>
            <TableCell>Preferred side of the trigger</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>align</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"center"</TableCell>
            <TableCell>"start", "center", "end"</TableCell>
            <TableCell>Preferred alignment</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>open</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Controls the open state</TableCell>
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
