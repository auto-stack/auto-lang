<script setup lang="ts">
import { Badge } from '@/components/ui/badge'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const badgeBasicAutoCode = `badge (text: "Badge") {}
`
const badgeBasicVueCode = `<Badge>Badge</Badge>
`
const badgeVariantsAutoCode = `row (gap: "2") {
    badge (text: "Default") {}
    badge (text: "Secondary", variant: "secondary") {}
    badge (text: "Destructive", variant: "destructive") {}
    badge (text: "Outline", variant: "outline") {}
}
`
const badgeVariantsVueCode = `<div class="flex flex-row gap-2">
  <Badge>Default</Badge>
  <Badge>Secondary</Badge>
  <Badge>Destructive</Badge>
  <Badge>Outline</Badge>
</div>
`
</script>

<template>
  <ComponentDocPage title="Badge" description="Displays a badge or a label that draws attention and labels items." installCommand="npx shadcn-vue@latest add badge">
    <DemoSection title="Simple" id="badge-basic" :autoCode="badgeBasicAutoCode" :vueCode="badgeBasicVueCode">
      <template #preview>
        <Badge>Badge</Badge>
      </template>
    </DemoSection>
    <DemoSection title="Variants" id="badge-variants" :autoCode="badgeVariantsAutoCode" :vueCode="badgeVariantsVueCode">
      <template #preview>
        <div class="flex flex-row gap-2">
              <Badge>Default</Badge>
              <Badge>Secondary</Badge>
              <Badge>Destructive</Badge>
              <Badge>Outline</Badge>
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
            <TableCell>text</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Badge text content (slot)</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>variant</TableCell>
            <TableCell>string</TableCell>
            <TableCell>default</TableCell>
            <TableCell>default, secondary, destructive, outline</TableCell>
            <TableCell>Badge style variant</TableCell>
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
