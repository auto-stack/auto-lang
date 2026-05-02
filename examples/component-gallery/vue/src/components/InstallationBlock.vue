<script setup lang="ts">
import { ref, nextTick, onMounted } from 'vue'
import Prism from 'prismjs'

const props = defineProps<{
  command: string
  id?: string
}>()

const copied = ref(false)

async function copy() {
  try {
    await navigator.clipboard.writeText(props.command)
    copied.value = true
    setTimeout(() => copied.value = false, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

onMounted(() => {
  nextTick(() => Prism.highlightAll())
})
</script>

<template>
  <div class="relative rounded-xl border overflow-hidden">
    <div class="flex items-center justify-between px-4 py-3 bg-muted border-b">
      <span class="text-xs text-muted-foreground font-medium">bash</span>
      <button
        @click="copy"
        class="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
      >
        <svg v-if="!copied" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
        <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
        {{ copied ? 'Copied!' : 'Copy' }}
      </button>
    </div>
    <pre class="p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))] overflow-x-auto"><code class="block font-mono !p-0 language-bash">{{ command }}</code></pre>
  </div>
</template>
