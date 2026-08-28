<template>
  <div ref="rootEl" class="scroll-area">
    <div ref="contentEl" class="scroll-content">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { attachFloatingScrollbar, type FloatingScrollHandle } from '../utils/floatingScroll';

/**
 * Thin wrapper giving any slotted content a floating overlay scrollbar
 * (delegates to utils/floatingScroll, shared with the CodeEditor).
 */
const rootEl = ref<HTMLElement | null>(null);
const contentEl = ref<HTMLElement | null>(null);
let handle: FloatingScrollHandle | null = null;

onMounted(() => {
  if (!rootEl.value || !contentEl.value) return;
  handle = attachFloatingScrollbar(contentEl.value, rootEl.value);
});

onBeforeUnmount(() => {
  handle?.destroy();
  handle = null;
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
</style>
