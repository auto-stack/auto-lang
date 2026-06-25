<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Toaster } from '@/components/ui/sonner'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'
import { toast } from 'vue-sonner'

const sonnerBasicAutoCode = `col (gap: "4") {
    row (gap: "2") {
        button (variant: "outline", text: "Show Toast") {}
        button (text: "Show Success", variant: "outline") {}
    }
    sonner-position {}
}
`
const sonnerBasicVueCode = `<Button variant="outline" @click="toast('Event has been created')">
  Show Toast
</Button>
<Toaster />
`

const sonnerSuccessAutoCode = `button (variant: "outline", text: "Show Success") {}
`
const sonnerSuccessVueCode = `<Button variant="outline" @click="toast.success('Event created successfully')">
  Show Success
</Button>
<Toaster />
`

const sonnerErrorAutoCode = `button (variant: "outline", text: "Show Error") {}
`
const sonnerErrorVueCode = `<Button variant="outline" @click="toast.error('Failed to create event')">
  Show Error
</Button>
<Toaster />
`

const sonnerDescriptionAutoCode = `button (variant: "outline", text: "With Description") {}
`
const sonnerDescriptionVueCode = `<Button
  variant="outline"
  @click="toast('Event has been created', {
    description: 'Monday, January 3rd at 6:00pm',
  })"
>
  With Description
</Button>
<Toaster />
`
</script>

<template>
  <ComponentDocPage title="Sonner" description="An opinionated toast component built on top of Sonner library." installCommand="npx shadcn-vue@latest add sonner">
    <Toaster />
    <DemoSection title="Simple" id="sonner-basic" :autoCode="sonnerBasicAutoCode" :vueCode="sonnerBasicVueCode">
      <template #preview>
        <Button variant="outline" @click="toast('Event has been created')">
          Show Toast
        </Button>
      </template>
    </DemoSection>

    <DemoSection title="Success" id="sonner-success" :autoCode="sonnerSuccessAutoCode" :vueCode="sonnerSuccessVueCode">
      <template #preview>
        <Button variant="outline" @click="toast.success('Event created successfully')">
          Show Success
        </Button>
      </template>
    </DemoSection>

    <DemoSection title="Error" id="sonner-error" :autoCode="sonnerErrorAutoCode" :vueCode="sonnerErrorVueCode">
      <template #preview>
        <Button variant="outline" @click="toast.error('Failed to create event')">
          Show Error
        </Button>
      </template>
    </DemoSection>

    <DemoSection title="With Description" id="sonner-description" :autoCode="sonnerDescriptionAutoCode" :vueCode="sonnerDescriptionVueCode">
      <template #preview>
        <Button
          variant="outline"
          @click="toast('Event has been created', {
            description: 'Monday, January 3rd at 6:00pm',
          })"
        >
          With Description
        </Button>
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
            <TableCell>position</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"bottom-right"</TableCell>
            <TableCell>"top-left", "top-center", "top-right", "bottom-left", "bottom-center", "bottom-right"</TableCell>
            <TableCell>Toast position</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>richColors</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Enable rich colors</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>expand</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Expand toasts on hover</TableCell>
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
