<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import Prism from 'prismjs'
import 'prismjs/components/prism-clike'
import 'prismjs/components/prism-javascript'
import 'prismjs/components/prism-typescript'
import 'prismjs/components/prism-css'
import 'prismjs/themes/prism-tomorrow.css'
import { blocks } from '../blocks'

const route = useRoute()
const block = computed(
  () =>
    blocks.find((b) => b.kind === route.params.kind && b.name === route.params.name),
)

// spec.md = `+++` frontmatter + NL body; show only the body here.
const specBody = computed(() => {
  const s = block.value?.spec ?? ''
  const close = s.indexOf('\n+++')
  if (!s.startsWith('+++') || close < 0) return s
  return s.slice(close + 4).replace(/^\r?\n/, '')
})

const variantNames = computed(() => Object.keys(block.value?.references ?? {}))
const activeVariant = ref<string | null>(null)
const currentVariant = computed(
  () => activeVariant.value ?? variantNames.value[0] ?? '',
)
const referenceSource = computed(
  () => block.value?.references[currentVariant.value] ?? '',
)

const highlighted = computed(() =>
  Prism.highlight(referenceSource.value, Prism.languages.markup, 'markup'),
)
</script>

<template>
  <div v-if="block">
    <h2 class="page-title">{{ block.kind }}/{{ block.name }}</h2>

    <section class="block-section">
      <h3 class="section-title">Spec</h3>
      <pre class="spec-body">{{ specBody }}</pre>
    </section>

    <section class="block-section">
      <h3 class="section-title">Reference implementation</h3>
      <div v-if="variantNames.length > 1" class="variant-tabs">
        <button
          v-for="v in variantNames"
          :key="v"
          type="button"
          class="variant-tab"
          :class="{ active: v === currentVariant }"
          @click="activeVariant = v"
        >
          {{ v }}
        </button>
      </div>
      <p class="hint">
        Source of <code>{{ currentVariant }}.at</code>. Live render via a2vue is a
        follow-up; for now the authored source is shown.
      </p>
      <div class="code-wrap">
        <pre class="code"><code v-html="highlighted"></code></pre>
      </div>
    </section>

    <section class="block-section">
      <h3 class="section-title">Gotchas</h3>
      <pre class="gotchas-body">{{ block.gotchas }}</pre>
    </section>
  </div>
  <div v-else>
    <p>Block not found.</p>
  </div>
</template>
