<script setup lang="ts">
// PLAN-715 T-02：站内搜索按钮。
// 打开机制 = VitePress 官方自用的合成热键法（VPNavBarSearch.vue algolia poll()：
// key='k' + metaKey/ctrlKey 派发到 window；local provider 的 onKeyStroke 会打开
// VPLocalSearchBox，其弹窗 Teleport 到 body，不受 VPNav CSS 隐藏影响）。
// 不导入 vitepress 内部组件；版本约束：依赖 vitepress 1.6.x 的 local search
// 行为（详见 docs/reports/p715-website-ui-baseline.md §3，升级需回归检查）。
import { computed } from 'vue'
import { useRoute } from 'vitepress'

const route = useRoute()
const zh = computed(() => route.path.startsWith('/zh'))

function openSearch() {
  const e = new KeyboardEvent('keydown', { key: 'k', ctrlKey: true, bubbles: true })
  window.dispatchEvent(e)
}
</script>

<template>
  <button
    type="button"
    class="site-search"
    :aria-label="zh ? '搜索（Ctrl+K）' : 'Search (Ctrl+K)'"
    :title="zh ? '搜索（Ctrl+K）' : 'Search (Ctrl+K)'"
    @click="openSearch"
  >
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
      stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.3-4.3" />
    </svg>
    <span class="site-search-text">{{ zh ? '搜索' : 'Search' }}</span>
    <kbd aria-hidden="true">Ctrl K</kbd>
  </button>
</template>

<style scoped>
.site-search {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  height: 2.25rem;
  padding: 0 0.5rem;
  border: 1px solid hsl(var(--border));
  border-radius: 0.5rem;
  background: hsl(var(--background) / 0.6);
  color: hsl(var(--muted-foreground));
  font-size: 0.8125rem;
  cursor: pointer;
  transition: color 0.15s ease, border-color 0.15s ease, background 0.15s ease;
}

.site-search:hover {
  color: hsl(var(--foreground));
  border-color: hsl(var(--foreground) / 0.3);
}

.site-search:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 2px;
}

.site-search-text {
  display: none;
}

@media (min-width: 768px) {
  .site-search-text {
    display: inline;
  }
}

.site-search kbd {
  display: none;
  padding: 1px 5px;
  border: 1px solid hsl(var(--border));
  border-radius: 4px;
  font-family: var(--vp-font-family-mono);
  font-size: 10px;
  line-height: 1.4;
  background: hsl(var(--muted));
}

@media (min-width: 1024px) {
  .site-search kbd {
    display: inline-block;
  }
}
</style>
