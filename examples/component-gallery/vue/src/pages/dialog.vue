<script setup lang="ts">
import { Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const dialogBasicAutoCode = `row (gap: "4") {
    dialog {}
    dialog-trigger (text: "Open Dialog") {}
}
`
const dialogBasicVueCode = `<Dialog>
  <DialogTrigger>
    <Button variant="outline">Open Dialog</Button>
  </DialogTrigger>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Dialog Title</DialogTitle>
      <DialogDescription>This is a dialog description.</DialogDescription>
    </DialogHeader>
    <DialogFooter>
      <DialogClose>
        <Button variant="outline">Close</Button>
      </DialogClose>
    </DialogFooter>
  </DialogContent>
</Dialog>
`
</script>

<template>
  <ComponentDocPage title="Dialog" description="A modal dialog that displays content on top of the page." installCommand="npx shadcn-vue@latest add dialog">
    <DemoSection title="Simple" id="dialog-basic" :autoCode="dialogBasicAutoCode" :vueCode="dialogBasicVueCode">
      <template #preview>
        <Dialog>
          <DialogTrigger>
            <Button variant="outline">Open Dialog</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Dialog Title</DialogTitle>
              <DialogDescription>This is a dialog description.</DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <DialogClose>
                <Button variant="outline">Close</Button>
              </DialogClose>
            </DialogFooter>
          </DialogContent>
        </Dialog>
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
            <TableCell>Controls dialog visibility</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>title</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Dialog title</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>description</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Dialog description</TableCell>
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
