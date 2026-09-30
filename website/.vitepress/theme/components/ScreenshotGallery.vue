<script setup lang="ts">
// PLAN-715 T-04：发布页/应用页截图画廊。
// 主图 + 选择器（缩略标签），不自动轮播；键盘（Tab/Enter/Space/方向键）可操作，
// 选中态 aria-selected/aria-live 反馈；放大走 EvidenceImage（Esc 关闭、焦点返回）。
import { computed, nextTick, ref } from 'vue'
import EvidenceImage from './EvidenceImage.vue'

export interface GalleryShot {
  src: string
  label: string
  alt: string
  caption: string
  /** 原图像素尺寸（占位防 CLS） */
  width?: number
  height?: number
}

const props = withDefaults(defineProps<{
  shots: GalleryShot[]
  zoomLabel?: string
  closeLabel?: string
  originalLabel?: string
  groupLabel?: string
  loading?: 'lazy' | 'eager'
}>(), {
  zoomLabel: 'View full size',
  closeLabel: 'Close',
  originalLabel: 'Open original',
  groupLabel: 'Screenshots',
  loading: 'lazy',
})

const selected = ref(0)
const tabList = ref<HTMLDivElement>()
const current = computed(() => props.shots[selected.value])

async function select(i: number) {
  selected.value = i
}

async function onKeydown(event: KeyboardEvent) {
  const n = props.shots.length
  let next: number | null = null
  if (event.key === 'ArrowRight') next = (selected.value + 1) % n
  else if (event.key === 'ArrowLeft') next = (selected.value + n - 1) % n
  else if (event.key === 'Home') next = 0
  else if (event.key === 'End') next = n - 1
  if (next === null) return
  event.preventDefault()
  selected.value = next
  await nextTick()
  tabList.value?.querySelectorAll<HTMLButtonElement>('[role="tab"]')[next]?.focus()
}
</script>

<template>
  <div class="shot-gallery">
    <div
      ref="tabList"
      class="shot-tabs"
      role="tablist"
      :aria-label="groupLabel"
      @keydown="onKeydown"
    >
      <button
        v-for="(shot, i) in shots"
        :key="shot.src"
        :id="'shot-tab-' + i"
        role="tab"
        type="button"
        :aria-selected="selected === i"
        aria-controls="shot-panel"
        :tabindex="selected === i ? 0 : -1"
        @click="select(i)"
      >{{ shot.label }}</button>
    </div>

    <div
      id="shot-panel"
      role="tabpanel"
      :aria-labelledby="'shot-tab-' + selected"
      aria-live="polite"
    >
      <EvidenceImage
        v-if="current"
        :key="current.src"
        :src="current.src"
        :alt="current.alt"
        :caption="current.caption"
        :zoom-label="zoomLabel"
        :close-label="closeLabel"
        :original-label="originalLabel"
        :loading="loading"
        :width="current.width"
        :height="current.height"
      />
    </div>
  </div>
</template>

<style scoped>
.shot-gallery {
  min-width: 0;
  width: 100%;
}

.shot-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  padding: 0.3rem;
  width: fit-content;
  max-width: 100%;
  margin: 0 0 0.9rem;
  background: hsl(var(--secondary));
  border: 1px solid hsl(var(--border));
  border-radius: 0.6rem;
}

.shot-tabs button {
  padding: 0.5rem 0.85rem;
  border: none;
  border-radius: 0.4rem;
  background: transparent;
  color: hsl(var(--muted-foreground));
  font-size: 0.82rem;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
}

.shot-tabs button:hover {
  color: hsl(var(--foreground));
}

.shot-tabs button[aria-selected="true"] {
  background: hsl(var(--background));
  color: hsl(var(--foreground));
  font-weight: 600;
  box-shadow: 0 2px 5px rgb(0 0 0 / 8%);
}

.shot-tabs button:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 1px;
}
</style>
