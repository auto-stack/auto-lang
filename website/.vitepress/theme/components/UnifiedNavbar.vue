<script setup lang="ts">
// PLAN-715 T-02：全站统一顶栏。
// - 数据单一来源 ../data/navigation.ts（双语链接/共享 SPA 标注），手机与桌面同数据。
// - 当前位置：命中项 aria-current="page"，父组触发器 aria-current="true"。
// - 键盘：下拉按钮 aria-expanded/aria-controls，Esc 收起并回焦触发器，导航后收起；
//   点击外部收起。
// - 语言切换按 navigation.switchLocaleHref 前缀规则，保留 fragment，不造 404。
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vitepress'
import {
  NAV_GROUPS,
  NAV_HOME,
  NAV_HIGHLIGHT,
  NAV_TOP_LEAVES,
  isRouteActive,
  localize,
  switchLocaleHref,
  type NavGroup,
  type NavItemView,
} from '../data/navigation'
import { useDarkMode } from '../composables/useDarkMode'
import SiteSearch from './SiteSearch.vue'

const route = useRoute()
const zh = computed(() => route.path === '/zh' || route.path.startsWith('/zh/'))

const home = computed(() => localize(NAV_HOME, zh.value))
const groups = computed(() =>
  NAV_GROUPS.map((g: NavGroup) => ({
    ...g,
    label: zh.value ? g.labelZh : g.labelEn,
    items: g.items.map((i) => localize(i, zh.value)),
  })),
)
const topLeaves = computed(() => NAV_TOP_LEAVES.map((i) => localize(i, zh.value)))
const highlight = computed(() => localize(NAV_HIGHLIGHT, zh.value))

const langSwitchHref = computed(() => switchLocaleHref(route.path, !zh.value))
const langSwitchLabel = computed(() => (zh.value ? 'English' : '中文'))

const { isDark, toggle } = useDarkMode()

const openGroup = ref<string | null>(null)
const mobileMenuOpen = ref(false)
const groupRefs = new Map<string, HTMLButtonElement>()
const mobileTrigger = ref<HTMLButtonElement>()

function toggleGroup(key: string) {
  openGroup.value = openGroup.value === key ? null : key
}

function groupActive(g: { items: NavItemView[] }): boolean {
  return g.items.some((i) => !i.shared && isRouteActive(i.href, route.path))
}

function setGroupRef(key: string, el: unknown) {
  if (el) groupRefs.set(key, el as HTMLButtonElement)
  else groupRefs.delete(key)
}

function onGroupKeydown(event: KeyboardEvent, key: string) {
  if (event.key === 'Escape' && openGroup.value === key) {
    event.stopPropagation()
    openGroup.value = null
    groupRefs.get(key)?.focus()
  }
}

function onDocClick(event: MouseEvent) {
  const el = event.target as HTMLElement
  if (!el.closest('.nav-group') && !el.closest('.mobile-menu')) {
    openGroup.value = null
  }
}

function onGlobalKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && mobileMenuOpen.value && !event.defaultPrevented) {
    mobileMenuOpen.value = false
    mobileTrigger.value?.focus()
  }
}

onMounted(() => {
  document.addEventListener('click', onDocClick)
  document.addEventListener('keydown', onGlobalKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', onDocClick)
  document.removeEventListener('keydown', onGlobalKeydown)
})

watch(() => route.path, () => {
  openGroup.value = null
  mobileMenuOpen.value = false
})

function onMobileKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && mobileMenuOpen.value) {
    event.stopPropagation()
    mobileMenuOpen.value = false
    mobileTrigger.value?.focus()
  }
}
</script>

