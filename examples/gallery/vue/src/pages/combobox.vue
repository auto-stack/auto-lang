<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from '@/components/ui/command'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const comboboxBasicAutoCode = `popover {
    popover-trigger {
        button (variant: "outline", text: "Select framework...") {}
    }
    popover-content {
        command {
            command-input (placeholder: "Search framework...") {}
            command-empty (text: "No framework found.") {}
            command-list {
                command-group {
                    command-item (text: "Vue") {}
                    command-item (text: "React") {}
                    command-item (text: "Svelte") {}
                }
            }
        }
    }
}
`
const comboboxBasicVueCode = `<Popover>
  <PopoverTrigger>
    <Button variant="outline">Select framework...</Button>
  </PopoverTrigger>
  <PopoverContent>
    <Command>
      <CommandInput placeholder="Search framework..." />
      <CommandEmpty>No framework found.</CommandEmpty>
      <CommandList>
        <CommandGroup>
          <CommandItem value="Vue">Vue</CommandItem>
          <CommandItem value="React">React</CommandItem>
          <CommandItem value="Svelte">Svelte</CommandItem>
        </CommandGroup>
      </CommandList>
    </Command>
  </PopoverContent>
</Popover>
`
</script>

<template>
  <ComponentDocPage title="Combobox" description="An autocomplete input with a listbox, combining a text input with a dropdown." installCommand="npx shadcn-vue@latest add combobox">
    <DemoSection title="Simple" id="combobox-basic" :autoCode="comboboxBasicAutoCode" :vueCode="comboboxBasicVueCode">
      <template #preview>
        <Popover>
              <PopoverTrigger>
                <Button variant="outline">Select framework...</Button>
              </PopoverTrigger>
              <PopoverContent>
                <Command>
                  <CommandInput placeholder="Search framework..." />
                  <CommandEmpty>No framework found.</CommandEmpty>
                  <CommandList>
                    <CommandGroup>
                      <CommandItem value="Vue">Vue</CommandItem>
                      <CommandItem value="React">React</CommandItem>
                      <CommandItem value="Svelte">Svelte</CommandItem>
                    </CommandGroup>
                  </CommandList>
                </Command>
              </PopoverContent>
            </Popover>
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
            <TableCell>v-model</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Selected value</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>filter</TableCell>
            <TableCell>function</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Custom filter function</TableCell>
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

