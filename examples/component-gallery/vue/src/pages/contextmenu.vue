<script setup lang="ts">
import { ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuSeparator, ContextMenuShortcut, ContextMenuTrigger } from '@/components/ui/context-menu'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const contextmenuBasicAutoCode = `context-menu {
    context-menu-trigger (class: "flex h-[149px] w-[300px] items-center justify-center rounded-md border border-dashed text-sm") {
        "Right click here"
    }
    context-menu-content (class: "w-53") {
        context-menu-item (text: "Back") {
            context-menu-shortcut (text: "⌘[") {}
        }
        context-menu-item (disabled: "true", text: "Forward") {
            context-menu-shortcut (text: "⌘]") {}
        }
        context-menu-item (text: "Reload") {
            context-menu-shortcut (text: "⌘R") {}
        }
        context-menu-separator {}
        context-menu-item (text: "Profile") {}
        context-menu-item (text: "Billing") {}
        context-menu-item (text: "Team") {}
        context-menu-separator {}
        context-menu-item (text: "Subscription") {}
    }
}
`
const contextmenuBasicVueCode = `<ContextMenu>
  <ContextMenuTrigger class="flex h-[149px] w-[300px] items-center justify-center rounded-md border border-dashed text-sm">
    Right click here
  </ContextMenuTrigger>
  <ContextMenuContent class="w-53">
    <ContextMenuItem>Back
      <ContextMenuShortcut>⌘[</ContextMenuShortcut>
    </ContextMenuItem>
    <ContextMenuItem disabled>Forward
      <ContextMenuShortcut>⌘]</ContextMenuShortcut>
    </ContextMenuItem>
    <ContextMenuItem>Reload
      <ContextMenuShortcut>⌘R</ContextMenuShortcut>
    </ContextMenuItem>
    <ContextMenuSeparator />
    <ContextMenuItem>Profile</ContextMenuItem>
    <ContextMenuItem>Billing</ContextMenuItem>
    <ContextMenuItem>Team</ContextMenuItem>
    <ContextMenuSeparator />
    <ContextMenuItem>Subscription</ContextMenuItem>
  </ContextMenuContent>
</ContextMenu>
`
</script>

<template>
  <ComponentDocPage title="ContextMenu" description="Displays a menu to the user — such as a set of actions or functions — triggered by a right click." installCommand="npx shadcn-vue@latest add context-menu">
    <DemoSection title="Example" id="contextmenu-basic" :autoCode="contextmenuBasicAutoCode" :vueCode="contextmenuBasicVueCode">
      <template #preview>
        <ContextMenu>
              <ContextMenuTrigger class="flex h-[149px] w-[300px] items-center justify-center rounded-md border border-dashed text-sm">
                Right click here
              </ContextMenuTrigger>
              <ContextMenuContent class="w-53">
                <ContextMenuItem>Back
                  <ContextMenuShortcut>⌘[</ContextMenuShortcut>
                </ContextMenuItem>
                <ContextMenuItem disabled>Forward
                  <ContextMenuShortcut>⌘]</ContextMenuShortcut>
                </ContextMenuItem>
                <ContextMenuItem>Reload
                  <ContextMenuShortcut>⌘R</ContextMenuShortcut>
                </ContextMenuItem>
                <ContextMenuSeparator />
                <ContextMenuItem>Profile</ContextMenuItem>
                <ContextMenuItem>Billing</ContextMenuItem>
                <ContextMenuItem>Team</ContextMenuItem>
                <ContextMenuSeparator />
                <ContextMenuItem>Subscription</ContextMenuItem>
              </ContextMenuContent>
            </ContextMenu>
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
            <TableCell>text</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Menu item text</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the menu item</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>shortcut</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Keyboard shortcut hint</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>inset</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Adds inset padding</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>variant</TableCell>
            <TableCell>string</TableCell>
            <TableCell>default</TableCell>
            <TableCell>default, destructive</TableCell>
            <TableCell>Item style variant</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>class</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Additional CSS classes</TableCell>
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