<template>
  <header class="site-navbar">
    <div class="site-navbar-inner">
      <!-- Logo -->
      <a :href="home.href" class="brand" :aria-label="home.label">
        <img src="/auto.svg" alt="" class="brand-logo" />
        <span class="brand-name">Auto Language</span>
      </a>

      <!-- Desktop nav -->
      <nav class="desktop-nav" :aria-label="zh ? '主导航' : 'Main navigation'">
        <a
          v-for="item in topLeaves.slice(0, 1)"
          :key="item.key"
          :href="item.href"
          class="nav-link nav-release"
          :aria-current="isRouteActive(item.href, route.path) ? 'page' : undefined"
        >{{ item.label }}</a>

        <div v-for="g in groups" :key="g.key" class="nav-group">
          <button
            :ref="(el) => setGroupRef(g.key, el)"
            type="button"
            class="nav-link nav-group-trigger"
            :aria-expanded="openGroup === g.key"
            :aria-controls="'nav-group-' + g.key"
            :aria-current="groupActive(g) ? 'true' : undefined"
            @click="toggleGroup(g.key)"
            @keydown="onGroupKeydown($event, g.key)"
          >
            {{ g.label }}
            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none"
              stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="nav-caret" :class="{ open: openGroup === g.key }" aria-hidden="true">
              <path d="m6 9 6 6 6-6" />
            </svg>
          </button>
          <div v-if="openGroup === g.key" :id="'nav-group-' + g.key" class="nav-dropdown">
            <a
              v-for="item in g.items"
              :key="item.key"
              :href="item.href"
              class="nav-dropdown-item"
              :aria-current="!item.shared && isRouteActive(item.href, route.path) ? 'page' : undefined"
            >
              <span>{{ item.label }}</span>
              <span v-if="item.shared" class="nav-shared-tag">{{ zh ? '共享' : 'shared' }}</span>
            </a>
          </div>
        </div>

        <a
          v-for="item in topLeaves.slice(1)"
          :key="item.key"
          :href="item.href"
          class="nav-link"
          :aria-current="isRouteActive(item.href, route.path) ? 'page' : undefined"
        >{{ item.label }}</a>
      </nav>

      <!-- Right side actions -->
      <div class="nav-actions">
        <SiteSearch class="nav-search" />

        <a :href="langSwitchHref" class="nav-icon-btn nav-lang" :lang="zh ? 'en' : 'zh-CN'">
          {{ langSwitchLabel }}
        </a>

        <button
          type="button"
          class="nav-icon-btn"
          :aria-label="zh ? '切换深浅主题' : 'Toggle dark mode'"
          :aria-pressed="isDark"
          @click="toggle"
        >
          <svg v-if="isDark" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="4" />
            <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
          </svg>
        </button>

        <a :href="highlight.href" class="nav-cta">{{ highlight.label }}</a>

        <button
          :ref="(el) => { if (el) mobileTrigger = el as HTMLButtonElement }"
          type="button"
          class="nav-icon-btn nav-mobile-trigger"
          :aria-label="zh ? '打开菜单' : 'Open menu'"
          :aria-expanded="mobileMenuOpen"
          :aria-controls="'mobile-menu'"
          @click="mobileMenuOpen = !mobileMenuOpen"
          @keydown="onMobileKeydown"
        >
          <svg v-if="!mobileMenuOpen" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <line x1="4" x2="20" y1="12" y2="12" /><line x1="4" x2="20" y1="6" y2="6" /><line x1="4" x2="20" y1="18" y2="18" />
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M18 6 6 18M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Mobile menu: same data, grouped -->
    <div v-if="mobileMenuOpen" id="mobile-menu" class="mobile-menu">
      <div v-for="g in groups" :key="g.key" class="mobile-group">
        <div class="mobile-group-title">{{ g.label }}</div>
        <a
          v-for="item in g.items"
          :key="item.key"
          :href="item.href"
          class="mobile-link"
          :aria-current="!item.shared && isRouteActive(item.href, route.path) ? 'page' : undefined"
        >
          <span>{{ item.label }}</span>
          <span v-if="item.shared" class="nav-shared-tag">{{ zh ? '共享' : 'shared' }}</span>
        </a>
      </div>
      <div class="mobile-group">
        <div class="mobile-group-title">{{ zh ? '更多' : 'More' }}</div>
        <a
          v-for="item in topLeaves"
          :key="item.key"
          :href="item.href"
          class="mobile-link"
          :aria-current="isRouteActive(item.href, route.path) ? 'page' : undefined"
        >{{ item.label }}</a>
        <a :href="highlight.href" class="mobile-link mobile-link-highlight">{{ highlight.label }}</a>
      </div>
    </div>
  </header>
</template>

<style scoped>
.site-navbar {
  position: sticky;
  top: 0;
  z-index: 50;
  width: 100%;
  border-bottom: 1px solid hsl(var(--border));
  background: hsl(var(--background) / 0.82);
  backdrop-filter: blur(12px);
}

.site-navbar-inner {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  height: var(--site-nav-height, 56px);
  max-width: 1280px;
  margin: 0 auto;
  padding: 0 1rem;
}

.brand {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  margin-right: 0.75rem;
  flex-shrink: 0;
  color: hsl(var(--foreground));
  text-decoration: none;
  border-radius: 8px;
}

.brand:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 2px;
}

.brand-logo {
  height: 30px;
  width: auto;
}

.brand-name {
  font-weight: 700;
  font-size: 1.02rem;
  white-space: nowrap;
}

@media (max-width: 640px) {
  .brand-name {
    display: none;
  }
}

.desktop-nav {
  display: none;
  align-items: center;
  gap: 0.125rem;
  font-size: 0.875rem;
}

@media (min-width: 1200px) {
  .desktop-nav {
    display: flex;
  }
}

