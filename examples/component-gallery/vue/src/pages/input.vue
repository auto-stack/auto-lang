<!-- InputPage component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import Prism from 'prismjs'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'


const activeTab = ref<string>('preview')

const copiedCode = ref<string>('')
const showInputBasicCode = ref<boolean>(true)
const activeInputBasicTab = ref<string>('auto')
const showInputFileCode = ref<boolean>(true)
const activeInputFileTab = ref<string>('auto')
const showInputDisabledCode = ref<boolean>(true)
const activeInputDisabledTab = ref<string>('auto')
const showInputLabelCode = ref<boolean>(true)
const activeInputLabelTab = ref<string>('auto')
const showInputButtonCode = ref<boolean>(true)
const activeInputButtonTab = ref<string>('auto')

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

const inputBasicAutoCode = `input (placeholder: "Email", type: "email") {}
`
const inputBasicVueCode = `<Input type="email" placeholder="Email" />
`
const inputFileAutoCode = `col (gap: "1.5", class: "w-full max-w-sm") {
    label (text: "Picture", for: "picture") {}
    input (id: "picture", type: "file") {}
}
`
const inputFileVueCode = `<div class="flex flex-col gap-1.5 w-full max-w-sm">
  <Label for="picture">Picture</Label>
  <Input type="file" />
</div>
`
const inputDisabledAutoCode = `input (type: "email", disabled: "true", placeholder: "Email") {}
`
const inputDisabledVueCode = `<Input type="email" placeholder="Email" disabled />
`
const inputLabelAutoCode = `col (class: "w-full max-w-sm", gap: "1.5") {
    label (for: "email", text: "Email") {}
    input (placeholder: "Email", id: "email", type: "email") {}
}
`
const inputLabelVueCode = `<div class="flex flex-col gap-1.5 w-full max-w-sm">
  <Label for="email">Email</Label>
  <Input type="email" placeholder="Email" />
</div>
`
const inputButtonAutoCode = `row (gap: "2", class: "w-full max-w-sm items-center") {
    input (placeholder: "Email", type: "email") {}
    button (text: "Subscribe", type: "submit") {}
}
`
const inputButtonVueCode = `<div class="flex flex-row gap-2 w-full max-w-sm items-center">
  <Input type="email" placeholder="Email" />
  <Button>Subscribe</Button>
</div>
`
const codeblock1Code = `npx shadcn-vue@latest add input`
watch(activeInputBasicTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeInputFileTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeInputDisabledTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeInputLabelTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeInputButtonTab, () => {
  nextTick(() => Prism.highlightAll())
})
onMounted(() => {
  nextTick(() => Prism.highlightAll())
})


</script>

<template>
  <div class="flex flex-col">
    <div class="flex flex-col gap-4">
      <h1 class="text-4xl font-bold tracking-tight">Input</h1>
      Displays a form input field or a component that looks like an input field.
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
      <!-- Merged InputBasic Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <Input type="email" placeholder="Email" />
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showInputBasicCode = !showInputBasicCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showInputBasicCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showInputBasicCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeInputBasicTab = 'auto'"
                  :class="activeInputBasicTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeInputBasicTab = 'vue'"
                  :class="activeInputBasicTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeInputBasicTab === 'auto' ? inputBasicAutoCode : inputBasicVueCode, 'input-basic')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'input-basic'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'input-basic' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeInputBasicTab === 'auto' ? 'auto' : 'html')">{{ activeInputBasicTab === 'auto' ? inputBasicAutoCode : inputBasicVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Examples</h2>
      <h3 class="text-xl font-semibold">File</h3>
      <!-- Merged InputFile Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <div class="flex flex-col gap-1.5 w-full max-w-sm">
              <Label for="picture">Picture</Label>
              <Input type="file" />
            </div>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showInputFileCode = !showInputFileCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showInputFileCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showInputFileCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeInputFileTab = 'auto'"
                  :class="activeInputFileTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeInputFileTab = 'vue'"
                  :class="activeInputFileTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeInputFileTab === 'auto' ? inputFileAutoCode : inputFileVueCode, 'input-file')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'input-file'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'input-file' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeInputFileTab === 'auto' ? 'auto' : 'html')">{{ activeInputFileTab === 'auto' ? inputFileAutoCode : inputFileVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h3 class="text-xl font-semibold">Disabled</h3>
      <!-- Merged InputDisabled Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <Input type="email" placeholder="Email" disabled />
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showInputDisabledCode = !showInputDisabledCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showInputDisabledCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showInputDisabledCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeInputDisabledTab = 'auto'"
                  :class="activeInputDisabledTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeInputDisabledTab = 'vue'"
                  :class="activeInputDisabledTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeInputDisabledTab === 'auto' ? inputDisabledAutoCode : inputDisabledVueCode, 'input-disabled')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'input-disabled'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'input-disabled' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeInputDisabledTab === 'auto' ? 'auto' : 'html')">{{ activeInputDisabledTab === 'auto' ? inputDisabledAutoCode : inputDisabledVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h3 class="text-xl font-semibold">With Label</h3>
      <!-- Merged InputLabel Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <div class="flex flex-col gap-1.5 w-full max-w-sm">
              <Label for="email">Email</Label>
              <Input type="email" placeholder="Email" />
            </div>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showInputLabelCode = !showInputLabelCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showInputLabelCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showInputLabelCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeInputLabelTab = 'auto'"
                  :class="activeInputLabelTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeInputLabelTab = 'vue'"
                  :class="activeInputLabelTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeInputLabelTab === 'auto' ? inputLabelAutoCode : inputLabelVueCode, 'input-label')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'input-label'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'input-label' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeInputLabelTab === 'auto' ? 'auto' : 'html')">{{ activeInputLabelTab === 'auto' ? inputLabelAutoCode : inputLabelVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h3 class="text-xl font-semibold">With Button</h3>
      <!-- Merged InputButton Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <div class="flex flex-row gap-2 w-full max-w-sm items-center">
              <Input type="email" placeholder="Email" />
              <Button>Subscribe</Button>
            </div>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showInputButtonCode = !showInputButtonCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showInputButtonCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showInputButtonCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeInputButtonTab = 'auto'"
                  :class="activeInputButtonTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeInputButtonTab = 'vue'"
                  :class="activeInputButtonTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeInputButtonTab === 'auto' ? inputButtonAutoCode : inputButtonVueCode, 'input-button')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'input-button'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'input-button' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeInputButtonTab === 'auto' ? 'auto' : 'html')">{{ activeInputButtonTab === 'auto' ? inputButtonAutoCode : inputButtonVueCode }}</code></pre>
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
            <TableCell>type</TableCell>
            <TableCell>string</TableCell>
            <TableCell>text</TableCell>
            <TableCell>text, email, password, file, number, etc.</TableCell>
            <TableCell>Input type attribute</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>placeholder</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Placeholder text shown when input is empty</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the input</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>id</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Unique identifier for associating with labels</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>modelValue</TableCell>
            <TableCell>string | number</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>The value of the input (v-model binding)</TableCell>
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


