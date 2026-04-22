import { h } from 'vue'
import type { Theme } from 'vitepress'
import DefaultTheme from 'vitepress/theme'
import './style.css'
import HomeHero from './components/HomeHero.vue'
import FeatureCard from './components/FeatureCard.vue'
import AutoPlayground from './components/AutoPlayground.vue'

export default {
  extends: DefaultTheme,
  Layout: () => {
    return h(DefaultTheme.Layout, null, {
      // slot overrides if needed
    })
  },
  enhanceApp({ app }) {
    app.component('HomeHero', HomeHero)
    app.component('FeatureCard', FeatureCard)
    app.component('AutoPlayground', AutoPlayground)
  },
} satisfies Theme
