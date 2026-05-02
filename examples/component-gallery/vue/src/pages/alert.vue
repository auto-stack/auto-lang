<script setup lang="ts">
import { Alert } from '@/components/ui/alert'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const alertBasicAutoCode = `alert {
    "This is an alert message."
}
`
const alertBasicVueCode = `<Alert>
  This is an alert message.
</Alert>
`
const alertVariantsAutoCode = `col (gap: "4") {
    alert (variant: "default") {
        "Default alert"
    }
    alert (variant: "destructive") {
        "Error alert"
    }
}
`
const alertVariantsVueCode = `<div class="flex flex-col gap-4">
  <Alert variant="default">
    Default alert
  </Alert>
  <Alert variant="destructive">
    Error alert
  </Alert>
</div>
`
</script>

<template>
  <ComponentDocPage title="Alert" description="Displays a callout for user attention." installCommand="npx shadcn-vue@latest add alert">
    <DemoSection title="Simple" id="alert-basic" :autoCode="alertBasicAutoCode" :vueCode="alertBasicVueCode">
      <template #preview>
        <Alert>
              This is an alert message.
            </Alert>
      </template>
    </DemoSection>
    <DemoSection title="Variants" id="alert-variants" :autoCode="alertVariantsAutoCode" :vueCode="alertVariantsVueCode">
      <template #preview>
        <div class="flex flex-col gap-4">
              <Alert variant="default">
                Default alert
              </Alert>
              <Alert variant="destructive">
                Error alert
              </Alert>
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
            <TableCell>default</TableCell>
            <TableCell>default, destructive</TableCell>
            <TableCell>Alert style variant</TableCell>
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
