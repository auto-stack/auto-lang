<script setup lang="ts">
import { ScrollArea, ScrollBar } from '@/components/ui/scroll-area'
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
  <div class="flex flex-col gap-2 p-4">
    Scroll Area
    <Separator />
    <div>Item 1</div>
    <div>Item 2</div>
    <div>Item 3</div>
    <div>Item 4</div>
    <div>Item 5</div>
    <div>Item 6</div>
    <div>Item 7</div>
    <div>Item 8</div>
    <div>Item 9</div>
    <div>Item 10</div>
    <div>Item 11</div>
    <div>Item 12</div>
  </div>
</ScrollArea>
`

const scrollHorizontalAutoCode = `scroll (class: "w-96 whitespace-nowrap rounded-md border") {
    row (gap: "4", class: "p-4") {
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
    }
}
`
const scrollHorizontalVueCode = `<ScrollArea class="w-96 whitespace-nowrap rounded-md border">
  <div class="flex flex-row gap-4 p-4">
    <div>Item 1</div>
    <div>Item 2</div>
    <div>Item 3</div>
    <div>Item 4</div>
    <div>Item 5</div>
    <div>Item 6</div>
    <div>Item 7</div>
    <div>Item 8</div>
    <div>Item 9</div>
    <div>Item 10</div>
  </div>
  <ScrollBar orientation="horizontal" />
</ScrollArea>
`
</script>

<template>
  <ComponentDocPage title="ScrollArea" description="A scrollable container with custom styled scrollbars." installCommand="npx shadcn-vue@latest add scroll-area">
    <DemoSection title="Vertical Scroll" id="scroll-vertical" :autoCode="scrollVerticalAutoCode" :vueCode="scrollVerticalVueCode">
      <template #preview>
        <ScrollArea class="h-72 w-48 rounded-md border">
          <div class="flex flex-col gap-2 p-4">
            Scroll Area
            <Separator />
            <div>Item 1</div>
            <div>Item 2</div>
            <div>Item 3</div>
            <div>Item 4</div>
            <div>Item 5</div>
            <div>Item 6</div>
            <div>Item 7</div>
            <div>Item 8</div>
            <div>Item 9</div>
            <div>Item 10</div>
            <div>Item 11</div>
            <div>Item 12</div>
          </div>
        </ScrollArea>
      </template>
    </DemoSection>

    <DemoSection title="Horizontal Scroll" id="scroll-horizontal" :autoCode="scrollHorizontalAutoCode" :vueCode="scrollHorizontalVueCode">
      <template #preview>
        <ScrollArea class="w-96 whitespace-nowrap rounded-md border">
          <div class="flex flex-row gap-4 p-4">
            <div>Item 1</div>
            <div>Item 2</div>
            <div>Item 3</div>
            <div>Item 4</div>
            <div>Item 5</div>
            <div>Item 6</div>
            <div>Item 7</div>
            <div>Item 8</div>
            <div>Item 9</div>
            <div>Item 10</div>
          </div>
          <ScrollBar orientation="horizontal" />
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
