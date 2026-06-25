<script setup lang="ts">
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
  DropdownMenuGroup,
  DropdownMenuCheckboxItem,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
} from '@/components/ui/dropdown-menu'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'
import { ref } from 'vue'

const showStatusBar = ref(true)
const showActivityBar = ref(false)
const position = ref('bottom')

const dropdownBasicAutoCode = `dropdown-menu {
    dropdown-menu-trigger {
        button (variant: "outline", text: "Open") {}
    }
    dropdown-menu-content {
        dropdown-menu-item (text: "Profile") {}
        dropdown-menu-item (text: "Billing") {}
        dropdown-menu-item (text: "Settings") {}
        dropdown-menu-separator {}
        dropdown-menu-item (text: "Log out") {}
    }
}
`
const dropdownBasicVueCode = `<DropdownMenu>
  <DropdownMenuTrigger>
    <Button variant="outline">Open</Button>
  </DropdownMenuTrigger>
  <DropdownMenuContent>
    <DropdownMenuItem>Profile</DropdownMenuItem>
    <DropdownMenuItem>Billing</DropdownMenuItem>
    <DropdownMenuItem>Settings</DropdownMenuItem>
    <DropdownMenuSeparator />
    <DropdownMenuItem>Log out</DropdownMenuItem>
  </DropdownMenuContent>
</DropdownMenu>
`

const dropdownWithShortcutsAutoCode = `dropdown-menu {
    dropdown-menu-trigger {
        button (variant: "outline", text: "Open") {}
    }
    dropdown-menu-content {
        dropdown-menu-item (text: "Back") {
            dropdown-menu-shortcut (text: "⌘[") {}
        }
        dropdown-menu-item (text: "Forward") {
            dropdown-menu-shortcut (text: "⌘]") {}
        }
        dropdown-menu-item (text: "Reload") {
            dropdown-menu-shortcut (text: "⌘R") {}
        }
    }
}
`
const dropdownWithShortcutsVueCode = `<DropdownMenu>
  <DropdownMenuTrigger>
    <Button variant="outline">Open</Button>
  </DropdownMenuTrigger>
  <DropdownMenuContent>
    <DropdownMenuItem>
      Back
      <DropdownMenuShortcut>⌘[</DropdownMenuShortcut>
    </DropdownMenuItem>
    <DropdownMenuItem>
      Forward
      <DropdownMenuShortcut>⌘]</DropdownMenuShortcut>
    </DropdownMenuItem>
    <DropdownMenuItem>
      Reload
      <DropdownMenuShortcut>⌘R</DropdownMenuShortcut>
    </DropdownMenuItem>
  </DropdownMenuContent>
</DropdownMenu>
`

const dropdownWithGroupsAutoCode = `dropdown-menu {
    dropdown-menu-trigger {
        button (variant: "outline", text: "Open") {}
    }
    dropdown-menu-content {
        dropdown-menu-label (text: "My Account") {}
        dropdown-menu-separator {}
        dropdown-menu-group {
            dropdown-menu-item (text: "Profile") {}
            dropdown-menu-item (text: "Billing") {}
            dropdown-menu-item (text: "Settings") {}
        }
        dropdown-menu-separator {}
        dropdown-menu-group {
            dropdown-menu-item (text: "Team") {}
            dropdown-menu-item (text: "Invite users") {}
        }
    }
}
`
const dropdownWithGroupsVueCode = `<DropdownMenu>
  <DropdownMenuTrigger>
    <Button variant="outline">Open</Button>
  </DropdownMenuTrigger>
  <DropdownMenuContent>
    <DropdownMenuLabel>My Account</DropdownMenuLabel>
    <DropdownMenuSeparator />
    <DropdownMenuGroup>
      <DropdownMenuItem>Profile</DropdownMenuItem>
      <DropdownMenuItem>Billing</DropdownMenuItem>
      <DropdownMenuItem>Settings</DropdownMenuItem>
    </DropdownMenuGroup>
    <DropdownMenuSeparator />
    <DropdownMenuGroup>
      <DropdownMenuItem>Team</DropdownMenuItem>
      <DropdownMenuItem>Invite users</DropdownMenuItem>
    </DropdownMenuGroup>
  </DropdownMenuContent>
</DropdownMenu>
`

const dropdownCheckboxAutoCode = `dropdown-menu {
    dropdown-menu-trigger {
        button (variant: "outline", text: "Open") {}
    }
    dropdown-menu-content {
        dropdown-menu-label (text: "Appearance") {}
        dropdown-menu-separator {}
        dropdown-menu-checkbox-item (checked: .showStatusBar, text: "Status Bar") {}
        dropdown-menu-checkbox-item (checked: .showActivityBar, text: "Activity Bar") {}
    }
}
`
const dropdownCheckboxVueCode = `<DropdownMenu>
  <DropdownMenuTrigger>
    <Button variant="outline">Open</Button>
  </DropdownMenuTrigger>
  <DropdownMenuContent>
    <DropdownMenuLabel>Appearance</DropdownMenuLabel>
    <DropdownMenuSeparator />
    <DropdownMenuCheckboxItem v-model:checked="showStatusBar">
      Status Bar
    </DropdownMenuCheckboxItem>
    <DropdownMenuCheckboxItem v-model:checked="showActivityBar">
      Activity Bar
    </DropdownMenuCheckboxItem>
  </DropdownMenuContent>
</DropdownMenu>
`