.nav-link {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.4rem 0.65rem;
  border-radius: 0.5rem;
  color: hsl(var(--foreground) / 0.8);
  text-decoration: none;
  white-space: nowrap;
  background: transparent;
  border: none;
  cursor: pointer;
  font: inherit;
  transition: color 0.15s ease, background 0.15s ease;
}

.nav-link:hover {
  color: hsl(var(--foreground));
  background: hsl(var(--accent));
}

.nav-link[aria-current="page"],
.nav-link[aria-current="true"] {
  color: var(--vp-c-brand-1);
  background: var(--vp-c-brand-soft);
  font-weight: 600;
}

.nav-link:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 1px;
}

.nav-release {
  color: var(--vp-c-brand-1);
  font-weight: 700;
}

.nav-release:hover {
  background: var(--vp-c-brand-soft);
}

.nav-release[aria-current="page"] {
  background: var(--vp-c-brand-soft);
}

.nav-group {
  position: relative;
}

.nav-caret {
  transition: transform 0.15s ease;
}

.nav-caret.open {
  transform: rotate(180deg);
}

.nav-dropdown {
  position: absolute;
  left: 0;
  top: calc(100% + 6px);
  min-width: 13rem;
  padding: 0.375rem;
  background: hsl(var(--popover));
  border: 1px solid hsl(var(--border));
  border-radius: 0.625rem;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.14);
  z-index: 60;
}

.nav-dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.45rem 0.6rem;
  border-radius: 0.4rem;
  color: hsl(var(--popover-foreground));
  text-decoration: none;
  font-size: 0.875rem;
}

.nav-dropdown-item:hover {
  background: hsl(var(--accent));
}

.nav-dropdown-item[aria-current="page"] {
  color: var(--vp-c-brand-1);
  font-weight: 600;
}

.nav-dropdown-item:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 1px;
}

.nav-shared-tag {
  flex-shrink: 0;
  padding: 1px 6px;
  border-radius: 9999px;
  background: hsl(var(--muted));
  color: hsl(var(--muted-foreground));
  font-size: 10px;
  font-weight: 600;
}

.nav-actions {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  margin-left: auto;
}

.nav-search {
  margin-right: 0.25rem;
}

.nav-lang {
  width: auto;
  padding: 0 0.5rem;
  font-size: 0.8125rem;
}

.nav-icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  border: none;
  border-radius: 0.5rem;
  background: transparent;
  color: hsl(var(--foreground) / 0.8);
  cursor: pointer;
  text-decoration: none;
  transition: color 0.15s ease, background 0.15s ease;
}

.nav-icon-btn:hover {
  color: hsl(var(--foreground));
  background: hsl(var(--accent));
}

.nav-icon-btn:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 1px;
}

@media (min-width: 768px) {
  .nav-icon-btn {
    width: 36px;
    height: 36px;
  }
}

.nav-cta {
  display: none;
  align-items: center;
  height: 36px;
  padding: 0 0.9rem;
  margin-left: 0.25rem;
  border-radius: 9999px;
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  color: #fff;
  font-size: 0.8125rem;
  font-weight: 600;
  text-decoration: none;
  white-space: nowrap;
  transition: box-shadow 0.15s ease, transform 0.15s ease;
}

.nav-cta:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 14px rgba(99, 102, 241, 0.35);
}

.nav-cta:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 2px;
}

@media (min-width: 768px) {
  .nav-cta {
    display: inline-flex;
  }
}

.nav-mobile-trigger {
  display: inline-flex;
}

@media (min-width: 1200px) {
  .nav-mobile-trigger {
    display: none;
  }
}

.mobile-menu {
  display: block;
  max-height: calc(100dvh - var(--site-nav-height, 56px));
  overflow-y: auto;
  border-top: 1px solid hsl(var(--border));
  padding: 0.5rem 1rem 1.25rem;
  background: hsl(var(--background));
}

@media (min-width: 1200px) {
  .mobile-menu {
    display: none;
  }
}

.mobile-group + .mobile-group {
  margin-top: 0.75rem;
}

.mobile-group-title {
  padding: 0.5rem 0 0.25rem;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: hsl(var(--muted-foreground));
}

.mobile-link {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 44px;
  padding: 0.5rem 0.5rem;
  border-radius: 0.5rem;
  color: hsl(var(--foreground));
  text-decoration: none;
  font-size: 0.95rem;
}

.mobile-link:hover {
  background: hsl(var(--accent));
}

.mobile-link[aria-current="page"] {
  color: var(--vp-c-brand-1);
  font-weight: 600;
}

.mobile-link:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 1px;
}

.mobile-link-highlight {
  color: var(--vp-c-brand-1);
  font-weight: 700;
}

@media (prefers-reduced-motion: reduce) {
  .nav-caret,
  .nav-cta,
  .nav-link,
  .nav-icon-btn {
    transition: none !important;
    transform: none !important;
  }
}
</style>
