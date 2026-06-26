import { createApp } from 'vue'
// Zero-config styling path (Plan 331 §6): a single precompiled stylesheet
// instead of running our own Tailwind. Provides widget classes + design tokens.
import '@auto-ui/widgets/styles.css'
// Showcase chrome (plain CSS, reuses the token variables above). Loaded after,
// so it can reference --background/--border/etc. and layer on top of the widgets.
import './assets/app.css'
import App from './App.vue'
import { router } from './router'

createApp(App).use(router).mount('#app')
