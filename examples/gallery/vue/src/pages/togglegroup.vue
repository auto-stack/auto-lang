<script setup lang="ts">
import { Bold, Italic, Underline } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const togglegroupBasicAutoCode = `row (gap: "4") {
    togglegroup (variant: "outline", type: "multiple") {
        togglegroup-item (value: "bold", aria-label: "Toggle bold") {
            "B"
        }
        togglegroup-item (aria-label: "Toggle italic", value: "italic") {
            "I"
        }
        togglegroup-item (value: "underline", aria-label: "Toggle underline") {
            "U"
        }
    }
}
`
const togglegroupBasicVueCode = `<div class="flex gap-2">
  <Button variant="outline" size="icon" aria-label="Toggle bold">
    <Bold class="h-4 w-4" />
  </Button>
  <Button variant="outline" size="icon" aria-label="Toggle italic">
    <Italic class="h-4 w-4" />
  </Button>
  <Button variant="outline" size="icon" aria-label="Toggle underline">
    <Underline class="h-4 w-4" />
  </Button>
</div>
`
</script>

<template>
  <ComponentDocPage title="ToggleGroup" description="A group of toggle buttons that can be configured for single or multiple selection." installCommand="npx shadcn-vue@latest add toggle-group">
    <DemoSection title="Simple" id="togglegroup-basic" :autoCode="togglegroupBasicAutoCode" :vueCode="togglegroupBasicVueCode">
      <template #preview>
        <div class="flex gap-2">
          <Button variant="outline" size="icon" aria-label="Toggle bold">
            <Bold class="h-4 w-4" />
          </Button>
          <Button variant="outline" size="icon" aria-label="Toggle italic">
            <Italic class="h-4 w-4" />
          </Button>
          <Button variant="outline" size="icon" aria-label="Toggle underline">
            <Underline class="h-4 w-4" />
          </Button>
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
            <TableCell>type</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"single"</TableCell>
            <TableCell>"single", "multiple"</TableCell>
            <TableCell>Selection type</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>variant</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"default"</TableCell>
            <TableCell>"default", "outline"</TableCell>
            <TableCell>Toggle style variant</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>size</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"default"</TableCell>
            <TableCell>"default", "sm", "lg"</TableCell>
            <TableCell>Toggle size</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the group</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </template>
  </ComponentDocPage>
</template>

<style scoped>
pre[class*="language-"] {
  margin: 0;
}
</style>
