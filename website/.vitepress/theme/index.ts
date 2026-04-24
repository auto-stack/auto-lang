import { h } from 'vue'
import type { Theme } from 'vitepress'
import DefaultTheme from 'vitepress/theme'
import './style.css'
import HomeHero from './components/HomeHero.vue'
import AIHero from './components/AIHero.vue'
import OSHero from './components/OSHero.vue'
import FeatureCard from './components/FeatureCard.vue'
import StatCard from './components/StatCard.vue'
import ShowcaseSection from './components/ShowcaseSection.vue'
import AutoPlayground from './components/AutoPlayground.vue'
import CodeView from './components/CodeView.vue'

export default {
  extends: DefaultTheme,
  Layout: () => {
    return h(DefaultTheme.Layout, null, {
      // slot overrides if needed
    })
  },
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
