<!-- AspectratioPage component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import Prism from 'prismjs'
import { AspectRatio } from '@/components/ui/aspect-ratio'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'


const activeTab = ref<string>('preview')

const copiedCode = ref<string>('')
const showAspectratioBasicCode = ref<boolean>(true)
const activeAspectratioBasicTab = ref<string>('auto')
const showAspectratioRatiosCode = ref<boolean>(true)
const activeAspectratioRatiosTab = ref<string>('auto')

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

const aspectratioBasicAutoCode = `div (class: "w-[450px] bg-muted rounded-md overflow-hidden border") {
    aspect-ratio (ratio: 1.777) {
        img (class: "object-cover w-full h-full", src: "https://images.unsplash.com/photo-1588345921523-c2dcdb7f1dcd?w=800&dpr=2&q=80", alt: "Photo by Drew Beamer") {}
    }
}
`
const aspectratioBasicVueCode = `<div class="w-[450px] bg-muted rounded-md overflow-hidden border">
  <AspectRatio>
    <img class="object-cover w-full h-full" src="https://images.unsplash.com/photo-1588345921523-c2dcdb7f1dcd?w=800&dpr=2&q=80" alt="Photo by Drew Beamer" />
  </AspectRatio>
</div>
`
const aspectratioRatiosAutoCode = `row (gap: "4", class: "flex-wrap") {
    col (class: "w-[150px]", gap: "2") {
        div (class: "bg-muted rounded-md overflow-hidden") {
            aspect-ratio (ratio: 1) {
                div (class: "flex items-center justify-center w-full h-full bg-primary/10") {
                    "1:1"
                }
            }
        }
        text (text: "Square (1:1)", class: "text-sm text-muted-foreground") {}
    }
    col (class: "w-[150px]", gap: "2") {
        div (class: "bg-muted rounded-md overflow-hidden") {
            aspect-ratio (ratio: 1.777) {
                div (class: "flex items-center justify-center w-full h-full bg-primary/10") {
                    "16:9"
                }
            }
        }
        text (class: "text-sm text-muted-foreground", text: "Widescreen (16:9)") {}
    }
    col (gap: "2", class: "w-[150px]") {
        div (class: "bg-muted rounded-md overflow-hidden") {
            aspect-ratio (ratio: 0.5625) {
                div (class: "flex items-center justify-center w-full h-full bg-primary/10") {
                    "9:16"
                }
            }
        }
        text (class: "text-sm text-muted-foreground", text: "Portrait (9:16)") {}
    }
    col (gap: "2", class: "w-[150px]") {
        div (class: "bg-muted rounded-md overflow-hidden") {
            aspect-ratio (ratio: 1.333) {
                div (class: "flex items-center justify-center w-full h-full bg-primary/10") {
                    "4:3"
                }
            }
        }
        text (text: "Standard (4:3)", class: "text-sm text-muted-foreground") {}
    }
}
`
const aspectratioRatiosVueCode = `<div class="flex flex-row gap-4 flex-wrap">
  <div class="flex flex-col gap-2 w-[150px]">
    <div class="bg-muted rounded-md overflow-hidden">
      <AspectRatio>
        <div class="flex items-center justify-center w-full h-full bg-primary/10">
          1:1
        </div>
      </AspectRatio>
    </div>
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Square (1:1)</span>
  </div>
  <div class="flex flex-col gap-2 w-[150px]">
    <div class="bg-muted rounded-md overflow-hidden">
      <AspectRatio>
        <div class="flex items-center justify-center w-full h-full bg-primary/10">
          16:9
        </div>
      </AspectRatio>
    </div>
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Widescreen (16:9)</span>
  </div>
  <div class="flex flex-col gap-2 w-[150px]">
    <div class="bg-muted rounded-md overflow-hidden">
      <AspectRatio>
        <div class="flex items-center justify-center w-full h-full bg-primary/10">
          9:16
        </div>
      </AspectRatio>
    </div>
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Portrait (9:16)</span>
  </div>
  <div class="flex flex-col gap-2 w-[150px]">
    <div class="bg-muted rounded-md overflow-hidden">
      <AspectRatio>
        <div class="flex items-center justify-center w-full h-full bg-primary/10">
          4:3
        </div>
      </AspectRatio>
    </div>
    <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Standard (4:3)</span>
  </div>
</div>
`
const codeblock1Code = `npx shadcn-vue@latest add aspect-ratio`
watch(activeAspectratioBasicTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeAspectratioRatiosTab, () => {
  nextTick(() => Prism.highlightAll())
})
onMounted(() => {
  nextTick(() => Prism.highlightAll())
})