const dropdownRadioAutoCode = `dropdown-menu {
    dropdown-menu-trigger {
        button (variant: "outline", text: "Open") {}
    }
    dropdown-menu-content {
        dropdown-menu-label (text: "Panel Position") {}
        dropdown-menu-separator {}
        dropdown-menu-radio-group (value: .position) {
            dropdown-menu-radio-item (value: "top", text: "Top") {}
            dropdown-menu-radio-item (value: "bottom", text: "Bottom") {}
            dropdown-menu-radio-item (value: "right", text: "Right") {}
        }
    }
}
`
const dropdownRadioVueCode = `<DropdownMenu>
  <DropdownMenuTrigger>
    <Button variant="outline">Open</Button>
  </DropdownMenuTrigger>
  <DropdownMenuContent>
    <DropdownMenuLabel>Panel Position</DropdownMenuLabel>
    <DropdownMenuSeparator />
    <DropdownMenuRadioGroup v-model="position">
      <DropdownMenuRadioItem value="top">Top</DropdownMenuRadioItem>
      <DropdownMenuRadioItem value="bottom">Bottom</DropdownMenuRadioItem>
      <DropdownMenuRadioItem value="right">Right</DropdownMenuRadioItem>
    </DropdownMenuRadioGroup>
  </DropdownMenuContent>
</DropdownMenu>
`
</script>

<template>
  <ComponentDocPage title="DropdownMenu" description="A menu that drops down from a trigger element." installCommand="npx shadcn-vue@latest add dropdown-menu">
    <DemoSection title="Simple" id="dropdown-basic" :autoCode="dropdownBasicAutoCode" :vueCode="dropdownBasicVueCode">
      <template #preview>
        <DropdownMenu>
          <DropdownMenuTrigger>
            <Button variant="outline">Open</Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem>Profile</DropdownMenuItem>
            <DropdownMenuItem>Billing</DropdownMenuItem>
            <DropdownMenuItem>Settings</DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem>Log out</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </template>
    </DemoSection>

    <DemoSection title="With Shortcuts" id="dropdown-shortcuts" :autoCode="dropdownWithShortcutsAutoCode" :vueCode="dropdownWithShortcutsVueCode">
      <template #preview>
        <DropdownMenu>
          <DropdownMenuTrigger>
            <Button variant="outline">Open</Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem>
              Back
              <DropdownMenuShortcut>⌘[</DropdownMenuShortcut>
            </DropdownMenuItem>
            <DropdownMenuItem>
              Forward
              <DropdownMenuShortcut>⌘]</DropdownMenuShortcut>
            </DropdownMenuItem>
            <DropdownMenuItem>
              Reload
              <DropdownMenuShortcut>⌘R</DropdownMenuShortcut>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </template>
    </DemoSection>

    <DemoSection title="With Groups" id="dropdown-groups" :autoCode="dropdownWithGroupsAutoCode" :vueCode="dropdownWithGroupsVueCode">
      <template #preview>
        <DropdownMenu>
          <DropdownMenuTrigger>
            <Button variant="outline">Open</Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuLabel>My Account</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <DropdownMenuItem>Profile</DropdownMenuItem>
              <DropdownMenuItem>Billing</DropdownMenuItem>
              <DropdownMenuItem>Settings</DropdownMenuItem>
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <DropdownMenuItem>Team</DropdownMenuItem>
              <DropdownMenuItem>Invite users</DropdownMenuItem>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </template>
    </DemoSection>

    <DemoSection title="Checkbox Items" id="dropdown-checkbox" :autoCode="dropdownCheckboxAutoCode" :vueCode="dropdownCheckboxVueCode">
      <template #preview>
        <DropdownMenu>
          <DropdownMenuTrigger>
            <Button variant="outline">Open</Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuLabel>Appearance</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuCheckboxItem v-model:checked="showStatusBar">
              Status Bar
            </DropdownMenuCheckboxItem>
            <DropdownMenuCheckboxItem v-model:checked="showActivityBar">
              Activity Bar
            </DropdownMenuCheckboxItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </template>
    </DemoSection>

    <DemoSection title="Radio Items" id="dropdown-radio" :autoCode="dropdownRadioAutoCode" :vueCode="dropdownRadioVueCode">
      <template #preview>
        <DropdownMenu>
          <DropdownMenuTrigger>
            <Button variant="outline">Open</Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuLabel>Panel Position</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuRadioGroup v-model="position">
              <DropdownMenuRadioItem value="top">Top</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="bottom">Bottom</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="right">Right</DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
          </DropdownMenuContent>
        </DropdownMenu>
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
