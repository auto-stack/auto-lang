<!-- SelectPage component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import Prism from 'prismjs'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Select } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'


const activeTab = ref<string>('preview')

const copiedCode = ref<string>('')
const showSelectBasicCode = ref<boolean>(true)
const activeSelectBasicTab = ref<string>('auto')
const showSelectScrollableCode = ref<boolean>(true)
const activeSelectScrollableTab = ref<string>('auto')

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
const codeblock1Code = `npx shadcn-vue@latest add select`
watch(activeSelectBasicTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeSelectScrollableTab, () => {
  nextTick(() => Prism.highlightAll())
})
onMounted(() => {
  nextTick(() => Prism.highlightAll())
})


</script>

<template>
  <div class="flex flex-col">
    <div class="flex flex-col gap-4">
      <h1 class="text-4xl font-bold tracking-tight">Select</h1>
      Displays a list of options for the user to pick from.
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
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Simple</h2>
      <!-- Merged SelectBasic Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
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
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showSelectBasicCode = !showSelectBasicCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showSelectBasicCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showSelectBasicCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeSelectBasicTab = 'auto'"
                  :class="activeSelectBasicTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeSelectBasicTab = 'vue'"
                  :class="activeSelectBasicTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeSelectBasicTab === 'auto' ? selectBasicAutoCode : selectBasicVueCode, 'select-basic')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'select-basic'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'select-basic' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeSelectBasicTab === 'auto' ? 'auto' : 'html')">{{ activeSelectBasicTab === 'auto' ? selectBasicAutoCode : selectBasicVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Scrollable</h2>
      <!-- Merged SelectScrollable Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
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
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showSelectScrollableCode = !showSelectScrollableCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showSelectScrollableCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showSelectScrollableCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeSelectScrollableTab = 'auto'"
                  :class="activeSelectScrollableTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeSelectScrollableTab = 'vue'"
                  :class="activeSelectScrollableTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeSelectScrollableTab === 'auto' ? selectScrollableAutoCode : selectScrollableVueCode, 'select-scrollable')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'select-scrollable'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'select-scrollable' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeSelectScrollableTab === 'auto' ? 'auto' : 'html')">{{ activeSelectScrollableTab === 'auto' ? selectScrollableAutoCode : selectScrollableVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Properties</h2>
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


