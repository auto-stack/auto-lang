<script setup lang="ts">
import { Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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

const dialogWithFormAutoCode = `dialog {
    dialog-trigger (text: "Edit Profile") {}
    dialog-content {
        dialog-header {
            dialog-title (text: "Edit profile") {}
            dialog-description (text: "Make changes to your profile here.") {}
        }
        col (gap: "4") {
            row (gap: "4", align: "center") {
                label (text: "Name", class: "w-16") {}
                input (value: "Pedro Duarte", class: "flex-1") {}
            }
            row (gap: "4", align: "center") {
                label (text: "Username", class: "w-16") {}
                input (value: "@peduarte", class: "flex-1") {}
            }
        }
        dialog-footer {
            dialog-close {
                button (text: "Save changes") {}
            }
        }
    }
}
`
const dialogWithFormVueCode = `<Dialog>
  <DialogTrigger>
    <Button variant="outline">Edit Profile</Button>
  </DialogTrigger>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Edit profile</DialogTitle>
      <DialogDescription>
        Make changes to your profile here.
      </DialogDescription>
    </DialogHeader>
    <div class="flex flex-col gap-4 py-4">
      <div class="flex items-center gap-4">
        <Label class="w-16 text-right">Name</Label>
        <Input value="Pedro Duarte" class="flex-1" />
      </div>
      <div class="flex items-center gap-4">
        <Label class="w-16 text-right">Username</Label>
        <Input value="@peduarte" class="flex-1" />
      </div>
    </div>
    <DialogFooter>
      <DialogClose>
        <Button>Save changes</Button>
      </DialogClose>
    </DialogFooter>
  </DialogContent>
</Dialog>
`

const dialogCustomSizeAutoCode = `dialog {
    dialog-trigger (text: "Large Dialog") {}
    dialog-content (class: "sm:max-w-xl") {
        dialog-header {
            dialog-title (text: "Large Dialog") {}
            dialog-description (text: "This dialog has a wider content area.") {}
        }
        "More content can go here with the extra width."
        dialog-footer {
            dialog-close {
                button (text: "Close") {}
            }
        }
    }
}
`
const dialogCustomSizeVueCode = `<Dialog>
  <DialogTrigger>
    <Button variant="outline">Large Dialog</Button>
  </DialogTrigger>
  <DialogContent class="sm:max-w-xl">
    <DialogHeader>
      <DialogTitle>Large Dialog</DialogTitle>
      <DialogDescription>
        This dialog has a wider content area.
      </DialogDescription>
    </DialogHeader>
    <div>More content can go here with the extra width.</div>
    <DialogFooter>
      <DialogClose>
        <Button>Close</Button>
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

    <DemoSection title="With Form" id="dialog-form" :autoCode="dialogWithFormAutoCode" :vueCode="dialogWithFormVueCode">
      <template #preview>
        <Dialog>
          <DialogTrigger>
            <Button variant="outline">Edit Profile</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Edit profile</DialogTitle>
              <DialogDescription>
                Make changes to your profile here.
              </DialogDescription>
            </DialogHeader>
            <div class="flex flex-col gap-4 py-4">
              <div class="flex items-center gap-4">
                <Label class="w-16 text-right">Name</Label>
                <Input value="Pedro Duarte" class="flex-1" />
              </div>
              <div class="flex items-center gap-4">
                <Label class="w-16 text-right">Username</Label>
                <Input value="@peduarte" class="flex-1" />
              </div>
            </div>
            <DialogFooter>
              <DialogClose>
                <Button>Save changes</Button>
              </DialogClose>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </template>
    </DemoSection>

    <DemoSection title="Custom Size" id="dialog-size" :autoCode="dialogCustomSizeAutoCode" :vueCode="dialogCustomSizeVueCode">
      <template #preview>
        <Dialog>
          <DialogTrigger>
            <Button variant="outline">Large Dialog</Button>
          </DialogTrigger>
          <DialogContent class="sm:max-w-xl">
            <DialogHeader>
              <DialogTitle>Large Dialog</DialogTitle>
              <DialogDescription>
                This dialog has a wider content area.
              </DialogDescription>
            </DialogHeader>
            <div>More content can go here with the extra width.</div>
            <DialogFooter>
              <DialogClose>
                <Button>Close</Button>
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
