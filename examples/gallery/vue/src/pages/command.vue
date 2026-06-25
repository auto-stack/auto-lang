<script setup lang="ts">
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList, CommandSeparator } from '@/components/ui/command'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const commandBasicAutoCode = `command (class: "rounded-lg border shadow-md") {
    command-input (placeholder: "Type a command or search...") {}
    command-list {
        command-empty (text: "No results found.") {}
        command-group (heading: "Suggestions") {
            command-item (text: "Calendar") {}
            command-item (text: "Search Emoji") {}
            command-item (text: "Calculator") {}
        }
        command-separator {}
        command-group (heading: "Settings") {
            command-item (text: "Profile") {}
            command-item (text: "Billing") {}
            command-item (text: "Settings") {}
        }
    }
}
`
const commandBasicVueCode = `<Command>
  <CommandInput placeholder="Type a command or search..." />
  <CommandList>
    <CommandEmpty>No results found.</CommandEmpty>
    <CommandGroup heading="Suggestions">
      <CommandItem value="Calendar">Calendar</CommandItem>
      <CommandItem value="Search Emoji">Search Emoji</CommandItem>
      <CommandItem value="Calculator">Calculator</CommandItem>
    </CommandGroup>
    <CommandSeparator />
    <CommandGroup heading="Settings">
      <CommandItem value="Profile">Profile</CommandItem>
      <CommandItem value="Billing">Billing</CommandItem>
      <CommandItem value="Settings">Settings</CommandItem>
    </CommandGroup>
  </CommandList>
</Command>
`
</script>

<template>
  <ComponentDocPage title="Command" description="A command menu for searching and performing actions quickly. Often used with keyboard shortcuts." installCommand="npx shadcn-vue@latest add command">
    <DemoSection title="Simple" id="command-basic" :autoCode="commandBasicAutoCode" :vueCode="commandBasicVueCode">
      <template #preview>
        <Command>
              <CommandInput placeholder="Type a command or search..." />
              <CommandList>
                <CommandEmpty>No results found.</CommandEmpty>
                <CommandGroup heading="Suggestions">
                  <CommandItem value="Calendar">Calendar</CommandItem>
                  <CommandItem value="Search Emoji">Search Emoji</CommandItem>
                  <CommandItem value="Calculator">Calculator</CommandItem>
                </CommandGroup>
                <CommandSeparator />
                <CommandGroup heading="Settings">
                  <CommandItem value="Profile">Profile</CommandItem>
                  <CommandItem value="Billing">Billing</CommandItem>
                  <CommandItem value="Settings">Settings</CommandItem>
                </CommandGroup>
              </CommandList>
            </Command>
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
            <TableCell>filter</TableCell>
            <TableCell>function</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Custom filter function</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>loop</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Loop through items</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>v-model</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Search term</TableCell>
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

