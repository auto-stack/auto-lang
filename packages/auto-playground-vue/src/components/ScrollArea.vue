<template>
  <div
    class="scroll-area"
    @mouseenter="showThumb()"
    @mouseleave="hideSoon()"
  >
    <div ref="contentEl" class="scroll-content" @scroll="onScroll">
      <slot />
    </div>
    <div
      v-if="overflowY"
      class="scrollbar scrollbar-y"
      :class="{ visible: thumbVisible }"
      @mousedown.prevent="onTrackMouseDown($event, 'y')"
      @wheel.prevent="onScrollbarWheel($event, 'y')"
    >
      <div
        class="thumb"
        :class="{ dragging: dragging === 'y' }"
        :style="{ height: thumbYSize + 'px', top: thumbYPos + 'px' }"
      />
    </div>
    <div
      v-if="overflowX"
      class="scrollbar scrollbar-x"
      :class="{ visible: thumbVisible }"
      @mousedown.prevent="onTrackMouseDown($event, 'x')"
      @wheel.prevent="onScrollbarWheel($event, 'y')"
    >
      <div
        class="thumb"
        :class="{ dragging: dragging === 'x' }"
        :style="{ width: thumbXSize + 'px', left: thumbXPos + 'px' }"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';

const MIN_THUMB = 24;
const HIDE_DELAY = 700;

const contentEl = ref<HTMLElement | null>(null);
const overflowY = ref(false);
const overflowX = ref(false);
const thumbVisible = ref(false);
const thumbYSize = ref(0);
const thumbYPos = ref(0);
const thumbXSize = ref(0);
const thumbXPos = ref(0);
const dragging = ref<'x' | 'y' | null>(null);

let hideTimer: ReturnType<typeof setTimeout> | null = null;
let resizeObserver: ResizeObserver | null = null;
let mutationObserver: MutationObserver | null = null;

function showThumb() {
  thumbVisible.value = true;
  if (hideTimer) clearTimeout(hideTimer);
}

function hideSoon() {
  if (hideTimer) clearTimeout(hideTimer);
  hideTimer = setTimeout(() => {
    if (!dragging.value) thumbVisible.value = false;
  }, HIDE_DELAY);
}

function updateThumb() {
  const el = contentEl.value;
  if (!el) return;
  const trackY = el.clientHeight - 6; // track insets (3px top + 3px bottom)
  const trackX = el.clientWidth - 6;
  const maxY = el.scrollHeight - el.clientHeight;
  const maxX = el.scrollWidth - el.clientWidth;
  overflowY.value = maxY > 1;
  overflowX.value = maxX > 1;
  if (overflowY.value) {
    thumbYSize.value = Math.max(MIN_THUMB, Math.round((el.clientHeight / el.scrollHeight) * trackY));
    thumbYPos.value = Math.round((el.scrollTop / maxY) * (trackY - thumbYSize.value));
  }
  if (overflowX.value) {
    thumbXSize.value = Math.max(MIN_THUMB, Math.round((el.clientWidth / el.scrollWidth) * trackX));
    thumbXPos.value = Math.round((el.scrollLeft / maxX) * (trackX - thumbXSize.value));
  }
}

function onScroll() {
  updateThumb();
  showThumb();
  hideSoon();
}

function onScrollbarWheel(e: WheelEvent, axis: 'x' | 'y') {
  const el = contentEl.value;
  if (!el) return;
  if (axis === 'y') el.scrollTop += e.deltaY;
  else el.scrollLeft += e.deltaX;
}

function onTrackMouseDown(e: MouseEvent, axis: 'x' | 'y') {
  const el = contentEl.value;
  if (!el) return;
  showThumb();
  if (axis === 'y') {
    const trackH = el.clientHeight - 6;
    const y = e.clientY - el.getBoundingClientRect().top - 3;
    const pos = clamp(y - thumbYSize.value / 2, 0, trackH - thumbYSize.value);
    el.scrollTop = (pos / Math.max(1, trackH - thumbYSize.value)) * (el.scrollHeight - el.clientHeight);
  } else {
    const trackW = el.clientWidth - 6;
    const x = e.clientX - el.getBoundingClientRect().left - 3;
    const pos = clamp(x - thumbXSize.value / 2, 0, trackW - thumbXSize.value);
    el.scrollLeft = (pos / Math.max(1, trackW - thumbXSize.value)) * (el.scrollWidth - el.clientWidth);
  }
  updateThumb();
  startDrag(e, axis);
}

function startDrag(e: MouseEvent, axis: 'x' | 'y') {
  const el = contentEl.value;
  if (!el) return;
  dragging.value = axis;
  const startX = e.clientX;
  const startY = e.clientY;
  const startScrollLeft = el.scrollLeft;
  const startScrollTop = el.scrollTop;
  const scaleY = el.scrollHeight / Math.max(1, el.clientHeight - 6);
  const scaleX = el.scrollWidth / Math.max(1, el.clientWidth - 6);
  const onMove = (ev: MouseEvent) => {
    if (axis === 'y') el.scrollTop = startScrollTop + (ev.clientY - startY) * scaleY;
    else el.scrollLeft = startScrollLeft + (ev.clientX - startX) * scaleX;
    updateThumb();
  };
  const onUp = () => {
    dragging.value = null;
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', onUp);
    hideSoon();
  };
  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
}

function clamp(v: number, lo: number, hi: number) {
  return Math.min(hi, Math.max(lo, v));
}

onMounted(() => {
  const el = contentEl.value;
  if (!el) return;
  resizeObserver = new ResizeObserver(() => updateThumb());
  resizeObserver.observe(el);
  mutationObserver = new MutationObserver(() => updateThumb());
  mutationObserver.observe(el, { childList: true, subtree: true, characterData: true });
  updateThumb();
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  mutationObserver?.disconnect();
  if (hideTimer) clearTimeout(hideTimer);
});

defineExpose({
  scrollTo(options?: ScrollToOptions) {
    contentEl.value?.scrollTo(options);
  },
  get scrollTop() {
    return contentEl.value?.scrollTop ?? 0;
  },
  set scrollTop(v: number) {
    if (contentEl.value) contentEl.value.scrollTop = v;
  },
});
</script>

<style scoped>
.scroll-area {
  position: relative;
  height: 100%;
  overflow: hidden;
}
.scroll-content {
  height: 100%;
  overflow: auto;
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none; /* legacy Edge */
}
.scroll-content::-webkit-scrollbar {
  display: none;
}

/* Floating overlay scrollbar: transparent track, thin rounded thumb,
   fades in on hover/scroll and out when idle */
.scrollbar {
  position: absolute;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.2s ease;
  z-index: 5;
}
.scrollbar.visible {
  opacity: 1;
  pointer-events: auto;
}
.scrollbar-y {
  top: 3px;
  right: 3px;
  width: 12px;
  height: calc(100% - 6px);
}
.scrollbar-x {
  bottom: 3px;
  left: 3px;
  height: 12px;
  width: calc(100% - 6px);
}
.thumb {
  background: rgba(212, 212, 212, 0.18);
  border-radius: 3px;
  transition: background 0.15s ease;
}
.scrollbar-y .thumb {
  width: 7px;
  margin: 0 2.5px;
}
.scrollbar-x .thumb {
  height: 7px;
  margin: 2.5px 0;
}
.scrollbar:hover .thumb,
.thumb.dragging {
  background: rgba(212, 212, 212, 0.35);
}
</style>
