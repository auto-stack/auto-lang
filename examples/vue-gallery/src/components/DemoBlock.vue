<script setup lang="ts">
import { ref } from 'vue'

// Showcase shell: title + description + Preview/Code toggle (shadcn-style).
// Preview = live slot; Code = the Vue snippet that produces it, with copy.
const props = defineProps<{
  title: string
  description?: string
  code?: string
}>()

const tab = ref<'preview' | 'code'>('preview')
const copied = ref(false)

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
    <div class="demo-head">
      <div>
        <h3 class="demo-title">{{ title }}</h3>
        <p v-if="description" class="demo-desc">{{ description }}</p>
      </div>
      <div v-if="code" class="demo-tabs">
        <button
          type="button"
          class="demo-tab"
          :class="{ active: tab === 'preview' }"
          @click="tab = 'preview'"
        >
          Preview
        </button>
        <button
          type="button"
          class="demo-tab"
          :class="{ active: tab === 'code' }"
          @click="tab = 'code'"
        >
          Code
        </button>
      </div>
    </div>

    <div v-show="tab === 'preview'" class="demo-preview">
      <slot />
    </div>

    <div v-if="code && tab === 'code'" class="demo-code-wrap">
      <button type="button" class="demo-copy" @click="copy">
        {{ copied ? 'Copied' : 'Copy' }}
      </button>
      <pre class="demo-code"><code>{{ code }}</code></pre>
    </div>
  </section>
</template>
