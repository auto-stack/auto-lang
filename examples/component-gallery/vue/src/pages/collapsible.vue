<script setup lang="ts">
import { ref } from 'vue'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const isOpen = ref<boolean>(false)

const collapsibleBasicAutoCode = `collapsible (open: .isOpen) {
    collapsible-trigger (class: "flex items-center justify-between w-full py-4 font-medium [&[data-state=open]>span:last-child]:rotate-180") {
        "Can I use this in my project?"
        text (class: "transition-transform duration-200", text: "▸") {}
    }
    collapsible-content (class: "pb-4") {
        "Yes. Free to use for personal and commercial projects. No attribution required."
    }
}
`
const collapsibleBasicVueCode = `<Collapsible v-model:open="isOpen">
  <CollapsibleTrigger class="flex items-center justify-between w-full py-4 font-medium [&[data-state=open]>span:last-child]:rotate-180">
    Can I use this in my project?
    <span class="text-muted-foreground leading-7 transition-transform duration-200">▸</span>
  </CollapsibleTrigger>
  <CollapsibleContent class="pb-4">
    Yes. Free to use for personal and commercial projects. No attribution required.
  </CollapsibleContent>
</Collapsible>
`
</script>

<template>
  <ComponentDocPage title="Collapsible" description="An interactive component which expands or collapses a content panel." installCommand="npx shadcn-vue@latest add collapsible">
    <DemoSection title="Example" id="collapsible-basic" :autoCode="collapsibleBasicAutoCode" :vueCode="collapsibleBasicVueCode">
      <template #preview>
        <Collapsible v-model:open="isOpen">
              <CollapsibleTrigger class="flex items-center justify-between w-full py-4 font-medium [&[data-state=open]>span:last-child]:rotate-180">
                Can I use this in my project?
                <span class="text-muted-foreground leading-7 transition-transform duration-200">▸</span>
              </CollapsibleTrigger>
              <CollapsibleContent class="pb-4">
                Yes. Free to use for personal and commercial projects. No attribution required.
              </CollapsibleContent>
            </Collapsible>
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
            <TableCell>open</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Controls open state</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the collapsible</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>class</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Additional CSS classes</TableCell>
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
