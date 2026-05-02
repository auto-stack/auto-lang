<!-- SkeletonPage component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import Prism from 'prismjs'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'


const activeTab = ref<string>('preview')

const copiedCode = ref<string>('')
const showSkeletonBasicCode = ref<boolean>(true)
const activeSkeletonBasicTab = ref<string>('auto')
const showSkeletonCardCode = ref<boolean>(true)
const activeSkeletonCardTab = ref<string>('auto')

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

const skeletonBasicAutoCode = `skeleton (class: "w-48 h-4") {}
`
const skeletonBasicVueCode = `<div class="w-48 h-4" />
`
const skeletonCardAutoCode = `col (class: "w-64", gap: "4") {
    skeleton (class: "h-12 w-12 rounded-full") {}
    col (gap: "2") {
        skeleton (class: "h-4 w-full") {}
        skeleton (class: "h-4 w-3/4") {}
    }
}
`
const skeletonCardVueCode = `<div class="flex flex-col gap-4 w-64">
  <div class="h-12 w-12 rounded-full" />
  <div class="flex flex-col gap-2">
    <div class="h-4 w-full" />
    <div class="h-4 w-3/4" />
  </div>
</div>
`
const codeblock1Code = `npx shadcn-vue@latest add skeleton`
watch(activeSkeletonBasicTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeSkeletonCardTab, () => {
  nextTick(() => Prism.highlightAll())
})
onMounted(() => {
  nextTick(() => Prism.highlightAll())
})


</script>

<template>
  <div class="flex flex-col">
    <div class="flex flex-col gap-4">
      <h1 class="text-4xl font-bold tracking-tight">Skeleton</h1>
      Use to show a placeholder while content is loading.
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
      <!-- Merged SkeletonBasic Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <div class="w-48 h-4" />
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showSkeletonBasicCode = !showSkeletonBasicCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showSkeletonBasicCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showSkeletonBasicCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeSkeletonBasicTab = 'auto'"
                  :class="activeSkeletonBasicTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeSkeletonBasicTab = 'vue'"
                  :class="activeSkeletonBasicTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeSkeletonBasicTab === 'auto' ? skeletonBasicAutoCode : skeletonBasicVueCode, 'skeleton-basic')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'skeleton-basic'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'skeleton-basic' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeSkeletonBasicTab === 'auto' ? 'auto' : 'html')">{{ activeSkeletonBasicTab === 'auto' ? skeletonBasicAutoCode : skeletonBasicVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Card Skeleton</h2>
      <!-- Merged SkeletonCard Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <div class="flex flex-col gap-4 w-64">
              <div class="h-12 w-12 rounded-full" />
              <div class="flex flex-col gap-2">
                <div class="h-4 w-full" />
                <div class="h-4 w-3/4" />
              </div>
            </div>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showSkeletonCardCode = !showSkeletonCardCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showSkeletonCardCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showSkeletonCardCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeSkeletonCardTab = 'auto'"
                  :class="activeSkeletonCardTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeSkeletonCardTab = 'vue'"
                  :class="activeSkeletonCardTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeSkeletonCardTab === 'auto' ? skeletonCardAutoCode : skeletonCardVueCode, 'skeleton-card')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'skeleton-card'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'skeleton-card' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeSkeletonCardTab === 'auto' ? 'auto' : 'html')">{{ activeSkeletonCardTab === 'auto' ? skeletonCardAutoCode : skeletonCardVueCode }}</code></pre>
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
            <TableCell>class</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Tailwind classes for sizing</TableCell>
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


