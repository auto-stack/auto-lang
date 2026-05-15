import { h, defineComponent, watch, onMounted } from 'vue'
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
import { AutoPlayground } from 'auto-playground-vue'
import CodeView from './components/CodeView.vue'
import UnifiedNavbar from './components/UnifiedNavbar.vue'

// SPA routes served from public/ui/*/index.html.
// VitePress client-side router doesn't know about these, so we must
// force a full page load when navigating to them.
const SPA_ROUTES = ['/ui/gallery/', '/ui/blocks/', '/ui/charts/', '/ui/a2ui/']

function isSpaRoute(path: string): boolean {
  return SPA_ROUTES.some(r => path === r || path.startsWith(r))
}

const LayoutWrapper = defineComponent({
  setup() {
    const router = useRouter()
    const route = useRoute()

    onMounted(() => {
      // If we landed on a SPA route via initial load, force full reload
      if (isSpaRoute(route.path)) {
        window.location.href = route.path
        return
      }
    })

    watch(() => route.path, (to) => {
      if (isSpaRoute(to)) {
        // Intercept client-side navigation to SPA routes — do full page load
        window.location.href = to
      }
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
    app.component('CodeView', CodeView)
  },
} satisfies Theme
