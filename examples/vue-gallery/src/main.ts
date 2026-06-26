import { createApp } from 'vue'
// Zero-config styling path (Plan 331 §6): a single precompiled stylesheet
// instead of running our own Tailwind.
import '@auto-ui/widgets/styles.css'
import App from './App.vue'

createApp(App).mount('#app')
