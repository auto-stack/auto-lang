<script setup lang="ts">
import { Table, TableBody, TableCaption, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const tableBasicAutoCode = `table {
    table-caption (text: "A list of your recent invoices.") {}
    table-header {
        table-row {
            table-head (text: "Invoice", class: "w-[100px]") {}
            table-head (text: "Status") {}
            table-head (text: "Method") {}
            table-head (text: "Amount", class: "text-right") {}
        }
    }
    table-body {
        table-row {
            table-cell (text: "INV001", class: "font-medium") {}
            table-cell (text: "Paid") {}
            table-cell (text: "Credit Card") {}
            table-cell (text: "$250.00", class: "text-right") {}
        }
        table-row {
            table-cell (text: "INV002", class: "font-medium") {}
            table-cell (text: "Pending") {}
            table-cell (text: "PayPal") {}
            table-cell (text: "$150.00", class: "text-right") {}
        }
        table-row {
            table-cell (text: "INV003", class: "font-medium") {}
            table-cell (text: "Unpaid") {}
            table-cell (text: "Bank Transfer") {}
            table-cell (class: "text-right", text: "$350.00") {}
        }
        table-row {
            table-cell (text: "INV004", class: "font-medium") {}
            table-cell (text: "Paid") {}
            table-cell (text: "Credit Card") {}
            table-cell (text: "$450.00", class: "text-right") {}
        }
        table-row {
            table-cell (class: "font-medium", text: "INV005") {}
            table-cell (text: "Paid") {}
            table-cell (text: "PayPal") {}
            table-cell (text: "$550.00", class: "text-right") {}
        }
        table-row {
            table-cell (class: "font-medium", text: "INV006") {}
            table-cell (text: "Pending") {}
            table-cell (text: "Bank Transfer") {}
            table-cell (class: "text-right", text: "$200.00") {}
        }
        table-row {
            table-cell (text: "INV007", class: "font-medium") {}
            table-cell (text: "Unpaid") {}
            table-cell (text: "Credit Card") {}
            table-cell (class: "text-right", text: "$300.00") {}
        }
    }
}
`
const tableBasicVueCode = `<Table>
  <TableCaption>A list of your recent invoices.</TableCaption>
  <TableHeader>
    <TableRow>
      <TableHead class="w-[100px]">Invoice</TableHead>
      <TableHead>Status</TableHead>
      <TableHead>Method</TableHead>
      <TableHead class="text-right">Amount</TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    <TableRow>
      <TableCell class="font-medium">INV001</TableCell>
      <TableCell>Paid</TableCell>
      <TableCell>Credit Card</TableCell>
      <TableCell class="text-right">$250.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV002</TableCell>
      <TableCell>Pending</TableCell>
      <TableCell>PayPal</TableCell>
      <TableCell class="text-right">$150.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV003</TableCell>
      <TableCell>Unpaid</TableCell>
      <TableCell>Bank Transfer</TableCell>
      <TableCell class="text-right">$350.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV004</TableCell>
      <TableCell>Paid</TableCell>
      <TableCell>Credit Card</TableCell>
      <TableCell class="text-right">$450.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV005</TableCell>
      <TableCell>Paid</TableCell>
      <TableCell>PayPal</TableCell>
      <TableCell class="text-right">$550.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV006</TableCell>
      <TableCell>Pending</TableCell>
      <TableCell>Bank Transfer</TableCell>
      <TableCell class="text-right">$200.00</TableCell>
    </TableRow>
    <TableRow>
      <TableCell class="font-medium">INV007</TableCell>
      <TableCell>Unpaid</TableCell>
      <TableCell>Credit Card</TableCell>
      <TableCell class="text-right">$300.00</TableCell>
    </TableRow>
  </TableBody>
</Table>
`
</script>

<template>
  <ComponentDocPage title="Table" description="A responsive table component." installCommand="npx shadcn-vue@latest add table">
    <DemoSection title="Example" id="table-basic" :autoCode="tableBasicAutoCode" :vueCode="tableBasicVueCode">
      <template #preview>
        <Table>
              <TableCaption>A list of your recent invoices.</TableCaption>
              <TableHeader>
                <TableRow>
                  <TableHead class="w-[100px]">Invoice</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Method</TableHead>
                  <TableHead class="text-right">Amount</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow>
                  <TableCell class="font-medium">INV001</TableCell>
                  <TableCell>Paid</TableCell>
                  <TableCell>Credit Card</TableCell>
                  <TableCell class="text-right">$250.00</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell class="font-medium">INV002</TableCell>
                  <TableCell>Pending</TableCell>
                  <TableCell>PayPal</TableCell>
                  <TableCell class="text-right">$150.00</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell class="font-medium">INV003</TableCell>
                  <TableCell>Unpaid</TableCell>
                  <TableCell>Bank Transfer</TableCell>
                  <TableCell class="text-right">$350.00</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell class="font-medium">INV004</TableCell>
                  <TableCell>Paid</TableCell>
                  <TableCell>Credit Card</TableCell>
                  <TableCell class="text-right">$450.00</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell class="font-medium">INV005</TableCell>
                  <TableCell>Paid</TableCell>
                  <TableCell>PayPal</TableCell>
                  <TableCell class="text-right">$550.00</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell class="font-medium">INV006</TableCell>
                  <TableCell>Pending</TableCell>
                  <TableCell>Bank Transfer</TableCell>
                  <TableCell class="text-right">$200.00</TableCell>
                </TableRow>
                <TableRow>
                  <TableCell class="font-medium">INV007</TableCell>
                  <TableCell>Unpaid</TableCell>
                  <TableCell>Credit Card</TableCell>
                  <TableCell class="text-right">$300.00</TableCell>
                </TableRow>
              </TableBody>
            </Table>
      </template>
    </DemoSection>
    <template #properties>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Component</TableHead>
            <TableHead>Description</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>Table</TableCell>
            <TableCell>Main table container</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>TableHeader</TableCell>
            <TableCell>Header section</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>TableBody</TableCell>
            <TableCell>Body section</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>TableCaption</TableCell>
            <TableCell>Table caption/description</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>TableRow</TableCell>
            <TableCell>Table row</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>TableHead</TableCell>
            <TableCell>Header cell</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>TableCell</TableCell>
            <TableCell>Data cell</TableCell>
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
