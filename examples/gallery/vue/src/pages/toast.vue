<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Toaster } from '@/components/ui/sonner'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'
import { toast } from 'vue-sonner'

const toastBasicAutoCode = `col (gap: "4") {
    button (text: "Show Toast", variant: "outline") {}
    toast-provider {
        toast {
            toast-title (text: "Scheduled: Catch up") {}
            toast-description (text: "Friday, February 10, 2023 at 5:57 PM") {}
        }
    }
}
`
const toastBasicVueCode = `<Button variant="outline" @click="toast('Your message has been sent.')">
  Show Toast
</Button>
<Toaster />
`

const toastSuccessAutoCode = `col (gap: "4") {
    button (text: "Success", variant: "outline") {}
}
`
const toastSuccessVueCode = `<Button variant="outline" @click="toast.success('Event has been created')">
  Success
</Button>
<Toaster />
`

const toastErrorAutoCode = `col (gap: "4") {
    button (text: "Error", variant: "outline") {}
}
`
const toastErrorVueCode = `<Button variant="outline" @click="toast.error('Event has not been created')">
  Error
</Button>
<Toaster />
`

const toastActionAutoCode = `col (gap: "4") {
    button (text: "With Action", variant: "outline") {}
}
`
const toastActionVueCode = `<Button
  variant="outline"
  @click="handleActionToast"
>
  With Action
</Button>
<Toaster />
`

const toastPromiseAutoCode = `col (gap: "4") {
    button (text: "Promise", variant: "outline") {}
}
`
const toastPromiseVueCode = `<Button
  variant="outline"
  @click="handlePromiseToast"
>
  Promise
</Button>
<Toaster />
`

function handleActionToast() {
  toast('Event has been created', {
    action: {
      label: 'Undo',
      onClick: () => console.log('Undo'),
    },
  })
}

function handlePromiseToast() {
  toast.promise(
    new Promise<void>((resolve) => setTimeout(resolve, 2000)),
    {
      loading: 'Loading...',
      success: 'Success!',
      error: 'Error!',
    },
  )
}
</script>

<template>
  <ComponentDocPage title="Toast" description="A brief message that appears temporarily and disappears automatically." installCommand="npx shadcn-vue@latest add sonner">
    <Toaster />
    <DemoSection title="Simple" id="toast-basic" :autoCode="toastBasicAutoCode" :vueCode="toastBasicVueCode">
      <template #preview>
        <Button variant="outline" @click="toast('Your message has been sent.')">
          Show Toast
        </Button>
      </template>
    </DemoSection>

    <DemoSection title="Success" id="toast-success" :autoCode="toastSuccessAutoCode" :vueCode="toastSuccessVueCode">
      <template #preview>
        <Button variant="outline" @click="toast.success('Event has been created')">
          Success
        </Button>
      </template>
    </DemoSection>

    <DemoSection title="Error" id="toast-error" :autoCode="toastErrorAutoCode" :vueCode="toastErrorVueCode">
      <template #preview>
        <Button variant="outline" @click="toast.error('Event has not been created')">
          Error
        </Button>
      </template>
    </DemoSection>

    <DemoSection title="With Action" id="toast-action" :autoCode="toastActionAutoCode" :vueCode="toastActionVueCode">
      <template #preview>
        <Button variant="outline" @click="handleActionToast">
          With Action
        </Button>
      </template>
    </DemoSection>

    <DemoSection title="Promise" id="toast-promise" :autoCode="toastPromiseAutoCode" :vueCode="toastPromiseVueCode">
      <template #preview>
        <Button variant="outline" @click="handlePromiseToast">
          Promise
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
            <TableCell>variant</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"default"</TableCell>
            <TableCell>"default", "destructive"</TableCell>
            <TableCell>Toast style variant</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>duration</TableCell>
            <TableCell>number</TableCell>
            <TableCell>5000</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Duration in ms</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>open</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Controls open state</TableCell>
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
