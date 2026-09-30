<script setup lang="ts">
import { ref } from 'vue'

// PLAN-715 T-04：EvidenceImage 扩展。
// 新增可选 props（既有调用全部缺省，行为不变）：
// - loading: 'lazy'(默认) | 'eager' —— 首屏主图用 eager + 尺寸占位（防 CLS）
// - width/height: 原图像素尺寸，用于 aspect-ratio 占位
// - framed: false 时去掉深色底容器（浅色截图不再衬深框），默认 true 保持现状
const props = withDefaults(defineProps<{
  src: string
  alt: string
  caption: string
  zoomLabel: string
  closeLabel: string
  originalLabel: string
  tablePreview?: boolean
  loading?: 'lazy' | 'eager'
  width?: number
  height?: number
  framed?: boolean
}>(), {
  tablePreview: false,
  loading: 'lazy',
  framed: true,
})

const dialog = ref<HTMLDialogElement>()
function open() {
  dialog.value?.showModal()
  dialog.value?.querySelector<HTMLButtonElement>('button')?.focus()
}
function close() { dialog.value?.close() }
function backdrop(event: MouseEvent) {
  if (event.target === dialog.value) close()
}
// Esc 关闭后焦点回缩略图触发器（dialog 原生行为回焦 open() 前的元素，
// 但显式回焦更稳——tabindex 由按钮自带）。
function onDialogClose() {
  ;(dialog.value?.previousElementSibling as HTMLElement | null)?.querySelector<HTMLElement>('.evidence-image')?.focus()
}
</script>

<template>
  <figure class="evidence-figure">
    <button type="button" class="evidence-image" :class="{ framed }" :aria-label="zoomLabel + ': ' + alt" @click="open">
      <span :class="{ 'table-preview': tablePreview }">
        <img :src="src" :alt="alt" :loading="loading" decoding="async"
          :width="width" :height="height"
          :style="width && height ? { aspectRatio: `${width} / ${height}` } : undefined" />
      </span>
      <span class="zoom-label">{{ zoomLabel }} <span aria-hidden="true">↗</span></span>
    </button>
    <figcaption>{{ caption }}</figcaption>
  </figure>
  <dialog ref="dialog" class="evidence-dialog" :aria-label="alt" @click="backdrop" @close="onDialogClose">
    <div class="dialog-content">
      <div class="dialog-toolbar">
        <span>{{ alt }}</span>
        <button type="button" autofocus @click="close">{{ closeLabel }} <span aria-hidden="true">×</span></button>
      </div>
      <img :src="src" :alt="alt" />
      <div class="dialog-footer"><p>{{ caption }}</p><a :href="src" target="_blank" rel="noopener">{{ originalLabel }} ↗</a></div>
    </div>
  </dialog>
</template>

<style scoped>
.evidence-figure { margin: 0; min-width: 0; width: 100%; }
.evidence-image { display: block; position: relative; padding: 0; width: 100%; border: 1px solid hsl(var(--border)); border-radius: 12px; overflow: hidden; background: transparent; cursor: zoom-in; }
.evidence-image.framed { background: #0b1020; }
.evidence-image img { display: block; width: 100%; height: auto; }
.table-preview { display: block; aspect-ratio: 1000 / 502; overflow: hidden; text-align: left; }
.table-preview img { width: 192%; max-width: none; }
.zoom-label { display: flex; justify-content: space-between; padding: 10px 16px; color: #cdd6f4; background: #151d2e; font-size: 12px; font-weight: 600; }
figcaption { margin-top: 12px; color: hsl(var(--muted-foreground)); font-size: 12px; line-height: 1.7; }
.evidence-dialog { width: min(1280px, calc(100vw - 32px)); max-width: none; max-height: calc(100dvh - 32px); margin: auto; padding: 0; border: 1px solid hsl(var(--border)); border-radius: 14px; color: hsl(var(--foreground)); background: hsl(var(--background)); overflow: auto; }
.evidence-dialog::backdrop { background: rgb(3 7 18 / 85%); backdrop-filter: blur(6px); }
.dialog-toolbar { position: sticky; top: 0; display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 16px 20px; border-bottom: 1px solid hsl(var(--border)); background: hsl(var(--background)); z-index: 1; font-weight: 600; }
.dialog-toolbar button { flex-shrink: 0; padding: 6px 12px; border: 1px solid hsl(var(--border)); border-radius: 6px; background: hsl(var(--secondary)); }
.dialog-content > img { display: block; width: 100%; }
.dialog-footer { padding: 16px 20px; font-size: 13px; line-height: 1.7; }
.dialog-footer p { color: hsl(var(--muted-foreground)); margin: 0 0 8px; }
.dialog-footer a { color: var(--vp-c-brand-1); text-decoration: underline; }
button:focus-visible, a:focus-visible { outline: 3px solid var(--vp-c-brand-1); outline-offset: 4px; }
@media (max-width: 480px) { .dialog-toolbar { font-size: 13px; padding: 12px; } }
</style>
