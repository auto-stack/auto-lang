<!-- ContextmenuPage component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import Prism from 'prismjs'
import { ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuSeparator, ContextMenuShortcut, ContextMenuTrigger } from '@/components/ui/context-menu'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'


const activeTab = ref<string>('preview')

const copiedCode = ref<string>('')
const showContextmenuBasicCode = ref<boolean>(true)
const activeContextmenuBasicTab = ref<string>('auto')

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

const contextmenuBasicAutoCode = `context-menu {
    context-menu-trigger (class: "flex h-[149px] w-[300px] items-center justify-center rounded-md border border-dashed text-sm") {
        "Right click here"
    }
    context-menu-content (class: "w-53") {
        context-menu-item (text: "Back") {
            context-menu-shortcut (text: "⌘[") {}
        }
        context-menu-item (disabled: "true", text: "Forward") {
            context-menu-shortcut (text: "⌘]") {}
        }
        context-menu-item (text: "Reload") {
            context-menu-shortcut (text: "⌘R") {}
        }
        context-menu-separator {}
        context-menu-item (text: "Profile") {}
        context-menu-item (text: "Billing") {}
        context-menu-item (text: "Team") {}
        context-menu-separator {}
        context-menu-item (text: "Subscription") {}
    }
}
`
const contextmenuBasicVueCode = `<ContextMenu>
  <ContextMenuTrigger class="flex h-[149px] w-[300px] items-center justify-center rounded-md border border-dashed text-sm">
    Right click here
  </ContextMenuTrigger>
  <ContextMenuContent class="w-53">
    <ContextMenuItem>Back
      <ContextMenuShortcut>⌘[</ContextMenuShortcut>
    </ContextMenuItem>
    <ContextMenuItem disabled>Forward
      <ContextMenuShortcut>⌘]</ContextMenuShortcut>
    </ContextMenuItem>
    <ContextMenuItem>Reload
      <ContextMenuShortcut>⌘R</ContextMenuShortcut>
    </ContextMenuItem>
    <ContextMenuSeparator />
    <ContextMenuItem>Profile</ContextMenuItem>
    <ContextMenuItem>Billing</ContextMenuItem>
    <ContextMenuItem>Team</ContextMenuItem>
    <ContextMenuSeparator />
    <ContextMenuItem>Subscription</ContextMenuItem>
  </ContextMenuContent>
</ContextMenu>
`
const codeblock1Code = `npx shadcn-vue@latest add context-menu`
watch(activeContextmenuBasicTab, () => {
  nextTick(() => Prism.highlightAll())
})
onMounted(() => {
  nextTick(() => Prism.highlightAll())
})


</script>

<template>
  <div class="flex flex-col">
    <div class="flex flex-col gap-4">
      <h1 class="text-4xl font-bold tracking-tight">ContextMenu</h1>
      Displays a menu to the user — such as a set of actions or functions — triggered by a right click.
      <h1 class="text-4xl font-bold tracking-tight">Installation</h1>
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
      <h1 class="text-4xl font-bold tracking-tight">Example</h1>
      <!-- Merged ContextmenuBasic Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
                      <ContextMenu>
              <ContextMenuTrigger class="flex h-[149px] w-[300px] items-center justify-center rounded-md border border-dashed text-sm">
                Right click here
              </ContextMenuTrigger>
              <ContextMenuContent class="w-53">
                <ContextMenuItem>Back
                  <ContextMenuShortcut>⌘[</ContextMenuShortcut>
                </ContextMenuItem>
                <ContextMenuItem disabled>Forward
                  <ContextMenuShortcut>⌘]</ContextMenuShortcut>
                </ContextMenuItem>
                <ContextMenuItem>Reload
                  <ContextMenuShortcut>⌘R</ContextMenuShortcut>
                </ContextMenuItem>
                <ContextMenuSeparator />
                <ContextMenuItem>Profile</ContextMenuItem>
                <ContextMenuItem>Billing</ContextMenuItem>
                <ContextMenuItem>Team</ContextMenuItem>
                <ContextMenuSeparator />
                <ContextMenuItem>Subscription</ContextMenuItem>
              </ContextMenuContent>
            </ContextMenu>
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showContextmenuBasicCode = !showContextmenuBasicCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showContextmenuBasicCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showContextmenuBasicCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeContextmenuBasicTab = 'auto'"
                  :class="activeContextmenuBasicTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeContextmenuBasicTab = 'vue'"
                  :class="activeContextmenuBasicTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeContextmenuBasicTab === 'auto' ? contextmenuBasicAutoCode : contextmenuBasicVueCode, 'contextmenu-basic')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'contextmenu-basic'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'contextmenu-basic' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeContextmenuBasicTab === 'auto' ? 'auto' : 'html')">{{ activeContextmenuBasicTab === 'auto' ? contextmenuBasicAutoCode : contextmenuBasicVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h1 class="text-4xl font-bold tracking-tight">Properties</h1>
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
            <TableCell>text</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Menu item text</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>disabled</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Disables the menu item</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>shortcut</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Keyboard shortcut hint</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>inset</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Adds inset padding</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>variant</TableCell>
            <TableCell>string</TableCell>
            <TableCell>default</TableCell>
            <TableCell>default, destructive</TableCell>
            <TableCell>Item style variant</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>class</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Additional CSS classes</TableCell>
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


