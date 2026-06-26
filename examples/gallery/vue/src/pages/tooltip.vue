<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
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
const tooltipBasicVueCode = `<TooltipProvider>
  <Tooltip>
    <TooltipTrigger>
      <Button variant="outline">Hover me</Button>
    </TooltipTrigger>
    <TooltipContent>
      <p>Add to library</p>
    </TooltipContent>
  </Tooltip>
</TooltipProvider>
`

const tooltipSidesAutoCode = `row (gap: "4") {
    tooltip-provider {
        tooltip {
            tooltip-trigger {
                button (text: "Top", variant: "outline") {}
            }
            tooltip-content (side: "top") { "Tooltip on top" }
        }
        tooltip {
            tooltip-trigger {
                button (text: "Right", variant: "outline") {}
            }
            tooltip-content (side: "right") { "Tooltip on right" }
        }
        tooltip {
            tooltip-trigger {
                button (text: "Bottom", variant: "outline") {}
            }
            tooltip-content (side: "bottom") { "Tooltip on bottom" }
        }
        tooltip {
            tooltip-trigger {
                button (text: "Left", variant: "outline") {}
            }
            tooltip-content (side: "left") { "Tooltip on left" }
        }
    }
}
`
const tooltipSidesVueCode = `<TooltipProvider>
  <div class="flex flex-row gap-4">
    <Tooltip>
      <TooltipTrigger>
        <Button variant="outline">Top</Button>
      </TooltipTrigger>
      <TooltipContent side="top">
        <p>Tooltip on top</p>
      </TooltipContent>
    </Tooltip>
    <Tooltip>
      <TooltipTrigger>
        <Button variant="outline">Right</Button>
      </TooltipTrigger>
      <TooltipContent side="right">
        <p>Tooltip on right</p>
      </TooltipContent>
    </Tooltip>
    <Tooltip>
      <TooltipTrigger>
        <Button variant="outline">Bottom</Button>
      </TooltipTrigger>
      <TooltipContent side="bottom">
        <p>Tooltip on bottom</p>
      </TooltipContent>
    </Tooltip>
    <Tooltip>
      <TooltipTrigger>
        <Button variant="outline">Left</Button>
      </TooltipTrigger>
      <TooltipContent side="left">
        <p>Tooltip on left</p>
      </TooltipContent>
    </Tooltip>
  </div>
</TooltipProvider>
`

const tooltipDelayAutoCode = `row (gap: "4") {
    tooltip-provider (delay-duration: "0") {
        tooltip {
            tooltip-trigger {
                button (text: "Instant", variant: "outline") {}
            }
            tooltip-content {
                "Appears immediately"
            }
        }
    }
}
`
const tooltipDelayVueCode = `<TooltipProvider :delay-duration="0">
  <Tooltip>
    <TooltipTrigger>
      <Button variant="outline">Instant</Button>
    </TooltipTrigger>
    <TooltipContent>
      <p>Appears immediately</p>
    </TooltipContent>
  </Tooltip>
</TooltipProvider>
`
</script>

<template>
  <ComponentDocPage title="Tooltip" description="A popup that displays information related to an element when hovering or focusing." installCommand="npx shadcn-vue@latest add tooltip">
    <DemoSection title="Simple" id="tooltip-basic" :autoCode="tooltipBasicAutoCode" :vueCode="tooltipBasicVueCode">
      <template #preview>
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger>
              <Button variant="outline">Hover me</Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>Add to library</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </template>
    </DemoSection>

    <DemoSection title="Sides" id="tooltip-sides" :autoCode="tooltipSidesAutoCode" :vueCode="tooltipSidesVueCode">
      <template #preview>
        <TooltipProvider>
          <div class="flex flex-row gap-4">
            <Tooltip>
              <TooltipTrigger>
                <Button variant="outline">Top</Button>
              </TooltipTrigger>
              <TooltipContent side="top">
                <p>Tooltip on top</p>
              </TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger>
                <Button variant="outline">Right</Button>
              </TooltipTrigger>
              <TooltipContent side="right">
                <p>Tooltip on right</p>
              </TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger>
                <Button variant="outline">Bottom</Button>
              </TooltipTrigger>
              <TooltipContent side="bottom">
                <p>Tooltip on bottom</p>
              </TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger>
                <Button variant="outline">Left</Button>
              </TooltipTrigger>
              <TooltipContent side="left">
                <p>Tooltip on left</p>
              </TooltipContent>
            </Tooltip>
          </div>
        </TooltipProvider>
      </template>
    </DemoSection>

    <DemoSection title="No Delay" id="tooltip-delay" :autoCode="tooltipDelayAutoCode" :vueCode="tooltipDelayVueCode">
      <template #preview>
        <TooltipProvider :delay-duration="0">
          <Tooltip>
            <TooltipTrigger>
              <Button variant="outline">Instant</Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>Appears immediately</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
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
