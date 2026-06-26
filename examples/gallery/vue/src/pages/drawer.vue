<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader, DrawerTitle, DrawerTrigger } from '@/components/ui/drawer'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const drawerBasicAutoCode = `drawer {
    drawer-trigger {
        button (variant: "outline", text: "Open Drawer") {}
    }
    drawer-content {
        drawer-header {
            drawer-title (text: "Are you absolutely sure?") {}
            drawer-description (text: "This action cannot be undone.") {}
        }
        drawer-footer {
            drawer-close {
                button (variant: "outline", text: "Cancel") {}
            }
            button (text: "Submit") {}
        }
    }
}
`
const drawerBasicVueCode = `<Drawer>
  <DrawerTrigger>
    <Button variant="outline">Open Drawer</Button>
  </DrawerTrigger>
  <DrawerContent>
    <DrawerHeader>
      <DrawerTitle>Are you absolutely sure?</DrawerTitle>
      <DrawerDescription>This action cannot be undone.</DrawerDescription>
    </DrawerHeader>
    <DrawerFooter>
      <DrawerClose>
        <Button variant="outline">Cancel</Button>
      </DrawerClose>
      <Button>Submit</Button>
    </DrawerFooter>
  </DrawerContent>
</Drawer>
`

const drawerBottomAutoCode = `drawer (direction: "bottom") {
    drawer-trigger {
        button (variant: "outline", text: "Open Bottom") {}
    }
    drawer-content {
        drawer-header {
            drawer-title (text: "Bottom Drawer") {}
            drawer-description (text: "This drawer slides from the bottom.") {}
        }
        drawer-footer {
            drawer-close {
                button (variant: "outline", text: "Close") {}
            }
        }
    }
}
`
const drawerBottomVueCode = `<Drawer direction="bottom">
  <DrawerTrigger>
    <Button variant="outline">Open Bottom</Button>
  </DrawerTrigger>
  <DrawerContent>
    <DrawerHeader>
      <DrawerTitle>Bottom Drawer</DrawerTitle>
      <DrawerDescription>This drawer slides from the bottom.</DrawerDescription>
    </DrawerHeader>
    <DrawerFooter>
      <DrawerClose>
        <Button variant="outline">Close</Button>
      </DrawerClose>
    </DrawerFooter>
  </DrawerContent>
</Drawer>
`
</script>

<template>
  <ComponentDocPage title="Drawer" description="A drawer that slides in from the edge of the screen." installCommand="npx shadcn-vue@latest add drawer">
    <DemoSection title="Simple" id="drawer-basic" :autoCode="drawerBasicAutoCode" :vueCode="drawerBasicVueCode">
      <template #preview>
        <Drawer>
          <DrawerTrigger>
            <Button variant="outline">Open Drawer</Button>
          </DrawerTrigger>
          <DrawerContent>
            <DrawerHeader>
              <DrawerTitle>Are you absolutely sure?</DrawerTitle>
              <DrawerDescription>This action cannot be undone.</DrawerDescription>
            </DrawerHeader>
            <DrawerFooter>
              <DrawerClose>
                <Button variant="outline">Cancel</Button>
              </DrawerClose>
              <Button>Submit</Button>
            </DrawerFooter>
          </DrawerContent>
        </Drawer>
      </template>
    </DemoSection>

    <DemoSection title="Bottom" id="drawer-bottom" :autoCode="drawerBottomAutoCode" :vueCode="drawerBottomVueCode">
      <template #preview>
        <Drawer direction="bottom">
          <DrawerTrigger>
            <Button variant="outline">Open Bottom</Button>
          </DrawerTrigger>
          <DrawerContent>
            <DrawerHeader>
              <DrawerTitle>Bottom Drawer</DrawerTitle>
              <DrawerDescription>This drawer slides from the bottom.</DrawerDescription>
            </DrawerHeader>
            <DrawerFooter>
              <DrawerClose>
                <Button variant="outline">Close</Button>
              </DrawerClose>
            </DrawerFooter>
          </DrawerContent>
        </Drawer>
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
            <TableCell>Controls drawer visibility</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>direction</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"bottom"</TableCell>
            <TableCell>"top", "right", "bottom", "left"</TableCell>
            <TableCell>Drawer slide direction</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>title</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Drawer title</TableCell>
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
