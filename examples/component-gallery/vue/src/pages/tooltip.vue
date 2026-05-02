<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Tooltip } from '@/components/ui/tooltip'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const tooltipBasicAutoCode = `row (gap: "4") {
    tooltip-provider {
        tooltip {
            tooltip-trigger {
                button (text: "Hover me", variant: "outline") {}
            }
            tooltip-content {
                "Add to library"
            }
        }
    }
}
`
const tooltipBasicVueCode = `<div class="flex flex-row gap-4">
  <div>
    <Tooltip>
      <div>
        <Button variant="outline">Hover me</Button>
      </div>
      <div>
        Add to library
      </div>
    </Tooltip>
  </div>
</div>
`
</script>

<template>
  <ComponentDocPage title="Tooltip" description="A popup that displays information related to an element when hovering or focusing." installCommand="npx shadcn-vue@latest add tooltip">
    <DemoSection title="Simple" id="tooltip-basic" :autoCode="tooltipBasicAutoCode" :vueCode="tooltipBasicVueCode">
      <template #preview>
        <div class="flex flex-row gap-4">
              <div>
                <Tooltip>
                  <div>
                    <Button variant="outline">Hover me</Button>
                  </div>
                  <div>
                    Add to library
                  </div>
                </Tooltip>
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
            <TableCell>side</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"top"</TableCell>
            <TableCell>"top", "right", "bottom", "left"</TableCell>
            <TableCell>Which side the tooltip appears</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>delay-duration</TableCell>
            <TableCell>number</TableCell>
            <TableCell>700</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Delay before showing (ms)</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the tooltip</TableCell>
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
