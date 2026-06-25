<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const popoverBasicAutoCode = `popover {
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
`
const popoverBasicVueCode = `<Popover>
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
`

const popoverWithFormAutoCode = `popover {
    popover-trigger (as: "button") {
        button (variant: "outline", text: "Open Popover") {}
    }
    popover-content (class: "w-80") {
        col (gap: "4") {
            col (gap: "2") {
                label (text: "Width") {}
                input (value: "100%", id: "width") {}
            }
            col (gap: "2") {
                label (text: "Height") {}
                input (value: "100%", id: "height") {}
            }
        }
    }
}
`
const popoverWithFormVueCode = `<Popover>
  <PopoverTrigger as-child>
    <Button variant="outline">Open Popover</Button>
  </PopoverTrigger>
  <PopoverContent class="w-80">
    <div class="flex flex-col gap-4">
      <div class="flex flex-col gap-2">
        <Label for="width">Width</Label>
        <Input id="width" default-value="100%" />
      </div>
      <div class="flex flex-col gap-2">
        <Label for="height">Height</Label>
        <Input id="height" default-value="100%" />
      </div>
    </div>
  </PopoverContent>
</Popover>
`

const popoverAlignmentAutoCode = `row (gap: "4") {
    popover {
        popover-trigger {
            button (variant: "outline", text: "Align Start") {}
        }
        popover-content (align: "start") {
            "Aligned to start."
        }
    }
    popover {
        popover-trigger {
            button (variant: "outline", text: "Align Center") {}
        }
        popover-content (align: "center") {
            "Aligned to center."
        }
    }
    popover {
        popover-trigger {
            button (variant: "outline", text: "Align End") {}
        }
        popover-content (align: "end") {
            "Aligned to end."
        }
    }
}
`
const popoverAlignmentVueCode = `<div class="flex flex-row gap-4">
  <Popover>
    <PopoverTrigger>
      <Button variant="outline">Align Start</Button>
    </PopoverTrigger>
    <PopoverContent align="start">
      Aligned to start.
    </PopoverContent>
  </Popover>
  <Popover>
    <PopoverTrigger>
      <Button variant="outline">Align Center</Button>
    </PopoverTrigger>
    <PopoverContent align="center">
      Aligned to center.
    </PopoverContent>
  </Popover>
  <Popover>
    <PopoverTrigger>
      <Button variant="outline">Align End</Button>
    </PopoverTrigger>
    <PopoverContent align="end">
      Aligned to end.
    </PopoverContent>
  </Popover>
</div>
`
</script>

<template>
  <ComponentDocPage title="Popover" description="A floating content that appears near a trigger element." installCommand="npx shadcn-vue@latest add popover">
    <DemoSection title="Simple" id="popover-basic" :autoCode="popoverBasicAutoCode" :vueCode="popoverBasicVueCode">
      <template #preview>
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
      </template>
    </DemoSection>

    <DemoSection title="With Form" id="popover-form" :autoCode="popoverWithFormAutoCode" :vueCode="popoverWithFormVueCode">
      <template #preview>
        <Popover>
          <PopoverTrigger as-child>
            <Button variant="outline">Open Popover</Button>
          </PopoverTrigger>
          <PopoverContent class="w-80">
            <div class="flex flex-col gap-4">
              <div class="flex flex-col gap-2">
                <Label for="width">Width</Label>
                <Input id="width" default-value="100%" />
              </div>
              <div class="flex flex-col gap-2">
                <Label for="height">Height</Label>
                <Input id="height" default-value="100%" />
              </div>
            </div>
          </PopoverContent>
        </Popover>
      </template>
    </DemoSection>

    <DemoSection title="Alignment" id="popover-alignment" :autoCode="popoverAlignmentAutoCode" :vueCode="popoverAlignmentVueCode">
      <template #preview>
        <div class="flex flex-row gap-4">
          <Popover>
            <PopoverTrigger>
              <Button variant="outline">Align Start</Button>
            </PopoverTrigger>
            <PopoverContent align="start">
              Aligned to start.
            </PopoverContent>
          </Popover>
          <Popover>
            <PopoverTrigger>
              <Button variant="outline">Align Center</Button>
            </PopoverTrigger>
            <PopoverContent align="center">
              Aligned to center.
            </PopoverContent>
          </Popover>
          <Popover>
            <PopoverTrigger>
              <Button variant="outline">Align End</Button>
            </PopoverTrigger>
            <PopoverContent align="end">
              Aligned to end.
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
