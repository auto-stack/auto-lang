<script setup lang="ts">
import { Switch } from '@/components/ui/switch'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const toggleBasicAutoCode = `row (gap: "4") {
    toggle (variant: "outline", aria-label: "Toggle bold") {
        "B"
    }
    toggle (variant: "outline", aria-label: "Toggle italic") {
        "I"
    }
    toggle (aria-label: "Toggle underline", variant: "outline") {
        "U"
    }
}
`
const toggleBasicVueCode = `<div class="flex flex-row gap-4">
  <Switch>
    B
  </Switch>
  <Switch>
    I
  </Switch>
  <Switch>
    U
  </Switch>
</div>
`
</script>

<template>
  <ComponentDocPage title="Toggle" description="A two-state button that can be toggled on or off, useful for formatting controls." installCommand="npx shadcn-vue@latest add toggle">
    <DemoSection title="Simple" id="toggle-basic" :autoCode="toggleBasicAutoCode" :vueCode="toggleBasicVueCode">
      <template #preview>
        <div class="flex flex-row gap-4">
              <Switch>
                B
              </Switch>
              <Switch>
                I
              </Switch>
              <Switch>
                U
              </Switch>
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
            <TableCell>pressed</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Pressed state</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the toggle</TableCell>
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
