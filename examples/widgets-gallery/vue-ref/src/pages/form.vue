<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Form, FormDescription, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const formBasicAutoCode = `form {
    col (gap: "4") {
        form-item {
            form-label (text: "Username") {}
            input (placeholder: "Enter username") {}
            form-description (text: "This is your public display name.") {}
            form-message {}
        }
        form-item {
            form-label (text: "Email") {}
            input (type: "email", placeholder: "Enter email") {}
            form-message {}
        }
        button (text: "Submit", type: "submit") {}
    }
}
`
const formBasicVueCode = `<Form>
  <div class="flex flex-col gap-4">
    <FormItem>
      <FormLabel>Username</FormLabel>
      <Input placeholder="Enter username" />
      <FormDescription>This is your public display name.</FormDescription>
      <FormMessage />
    </FormItem>
    <FormItem>
      <FormLabel>Email</FormLabel>
      <Input type="email" placeholder="Enter email" />
      <FormMessage />
    </FormItem>
    <Button>Submit</Button>
  </div>
</Form>
`
</script>

<template>
  <ComponentDocPage title="Form" description="A form component with validation support using VeeValidate and Zod schemas." installCommand="npx shadcn-vue@latest add form">
    <DemoSection title="Simple" id="form-basic" :autoCode="formBasicAutoCode" :vueCode="formBasicVueCode">
      <template #preview>
        <Form>
              <div class="flex flex-col gap-4">
                <FormItem>
                  <FormLabel>Username</FormLabel>
                  <Input placeholder="Enter username" />
                  <FormDescription>This is your public display name.</FormDescription>
                  <FormMessage />
                </FormItem>
                <FormItem>
                  <FormLabel>Email</FormLabel>
                  <Input type="email" placeholder="Enter email" />
                  <FormMessage />
                </FormItem>
                <Button>Submit</Button>
              </div>
            </Form>
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
            <TableCell>schema</TableCell>
            <TableCell>ZodSchema</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Validation schema</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>initial-values</TableCell>
            <TableCell>object</TableCell>
            <TableCell>{}</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Initial form values</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>on-submit</TableCell>
            <TableCell>function</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Submit handler</TableCell>
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
