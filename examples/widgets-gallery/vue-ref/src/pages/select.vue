<script setup lang="ts">
import { ScrollArea } from '@/components/ui/scroll-area'
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const selectBasicAutoCode = `select {
    select-trigger {
        select-value (placeholder: "Select a fruit") {}
    }
    select-content {
        select-group {
            select-label (text: "Fruits") {}
            select-item (value: "apple", text: "Apple") {}
            select-item (value: "banana", text: "Banana") {}
            select-item (value: "blueberry", text: "Blueberry") {}
            select-item (text: "Grapes", value: "grapes") {}
            select-item (value: "pineapple", text: "Pineapple") {}
        }
    }
}
`
const selectBasicVueCode = `<Select>
  <SelectTrigger class="w-[180px]">
    <SelectValue placeholder="Select a fruit" />
  </SelectTrigger>
  <SelectContent>
    <SelectGroup>
      <SelectLabel>Fruits</SelectLabel>
      <SelectItem value="apple">Apple</SelectItem>
      <SelectItem value="banana">Banana</SelectItem>
      <SelectItem value="blueberry">Blueberry</SelectItem>
      <SelectItem value="grapes">Grapes</SelectItem>
      <SelectItem value="pineapple">Pineapple</SelectItem>
    </SelectGroup>
  </SelectContent>
</Select>
`
const selectScrollableAutoCode = `select {
    select-trigger (class: "w-[280px]") {
        select-value (placeholder: "Select a timezone") {}
    }
    select-content {
        scroll (class: "h-[200px]") {
            select-item (text: "Pacific Standard Time", value: "pst") {}
            select-item (value: "mst", text: "Mountain Standard Time") {}
            select-item (value: "cst", text: "Central Standard Time") {}
            select-item (value: "est", text: "Eastern Standard Time") {}
            select-item (value: "ast", text: "Atlantic Standard Time") {}
            select-item (value: "gmt", text: "Greenwich Mean Time") {}
            select-item (value: "cet", text: "Central European Time") {}
            select-item (text: "Eastern European Time", value: "eet") {}
            select-item (value: "jst", text: "Japan Standard Time") {}
            select-item (text: "Australian Eastern Time", value: "aet") {}
        }
    }
}
`
const selectScrollableVueCode = `<Select>
  <SelectTrigger class="w-[280px]">
    <SelectValue placeholder="Select a timezone" />
  </SelectTrigger>
  <SelectContent>
    <ScrollArea class="h-[200px]">
      <SelectItem value="pst">Pacific Standard Time</SelectItem>
      <SelectItem value="mst">Mountain Standard Time</SelectItem>
      <SelectItem value="cst">Central Standard Time</SelectItem>
      <SelectItem value="est">Eastern Standard Time</SelectItem>
      <SelectItem value="ast">Atlantic Standard Time</SelectItem>
      <SelectItem value="gmt">Greenwich Mean Time</SelectItem>
      <SelectItem value="cet">Central European Time</SelectItem>
      <SelectItem value="eet">Eastern European Time</SelectItem>
      <SelectItem value="jst">Japan Standard Time</SelectItem>
      <SelectItem value="aet">Australian Eastern Time</SelectItem>
    </ScrollArea>
  </SelectContent>
</Select>
`
</script>

<template>
  <ComponentDocPage title="Select" description="Displays a list of options for the user to pick from." installCommand="npx shadcn-vue@latest add select">
    <DemoSection title="Simple" id="select-basic" :autoCode="selectBasicAutoCode" :vueCode="selectBasicVueCode">
      <template #preview>
        <Select>
          <SelectTrigger class="w-[180px]">
            <SelectValue placeholder="Select a fruit" />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectLabel>Fruits</SelectLabel>
              <SelectItem value="apple">Apple</SelectItem>
              <SelectItem value="banana">Banana</SelectItem>
              <SelectItem value="blueberry">Blueberry</SelectItem>
              <SelectItem value="grapes">Grapes</SelectItem>
              <SelectItem value="pineapple">Pineapple</SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </template>
    </DemoSection>
    <DemoSection title="Scrollable" id="select-scrollable" :autoCode="selectScrollableAutoCode" :vueCode="selectScrollableVueCode">
      <template #preview>
        <Select>
          <SelectTrigger class="w-[280px]">
            <SelectValue placeholder="Select a timezone" />
          </SelectTrigger>
          <SelectContent>
            <ScrollArea class="h-[200px]">
              <SelectItem value="pst">Pacific Standard Time</SelectItem>
              <SelectItem value="mst">Mountain Standard Time</SelectItem>
              <SelectItem value="cst">Central Standard Time</SelectItem>
              <SelectItem value="est">Eastern Standard Time</SelectItem>
              <SelectItem value="ast">Atlantic Standard Time</SelectItem>
              <SelectItem value="gmt">Greenwich Mean Time</SelectItem>
              <SelectItem value="cet">Central European Time</SelectItem>
              <SelectItem value="eet">Eastern European Time</SelectItem>
              <SelectItem value="jst">Japan Standard Time</SelectItem>
              <SelectItem value="aet">Australian Eastern Time</SelectItem>
            </ScrollArea>
          </SelectContent>
        </Select>
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
            <TableCell>placeholder</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Placeholder text for select-value</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the select</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </template>
  </ComponentDocPage>
</template>

<style scoped>
pre[class*="language-"] {
  margin: 0;
}
</style>
