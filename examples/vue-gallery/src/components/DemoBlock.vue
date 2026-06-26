<script setup lang="ts">
import { computed, ref } from 'vue'
import Prism from 'prismjs'
import 'prismjs/components/prism-clike'
import 'prismjs/components/prism-javascript'
import 'prismjs/components/prism-typescript'
import 'prismjs/components/prism-css'
import 'prismjs/themes/prism-tomorrow.css'

// Showcase shell: one bordered card with the live preview on top and the Vue
// source below (collapsible, open by default). Mirrors examples/gallery's
// DemoSection layout, minus the Auto/Vue toggle (vue-gallery only ships Vue).
const props = defineProps<{
  title: string
  description?: string
  code?: string
}>()

const open = ref(true)
const copied = ref(false)

// Highlight the SFC snippet with the markup grammar, which auto-highlights
// JS inside <script> and CSS inside <style> — exactly right for a .vue file.
const highlighted = computed(() =>
  Prism.highlight(props.code ?? '', Prism.languages.markup, 'markup'),
)

async function copy() {
  try {
    await navigator.clipboard.writeText(props.code ?? '')
    copied.value = true
    setTimeout(() => (copied.value = false), 1500)
  } catch {
    /* clipboard unavailable */
  }
}
</script>

<template>
  <section class="demo-block">
    <header class="demo-head">
      <h3 class="demo-title">{{ title }}</h3>
      <p v-if="description" class="demo-desc">{{ description }}</p>
    </header>

    <div class="demo-card">
      <!-- Live preview -->
      <div class="demo-preview">
        <slot />
      </div>

      <!-- Code section (collapsible, open by default) -->
      <button v-if="code" type="button" class="demo-code-bar" @click="open = !open">
        <span class="demo-code-label">Vue</span>
        <svg
          class="demo-chevron"
          :class="{ rotated: open }"
          xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"
          stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
        ><path d="m6 9 6 6 6-6" /></svg>
      </button>

      <div v-if="code && open" class="demo-code-wrap">
        <button type="button" class="demo-copy" @click="copy">
          {{ copied ? 'Copied' : 'Copy' }}
        </button>
        <pre class="demo-code"><code v-html="highlighted"></code></pre>
      </div>
    </div>
  </section>
</template>
