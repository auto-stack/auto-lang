<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const sheetBasicAutoCode = `row (gap: "4") {
    sheet (side: "right") {
        sheet-trigger {
            button (text: "Open", variant: "outline") {}
        }
        sheet-content {
            sheet-header {
                sheet-title (text: "Edit Profile") {}
                sheet-description (text: "Make changes to your profile here.") {}
            }
            col (gap: "4") {
                "Profile content goes here."
            }
        }
    }
}
`
const sheetBasicVueCode = `<div class="flex flex-row gap-4">
  <Sheet>
    <SheetTrigger>
      <Button variant="outline">Open</Button>
    </SheetTrigger>
    <SheetContent>
      <SheetHeader>
        <SheetTitle>Edit Profile</SheetTitle>
        <div>Make changes to your profile here.</div>
      </SheetHeader>
      <div class="flex flex-col gap-4">
        Profile content goes here.
      </div>
    </SheetContent>
  </Sheet>
</div>
`
</script>

<template>
  <ComponentDocPage title="Sheet" description="A panel that slides in from the side of the screen." installCommand="npx shadcn-vue@latest add sheet">
    <DemoSection title="Simple" id="sheet-basic" :autoCode="sheetBasicAutoCode" :vueCode="sheetBasicVueCode">
      <template #preview>
        <div class="flex flex-row gap-4">
              <Sheet>
                <SheetTrigger>
                  <Button variant="outline">Open</Button>
                </SheetTrigger>
                <SheetContent>
                  <SheetHeader>
                    <SheetTitle>Edit Profile</SheetTitle>
                    <div>Make changes to your profile here.</div>
                  </SheetHeader>
                  <div class="flex flex-col gap-4">
                    Profile content goes here.
                  </div>
                </SheetContent>
              </Sheet>
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
            <TableCell>side</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"right"</TableCell>
            <TableCell>"top", "right", "bottom", "left"</TableCell>
            <TableCell>Which side the sheet slides from</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>open</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Controls the open state</TableCell>
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
