<script setup lang="ts">
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const scrollVerticalAutoCode = `scroll (class: "h-72 w-48 rounded-md border") {
    col (gap: "2") {
        "Scroll Area"
        separator {}
        "Item 1"
        "Item 2"
        "Item 3"
        "Item 4"
        "Item 5"
        "Item 6"
        "Item 7"
        "Item 8"
        "Item 9"
        "Item 10"
        "Item 11"
        "Item 12"
    }
}
`
const scrollVerticalVueCode = `<ScrollArea class="h-72 w-48 rounded-md border">
  <div class="flex flex-col gap-2">
    Scroll Area
    <Separator />
    Item 1
    Item 2
    Item 3
    Item 4
    Item 5
    Item 6
    Item 7
    Item 8
    Item 9
    Item 10
    Item 11
    Item 12
  </div>
</ScrollArea>
`
</script>

<template>
  <ComponentDocPage title="ScrollArea" description="A scrollable container with custom styled scrollbars." installCommand="npx shadcn-vue@latest add scroll-area">
    <DemoSection title="Vertical Scroll" id="scroll-vertical" :autoCode="scrollVerticalAutoCode" :vueCode="scrollVerticalVueCode">
      <template #preview>
        <ScrollArea class="h-72 w-48 rounded-md border">
              <div class="flex flex-col gap-2">
                Scroll Area
                <Separator />
                Item 1
                Item 2
                Item 3
                Item 4
                Item 5
                Item 6
                Item 7
                Item 8
                Item 9
                Item 10
                Item 11
                Item 12
              </div>
            </ScrollArea>
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
            <TableCell>class</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Additional CSS classes</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>scrollbarVisibility</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"auto"</TableCell>
            <TableCell>"auto", "always", "scroll", "hover"</TableCell>
            <TableCell>Scrollbar visibility mode</TableCell>
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
