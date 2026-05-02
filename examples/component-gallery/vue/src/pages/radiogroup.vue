<!-- RadiogroupPage component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import Prism from 'prismjs'
import { Label } from '@/components/ui/label'
import { RadioGroup } from '@/components/ui/radio-group'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'


const activeTab = ref<string>('preview')

const copiedCode = ref<string>('')
const showRadiogroupBasicCode = ref<boolean>(true)
const activeRadiogroupBasicTab = ref<string>('auto')

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

const radiogroupBasicAutoCode = `radiogroup (default: "option-one") {
    row (gap: "2", class: "items-center") {
        radioitem (id: "option-one", value: "option-one") {}
        label (text: "Option One", for: "option-one") {}
    }
    row (gap: "2", class: "items-center") {
        radioitem (value: "option-two", id: "option-two") {}
        label (text: "Option Two", for: "option-two") {}
    }
}
`
const radiogroupBasicVueCode = `<RadioGroup>
  <div class="flex flex-row gap-2 items-center">
    <div id="option-one" value="option-one" />
    <Label for="option-one">Option One</Label>
  </div>
  <div class="flex flex-row gap-2 items-center">
    <div value="option-two" id="option-two" />
    <Label for="option-two">Option Two</Label>
  </div>
</RadioGroup>
`
const codeblock1Code = `npx shadcn-vue@latest add radio-group`
watch(activeRadiogroupBasicTab, () => {
  nextTick(() => Prism.highlightAll())
})
onMounted(() => {
  nextTick(() => Prism.highlightAll())
})


</script>

<template>
  <div class="flex flex-col">
    <div class="flex flex-col gap-4">
      <h1 class="text-4xl font-bold tracking-tight">RadioGroup</h1>
      A set of checkable buttons, known as radio buttons, where no more than one of the buttons can be checked at a time.
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
      <!-- Merged RadiogroupBasic Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <RadioGroup>
              <div class="flex flex-row gap-2 items-center">
                <div id="option-one" value="option-one" />
                <Label for="option-one">Option One</Label>
              </div>
              <div class="flex flex-row gap-2 items-center">
                <div value="option-two" id="option-two" />
                <Label for="option-two">Option Two</Label>
              </div>
            </RadioGroup>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showRadiogroupBasicCode = !showRadiogroupBasicCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showRadiogroupBasicCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showRadiogroupBasicCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeRadiogroupBasicTab = 'auto'"
                  :class="activeRadiogroupBasicTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeRadiogroupBasicTab = 'vue'"
                  :class="activeRadiogroupBasicTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeRadiogroupBasicTab === 'auto' ? radiogroupBasicAutoCode : radiogroupBasicVueCode, 'radiogroup-basic')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'radiogroup-basic'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'radiogroup-basic' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeRadiogroupBasicTab === 'auto' ? 'auto' : 'html')">{{ activeRadiogroupBasicTab === 'auto' ? radiogroupBasicAutoCode : radiogroupBasicVueCode }}</code></pre>
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
            <TableCell>value</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Selected radio value</TableCell>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the group</TableCell>
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


