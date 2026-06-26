import { createApp } from 'vue'
import '@auto-ui/widgets/styles.css'
import './app.css'
import App from './App.vue'
import { router } from './router'

createApp(App).use(router).mount('#app')
