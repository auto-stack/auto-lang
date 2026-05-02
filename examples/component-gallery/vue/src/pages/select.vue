<script setup lang="ts">
import { ScrollArea } from '@/components/ui/scroll-area'
import { Select } from '@/components/ui/select'
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
  <div>
    <div placeholder="Select a fruit" />
  </div>
  <div>
    <div>
      <div>Fruits</div>
      <div value="apple">Apple</div>
      <div value="banana">Banana</div>
      <div value="blueberry">Blueberry</div>
      <div value="grapes">Grapes</div>
      <div value="pineapple">Pineapple</div>
    </div>
  </div>
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
  <div class="w-[280px]">
    <div placeholder="Select a timezone" />
  </div>
  <div>
    <ScrollArea class="h-[200px]">
      <div value="pst">Pacific Standard Time</div>
      <div value="mst">Mountain Standard Time</div>
      <div value="cst">Central Standard Time</div>
      <div value="est">Eastern Standard Time</div>
      <div value="ast">Atlantic Standard Time</div>
      <div value="gmt">Greenwich Mean Time</div>
      <div value="cet">Central European Time</div>
      <div value="eet">Eastern European Time</div>
      <div value="jst">Japan Standard Time</div>
      <div value="aet">Australian Eastern Time</div>
    </ScrollArea>
  </div>
</Select>
`
</script>

<template>
  <ComponentDocPage title="Select" description="Displays a list of options for the user to pick from." installCommand="npx shadcn-vue@latest add select">
    <DemoSection title="Simple" id="select-basic" :autoCode="selectBasicAutoCode" :vueCode="selectBasicVueCode">
      <template #preview>
        <Select>
              <div>
                <div placeholder="Select a fruit" />
              </div>
              <div>
                <div>
                  <div>Fruits</div>
                  <div value="apple">Apple</div>
                  <div value="banana">Banana</div>
                  <div value="blueberry">Blueberry</div>
                  <div value="grapes">Grapes</div>
                  <div value="pineapple">Pineapple</div>
                </div>
              </div>
            </Select>
      </template>
    </DemoSection>
    <DemoSection title="Scrollable" id="select-scrollable" :autoCode="selectScrollableAutoCode" :vueCode="selectScrollableVueCode">
      <template #preview>
        <Select>
              <div class="w-[280px]">
                <div placeholder="Select a timezone" />
              </div>
              <div>
                <ScrollArea class="h-[200px]">
                  <div value="pst">Pacific Standard Time</div>
                  <div value="mst">Mountain Standard Time</div>
                  <div value="cst">Central Standard Time</div>
                  <div value="est">Eastern Standard Time</div>
                  <div value="ast">Atlantic Standard Time</div>
                  <div value="gmt">Greenwich Mean Time</div>
                  <div value="cet">Central European Time</div>
                  <div value="eet">Eastern European Time</div>
                  <div value="jst">Japan Standard Time</div>
                  <div value="aet">Australian Eastern Time</div>
                </ScrollArea>
              </div>
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
/* Override Prism.js default styles */
pre[class*="language-"] {
  margin: 0;
}

/* Component styles */

</style>
