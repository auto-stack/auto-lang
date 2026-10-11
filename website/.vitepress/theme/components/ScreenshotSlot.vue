<script setup lang="ts">
import { computed } from 'vue'
import { useData } from 'vitepress'
import { CAPTURE_IMAGES, CAPTURE_SLOTS } from '../data/capture-slots'
import EvidenceImage from './EvidenceImage.vue'

const props = withDefaults(defineProps<{ captureId: string; loading?: 'lazy' | 'eager' }>(), { loading: 'lazy' })
const { lang } = useData()
const zh = computed(() => lang.value.startsWith('zh'))
const copy = computed(() => CAPTURE_SLOTS[props.captureId])
const capture = computed(() => CAPTURE_IMAGES[props.captureId])
</script>

<template>
  <div v-if="copy && capture" :data-capture-id="captureId" class="completed-capture">
    <EvidenceImage :src="capture.src" :alt="copy.title[zh ? 1 : 0]" :caption="capture.caption[zh ? 1 : 0]" :width="capture.width" :height="capture.height" :loading="loading" :framed="false" :zoom-label="zh ? '放大图片' : 'Enlarge image'" :close-label="zh ? '关闭' : 'Close'" :original-label="zh ? '查看原图' : 'View original'" />
  </div>
  <aside v-else-if="copy" class="screenshot-slot" :data-capture-id="captureId" :aria-label="copy.title[zh ? 1 : 0]">
    <span class="slot-state">{{ zh ? '截图待补' : 'Capture pending' }} · {{ captureId }}</span>
    <strong>{{ copy.title[zh ? 1 : 0] }}</strong>
    <p>{{ copy.scene[zh ? 1 : 0] }}</p>
  </aside>
</template>

<style scoped>
.completed-capture { width: 100%; min-width: 0; }
.screenshot-slot { box-sizing: border-box; width: 100%; min-width: 0; min-height: 12rem; padding: 1.5rem; border: 1px dashed var(--vp-c-divider); border-radius: var(--site-radius, 12px); background: var(--vp-c-bg-soft); color: var(--vp-c-text-2); display: flex; flex-direction: column; justify-content: center; gap: .65rem; overflow-wrap: anywhere; }
.slot-state { font-size: .8rem; }
strong { color: var(--vp-c-text-1); font-size: 1rem; }
p { margin: 0; line-height: 1.7; font-size: .9rem; }
</style>
