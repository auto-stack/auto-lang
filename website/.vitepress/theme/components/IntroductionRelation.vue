<script setup lang="ts">
import { computed } from 'vue'
import { useData } from 'vitepress'
import { introCopy } from '../data/os-ai-introduction'
const props = defineProps<{ kind: 'os' | 'ai' }>()
const { lang } = useData()
const c = computed(() => introCopy(lang.value.startsWith('zh')).sections[props.kind])
</script>

<template>
    <figure class="system-relation">
      <ol :aria-label="c.diagramLabel">
        <li v-for="layer in c.layers" :key="layer">{{ layer }}</li>
      </ol>
      <figcaption>{{ c.diagramNote }}</figcaption>
    </figure>
</template>

<style scoped>
.system-relation { margin: 1.5rem 0 2rem; }
.system-relation ol { padding: 0; margin: 0; list-style: none; }
.system-relation li { position: relative; padding: .65rem 1rem; margin: 0 0 1.35rem; border: 1px solid var(--vp-c-divider); border-radius: var(--site-radius, 12px); background: var(--vp-c-bg-soft); overflow-wrap: anywhere; }
.system-relation li:not(:last-child)::after { content: '↓'; position: absolute; top: 100%; left: 50%; color: var(--vp-c-text-2); line-height: 1.35rem; }
figcaption { color: var(--vp-c-text-2); font-size: .85rem; line-height: 1.7; }
</style>
