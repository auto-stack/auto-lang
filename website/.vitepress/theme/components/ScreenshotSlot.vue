<script setup lang="ts">
import { computed } from 'vue'
import { useData } from 'vitepress'
import { CAPTURE_SLOTS } from '../data/capture-slots'

const props = defineProps<{ captureId: string }>()
const { lang } = useData()
const zh = computed(() => lang.value.startsWith('zh'))
const copy = computed(() => CAPTURE_SLOTS[props.captureId])
</script>

<template>
  <aside v-if="copy" class="screenshot-slot" :data-capture-id="captureId" :aria-label="copy.title[zh ? 1 : 0]">
    <span class="slot-state">{{ zh ? '截图待补' : 'Capture pending' }} · {{ captureId }}</span>
    <strong>{{ copy.title[zh ? 1 : 0] }}</strong>
    <p>{{ copy.scene[zh ? 1 : 0] }}</p>
  </aside>
</template>

<style scoped>
.screenshot-slot { box-sizing: border-box; width: 100%; min-width: 0; min-height: 12rem; padding: 1.5rem; border: 1px dashed var(--vp-c-divider); border-radius: var(--site-radius, 12px); background: var(--vp-c-bg-soft); color: var(--vp-c-text-2); display: flex; flex-direction: column; justify-content: center; gap: .65rem; overflow-wrap: anywhere; }
.slot-state { font-size: .8rem; }
strong { color: var(--vp-c-text-1); font-size: 1rem; }
p { margin: 0; line-height: 1.7; font-size: .9rem; }
</style>
