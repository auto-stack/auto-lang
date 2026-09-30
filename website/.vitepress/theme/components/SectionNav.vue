<script setup lang="ts">
// PLAN-715 T-06：发布页章节快捷导航。
// 桌面：吸顶在统一顶栏之下的横向条；手机：可展开目录（aria-expanded）。
// 当前章节高亮（scroll 位置跟踪 + hash 点击即时反馈）；链接为原生 hash，
// 刷新/直达可用；目标章节 scroll-margin-top 由调用方页面统一定义，
// 保证标题不被「顶栏+本条」两层 sticky 遮挡。
import { onBeforeUnmount, onMounted, ref } from 'vue'

export interface SectionNavItem {
  id: string
  label: string
}

const props = defineProps<{
  items: SectionNavItem[]
  label?: string
  openLabel?: string
  closeLabel?: string
}>()

const current = ref('')
const mobileOpen = ref(false)
const navEl = ref<HTMLElement>()
let observer: IntersectionObserver | null = null

function observeSections() {
  if (typeof window === 'undefined' || !('IntersectionObserver' in window)) return
  observer?.disconnect()
  observer = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (entry.isIntersecting) current.value = entry.target.id
    }
  }, { rootMargin: '-45% 0px -50% 0px' })
  for (const item of props.items) {
    const el = document.getElementById(item.id)
    if (el) observer.observe(el)
  }
}

function onHashClick(id: string) {
  current.value = id
  mobileOpen.value = false
}

function onHashchange() {
  const h = window.location.hash.slice(1)
  if (h && props.items.some(i => i.id === h)) current.value = h
}

onMounted(() => {
  onHashchange()
  window.addEventListener('hashchange', onHashchange)
  // 等内容渲染后启动跟踪
  requestAnimationFrame(observeSections)
})

onBeforeUnmount(() => {
  window.removeEventListener('hashchange', onHashchange)
  observer?.disconnect()
})
</script>

<template>
  <nav ref="navEl" class="section-nav" :aria-label="label ?? 'On this page'">
    <button
      type="button"
      class="section-nav-toggle"
      :aria-expanded="mobileOpen"
      aria-controls="section-nav-list"
      @click="mobileOpen = !mobileOpen"
    >
      <span class="section-nav-current">{{ items.find(i => i.id === current)?.label ?? items[0]?.label }}</span>
      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none"
        stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
        :class="{ open: mobileOpen }" aria-hidden="true">
        <path d="m6 9 6 6 6-6" />
      </svg>
    </button>
    <div id="section-nav-list" class="section-nav-list" :class="{ open: mobileOpen }">
      <a
        v-for="item in items"
        :key="item.id"
        :href="'#' + item.id"
        class="section-nav-link"
        :class="{ current: current === item.id }"
        :aria-current="current === item.id ? 'location' : undefined"
        @click="onHashClick(item.id)"
      >{{ item.label }}</a>
    </div>
  </nav>
</template>

<style scoped>
.section-nav {
  position: sticky;
  top: var(--site-nav-height, 56px);
  z-index: 30;
  display: flex;
  justify-content: center;
  padding: 10px 24px;
  border-bottom: 1px solid hsl(var(--border));
  background: hsl(var(--background) / 92%);
  backdrop-filter: blur(12px);
}

.section-nav-toggle {
  display: none;
}

.section-nav-list {
  display: flex;
  gap: 6px;
  overflow-x: auto;
  max-width: 100%;
  scrollbar-width: none;
}

.section-nav-list::-webkit-scrollbar {
  display: none;
}

.section-nav-link {
  white-space: nowrap;
  padding: 7px 13px;
  border-radius: 6px;
  color: hsl(var(--muted-foreground));
  font-size: 0.82rem;
  font-weight: 600;
  text-decoration: none;
}

.section-nav-link:hover {
  background: hsl(var(--secondary));
  color: hsl(var(--foreground));
}

.section-nav-link.current {
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
}

.section-nav-link:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 1px;
}

@media (max-width: 900px) {
  .section-nav {
    justify-content: flex-start;
    padding: 0;
  }

  .section-nav-toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    min-height: 44px;
    padding: 8px 16px;
    border: none;
    background: transparent;
    color: hsl(var(--foreground));
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
  }

  .section-nav-toggle:focus-visible {
    outline: 3px solid var(--vp-c-brand-1);
    outline-offset: -2px;
  }

  .section-nav-toggle svg {
    transition: transform 0.15s ease;
    margin-left: auto;
  }

  .section-nav-toggle svg.open {
    transform: rotate(180deg);
  }

  .section-nav-list {
    display: none;
    flex-direction: column;
    gap: 2px;
    padding: 0 12px 12px;
  }

  .section-nav-list.open {
    display: flex;
  }

  .section-nav-link {
    min-height: 44px;
    display: inline-flex;
    align-items: center;
  }
}

@media (prefers-reduced-motion: reduce) {
  .section-nav-toggle svg {
    transition: none !important;
  }
}
</style>
