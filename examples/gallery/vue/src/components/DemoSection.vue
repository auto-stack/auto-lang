<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import Prism from 'prismjs'

const props = defineProps<{
  title: string
  autoCode?: string
  vueCode?: string
  id: string
}>()

const showCode = ref(true)
const activeTab = ref<'auto' | 'vue'>('auto')
const copied = ref(false)

async function copy() {
  const code = activeTab.value === 'auto' ? (props.autoCode ?? '') : (props.vueCode ?? '')
  try {
    await navigator.clipboard.writeText(code)
    copied.value = true
    setTimeout(() => copied.value = false, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

watch(activeTab, () => {
  nextTick(() => Prism.highlightAll())
})

onMounted(() => {
  nextTick(() => Prism.highlightAll())
})
</script>

<template>
  <div class="flex flex-col gap-3 mb-10">
    <!-- Demo Title -->
    <h2 class="text-xl md:text-2xl font-semibold tracking-tight flex items-center gap-2">
      <span class="w-1 h-5 rounded-full bg-primary/60 inline-block" />
      {{ title }}
    </h2>

    <!-- Demo Card -->
    <div class="rounded-xl border overflow-hidden">
      <!-- Preview Area -->
      <div class="flex items-center justify-center p-4 md:p-6 min-h-[120px] bg-[hsl(var(--preview-bg))]">
        <slot name="preview" />
      </div>

      <!-- Toggle Code Footer -->
      <div class="border-t">
        <button
          @click="showCode = !showCode"
          class="flex w-full items-center justify-between px-4 py-2.5 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
        >
          <span class="font-medium">Code</span>
          <svg
            :class="showCode ? 'rotate-180' : ''"
            xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
            class="transition-transform duration-200"
          >
            <path d="m6 9 6 6 6-6"/>
          </svg>
        </button>

        <!-- Expandable Code Block -->
        <div v-if="showCode" class="border-t">
          <!-- Tabs -->
          <div class="flex items-center justify-between bg-muted">
            <div class="flex">
              <button
                @click="activeTab = 'auto'"
                :class="activeTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                class="px-4 py-2 text-xs font-medium transition-colors"
              >
                Auto
              </button>
              <button
                @click="activeTab = 'vue'"
                :class="activeTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                class="px-4 py-2 text-xs font-medium transition-colors"
              >
                Vue
              </button>
            </div>
            <button
              @click="copy"
              class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
            >
              <svg v-if="!copied" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
              {{ copied ? 'Copied!' : 'Copy' }}
            </button>
          </div>
          <!-- Code content -->
          <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeTab === 'auto' ? 'auto' : 'html')">{{ activeTab === 'auto' ? autoCode : vueCode }}</code></pre>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
pre[class*="language-"] {
  margin: 0;
}
</style>