</script>

<template>
  <div class="flex flex-col">
    <div class="flex flex-col gap-4">
      <h1 class="text-4xl font-bold tracking-tight">AspectRatio</h1>
      Displays content within a desired ratio.
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
      <!-- Merged AspectratioBasic Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <div class="w-[450px] bg-muted rounded-md overflow-hidden border">
              <AspectRatio>
                <img class="object-cover w-full h-full" src="https://images.unsplash.com/photo-1588345921523-c2dcdb7f1dcd?w=800&dpr=2&q=80" alt="Photo by Drew Beamer" />
              </AspectRatio>
            </div>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showAspectratioBasicCode = !showAspectratioBasicCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showAspectratioBasicCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showAspectratioBasicCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeAspectratioBasicTab = 'auto'"
                  :class="activeAspectratioBasicTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeAspectratioBasicTab = 'vue'"
                  :class="activeAspectratioBasicTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeAspectratioBasicTab === 'auto' ? aspectratioBasicAutoCode : aspectratioBasicVueCode, 'aspectratio-basic')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'aspectratio-basic'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'aspectratio-basic' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeAspectratioBasicTab === 'auto' ? 'auto' : 'html')">{{ activeAspectratioBasicTab === 'auto' ? aspectratioBasicAutoCode : aspectratioBasicVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Common Ratios</h2>
      <!-- Merged AspectratioRatios Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <div class="flex flex-row gap-4 flex-wrap">
              <div class="flex flex-col gap-2 w-[150px]">
                <div class="bg-muted rounded-md overflow-hidden">
                  <AspectRatio>
                    <div class="flex items-center justify-center w-full h-full bg-primary/10">
                      1:1
                    </div>
                  </AspectRatio>
                </div>
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Square (1:1)</span>
              </div>
              <div class="flex flex-col gap-2 w-[150px]">
                <div class="bg-muted rounded-md overflow-hidden">
                  <AspectRatio>
                    <div class="flex items-center justify-center w-full h-full bg-primary/10">
                      16:9
                    </div>
                  </AspectRatio>
                </div>
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Widescreen (16:9)</span>
              </div>
              <div class="flex flex-col gap-2 w-[150px]">
                <div class="bg-muted rounded-md overflow-hidden">
                  <AspectRatio>
                    <div class="flex items-center justify-center w-full h-full bg-primary/10">
                      9:16
                    </div>
                  </AspectRatio>
                </div>
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Portrait (9:16)</span>
              </div>
              <div class="flex flex-col gap-2 w-[150px]">
                <div class="bg-muted rounded-md overflow-hidden">
                  <AspectRatio>
                    <div class="flex items-center justify-center w-full h-full bg-primary/10">
                      4:3
                    </div>
                  </AspectRatio>
                </div>
                <span class="text-muted-foreground leading-7 text-sm text-muted-foreground">Standard (4:3)</span>
              </div>
            </div>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showAspectratioRatiosCode = !showAspectratioRatiosCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showAspectratioRatiosCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showAspectratioRatiosCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeAspectratioRatiosTab = 'auto'"
                  :class="activeAspectratioRatiosTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeAspectratioRatiosTab = 'vue'"
                  :class="activeAspectratioRatiosTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeAspectratioRatiosTab === 'auto' ? aspectratioRatiosAutoCode : aspectratioRatiosVueCode, 'aspectratio-ratios')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'aspectratio-ratios'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'aspectratio-ratios' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeAspectratioRatiosTab === 'auto' ? 'auto' : 'html')">{{ activeAspectratioRatiosTab === 'auto' ? aspectratioRatiosAutoCode : aspectratioRatiosVueCode }}</code></pre>
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
            <TableCell>ratio</TableCell>
            <TableCell>number</TableCell>
            <TableCell>1</TableCell>
            <TableCell>-</TableCell>
            <TableCell>The aspect ratio (width/height)</TableCell>
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


