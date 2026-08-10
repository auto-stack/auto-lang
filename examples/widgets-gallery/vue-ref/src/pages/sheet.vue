<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const sheetBasicAutoCode = `sheet {
    sheet-trigger {
        button (variant: "outline", text: "Open") {}
    }
    sheet-content {
        sheet-header {
            sheet-title (text: "Edit Profile") {}
            sheet-description (text: "Make changes to your profile here.") {}
        }
        "Profile content goes here."
    }
}
`
const sheetBasicVueCode = `<Sheet>
  <SheetTrigger>
    <Button variant="outline">Open</Button>
  </SheetTrigger>
  <SheetContent>
    <SheetHeader>
      <SheetTitle>Edit Profile</SheetTitle>
      <SheetDescription>Make changes to your profile here.</SheetDescription>
    </SheetHeader>
    <div>Profile content goes here.</div>
  </SheetContent>
</Sheet>
`

const sheetSidesAutoCode = `row (gap: "4") {
    sheet (side: "top") {
        sheet-trigger { button (variant: "outline", text: "Top") {} }
        sheet-content { sheet-header { sheet-title (text: "Top Sheet") {} } }
    }
    sheet (side: "right") {
        sheet-trigger { button (variant: "outline", text: "Right") {} }
        sheet-content { sheet-header { sheet-title (text: "Right Sheet") {} } }
    }
    sheet (side: "bottom") {
        sheet-trigger { button (variant: "outline", text: "Bottom") {} }
        sheet-content { sheet-header { sheet-title (text: "Bottom Sheet") {} } }
    }
    sheet (side: "left") {
        sheet-trigger { button (variant: "outline", text: "Left") {} }
        sheet-content { sheet-header { sheet-title (text: "Left Sheet") {} } }
    }
}
`
const sheetSidesVueCode = `<div class="flex flex-row gap-4">
  <Sheet>
    <SheetTrigger><Button variant="outline">Top</Button></SheetTrigger>
    <SheetContent side="top">
      <SheetHeader><SheetTitle>Top Sheet</SheetTitle></SheetHeader>
    </SheetContent>
  </Sheet>
  <Sheet>
    <SheetTrigger><Button variant="outline">Right</Button></SheetTrigger>
    <SheetContent side="right">
      <SheetHeader><SheetTitle>Right Sheet</SheetTitle></SheetHeader>
    </SheetContent>
  </Sheet>
  <Sheet>
    <SheetTrigger><Button variant="outline">Bottom</Button></SheetTrigger>
    <SheetContent side="bottom">
      <SheetHeader><SheetTitle>Bottom Sheet</SheetTitle></SheetHeader>
    </SheetContent>
  </Sheet>
  <Sheet>
    <SheetTrigger><Button variant="outline">Left</Button></SheetTrigger>
    <SheetContent side="left">
      <SheetHeader><SheetTitle>Left Sheet</SheetTitle></SheetHeader>
    </SheetContent>
  </Sheet>
</div>
`
</script>

<template>
  <ComponentDocPage title="Sheet" description="A panel that slides in from the side of the screen." installCommand="npx shadcn-vue@latest add sheet">
    <DemoSection title="Simple" id="sheet-basic" :autoCode="sheetBasicAutoCode" :vueCode="sheetBasicVueCode">
      <template #preview>
        <Sheet>
          <SheetTrigger>
            <Button variant="outline">Open</Button>
          </SheetTrigger>
          <SheetContent>
            <SheetHeader>
              <SheetTitle>Edit Profile</SheetTitle>
              <SheetDescription>Make changes to your profile here.</SheetDescription>
            </SheetHeader>
            <div>Profile content goes here.</div>
          </SheetContent>
        </Sheet>
      </template>
    </DemoSection>

    <DemoSection title="Sides" id="sheet-sides" :autoCode="sheetSidesAutoCode" :vueCode="sheetSidesVueCode">
      <template #preview>
        <div class="flex flex-row gap-4 flex-wrap">
          <Sheet>
            <SheetTrigger><Button variant="outline">Top</Button></SheetTrigger>
            <SheetContent side="top">
              <SheetHeader><SheetTitle>Top Sheet</SheetTitle></SheetHeader>
            </SheetContent>
          </Sheet>
          <Sheet>
            <SheetTrigger><Button variant="outline">Right</Button></SheetTrigger>
            <SheetContent side="right">
              <SheetHeader><SheetTitle>Right Sheet</SheetTitle></SheetHeader>
            </SheetContent>
          </Sheet>
          <Sheet>
            <SheetTrigger><Button variant="outline">Bottom</Button></SheetTrigger>
            <SheetContent side="bottom">
              <SheetHeader><SheetTitle>Bottom Sheet</SheetTitle></SheetHeader>
            </SheetContent>
          </Sheet>
          <Sheet>
            <SheetTrigger><Button variant="outline">Left</Button></SheetTrigger>
            <SheetContent side="left">
              <SheetHeader><SheetTitle>Left Sheet</SheetTitle></SheetHeader>
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
