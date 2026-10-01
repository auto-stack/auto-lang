import { h, defineComponent, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useRouter, useRoute } from 'vitepress'
import type { Theme } from 'vitepress'
import DefaultTheme from 'vitepress/theme'
import './style.css'
import HomeHero from './components/HomeHero.vue'
import AIHero from './components/AIHero.vue'
import OSHero from './components/OSHero.vue'
import FeatureCard from './components/FeatureCard.vue'
import StatCard from './components/StatCard.vue'
import ShowcaseSection from './components/ShowcaseSection.vue'
import { AutoPlayground, NotesExplorer } from 'auto-playground-vue'
import AutoFence from './components/AutoFence.vue'
import CodeView from './components/CodeView.vue'
import ScriptShipView from './components/ScriptShipView.vue'
import UnifiedNavbar from './components/UnifiedNavbar.vue'
import LearningHub from './components/LearningHub.vue'

// SPA routes served from public/ui/*/index.html.
// VitePress client-side router doesn't know about these, so we must
// force a full page load when navigating to them.
const SPA_ROUTES = ['/ui/gallery/', '/ui/blocks/', '/ui/charts/', '/ui/a2ui/', '/ui/demos/']

function isSpaRoute(path: string): boolean {
  return SPA_ROUTES.some(r => path === r || path.startsWith(r))
}

// Reveal-on-scroll for landing page sections: adds .reveal to section
// wrappers, then .is-visible when they enter the viewport. CSS lives in
// landing.css and is gated behind prefers-reduced-motion: no-preference.
const REVEAL_SELECTOR = [
  '.landing-page .stats-section',
  '.landing-page .showcase-wrapper',
  '.landing-page .features-section',
  '.landing-page .platforms-section',
  '.landing-page .pillars-section',
  '.landing-page .apps-section',
  '.landing-page .apps-list',
  '.landing-page .cta-section',
].join(', ')

function setupReveal() {
  if (typeof window === 'undefined' || !('IntersectionObserver' in window)) return
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return
  const els = Array.from(document.querySelectorAll(REVEAL_SELECTOR))
    .filter(el => !el.classList.contains('is-visible'))
  if (!els.length) return
  const observer = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (entry.isIntersecting) {
        entry.target.classList.add('is-visible')
        observer.unobserve(entry.target)
      }
    }
  }, { rootMargin: '0px 0px -10% 0px' })
  for (const el of els) {
    el.classList.add('reveal')
    observer.observe(el)
  }
}

const LayoutWrapper = defineComponent({
  setup() {
    const router = useRouter()
    const route = useRoute()

    // PLAN-715 T-02：Ctrl/Cmd+K 不抢编辑器输入。VitePress local search 的
    // Ctrl/Cmd+K 监听挂在 window（不检查编辑态），CodeMirror 用 contenteditable
    // 不属于它豁免的 INPUT/TEXTAREA。这里在 document 捕获相拦截 .cm-editor 内的
    // Ctrl/Cmd+K，事件不再到达 window 监听；搜索按钮（SiteSearch）直接向 window
    // 派发合成事件、不经过 document 捕获路径，不受影响。
    function cmHotkeyGuard(event: KeyboardEvent) {
      if (
        event.key === 'k'
        && (event.ctrlKey || event.metaKey)
        && (event.target as HTMLElement | null)?.closest?.('.cm-editor')
      ) {
        event.stopPropagation()
      }
    }

    onMounted(() => {
      document.addEventListener('keydown', cmHotkeyGuard, true)
      // If we landed on a SPA route via initial load, force full reload
      if (isSpaRoute(route.path)) {
        window.location.href = route.path
        return
      }
      setupReveal()
    })

    onUnmounted(() => {
      document.removeEventListener('keydown', cmHotkeyGuard, true)
    })

    watch(() => route.path, (to) => {
      if (isSpaRoute(to)) {
        // Intercept client-side navigation to SPA routes — do full page load
        window.location.href = to
      }
      nextTick(setupReveal)
    })

    return () => h(DefaultTheme.Layout, null, {
      'layout-top': () => h(UnifiedNavbar),
    })
  },
})

export default {
  extends: DefaultTheme,
  Layout: LayoutWrapper,
  enhanceApp({ app }) {
    app.component('HomeHero', HomeHero)
    app.component('AIHero', AIHero)
    app.component('OSHero', OSHero)
    app.component('FeatureCard', FeatureCard)
    app.component('StatCard', StatCard)
    app.component('ShowcaseSection', ShowcaseSection)
    app.component('AutoPlayground', AutoPlayground)
    app.component('NotesExplorer', NotesExplorer)
    app.component('AutoFence', AutoFence)
    app.component('CodeView', CodeView)
    app.component('ScriptShipView', ScriptShipView)
    app.component('LearningHub', LearningHub)
  },
} satisfies Theme
