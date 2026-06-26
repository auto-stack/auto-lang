import { createRouter, createWebHistory } from 'vue-router'
import Home from './pages/Home.vue'
import BlockPage from './pages/BlockPage.vue'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: Home },
    { path: '/:kind/:name', component: BlockPage },
  ],
})
