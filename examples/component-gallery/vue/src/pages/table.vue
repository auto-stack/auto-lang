<!-- TablePage component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import Prism from 'prismjs'
import { Table, TableBody, TableCaption, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'


const activeTab = ref<string>('preview')

const copiedCode = ref<string>('')
const showTableBasicCode = ref<boolean>(true)
const activeTableBasicTab = ref<string>('auto')

// Copy to clipboard function
async function copyCode(code: string, id: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(code)
    copiedCode.value = id
    setTimeout(() => {
      copiedCode.value = ''
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

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
const codeblock1Code = `npx shadcn-vue@latest add table`
watch(activeTableBasicTab, () => {
  nextTick(() => Prism.highlightAll())
})
onMounted(() => {
  nextTick(() => Prism.highlightAll())
})


</script>

<template>
  <div class="flex flex-col">
    <div class="flex flex-col gap-4">
      <h1 class="text-4xl font-bold tracking-tight">Table</h1>
      A responsive table component.
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Installation</h2>
      <div class="relative rounded-lg border overflow-hidden">
        <div class="flex items-center justify-between px-4 py-3 bg-muted border-b">
          <span class="text-xs text-muted-foreground font-medium">bash</span>
          <button
            @click="copyCode(codeblock1Code, 'codeblock1')"
            class="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
          >
            <svg v-if="copiedCode !== 'codeblock1'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
            {{ copiedCode === 'codeblock1' ? 'Copied!' : 'Copy' }}
          </button>
        </div>
        <pre class="p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))] overflow-x-auto"><code class="block font-mono !p-0 language-bash">{{ codeblock1Code }}</code></pre>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Example</h2>
      <!-- Merged TableBasic Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
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
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showTableBasicCode = !showTableBasicCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showTableBasicCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showTableBasicCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeTableBasicTab = 'auto'"
                  :class="activeTableBasicTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeTableBasicTab = 'vue'"
                  :class="activeTableBasicTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeTableBasicTab === 'auto' ? tableBasicAutoCode : tableBasicVueCode, 'table-basic')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'table-basic'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'table-basic' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeTableBasicTab === 'auto' ? 'auto' : 'html')">{{ activeTableBasicTab === 'auto' ? tableBasicAutoCode : tableBasicVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Data Table</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">You can use the Table component to build more complex data tables. Combine it with @tanstack/vue-table to create tables with sorting, filtering and pagination.</span>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Properties</h2>
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
    </div>
  </div>

</template>

<style scoped>
/* Override Prism.js default styles */
pre[class*="language-"] {
  margin: 0;
}

/* Component styles */

</style>


