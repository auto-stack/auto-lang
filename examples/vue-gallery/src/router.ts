import { createRouter, createWebHistory } from 'vue-router'
import Home from './pages/Home.vue'

// One route per v1 widget.
const routes = [
  { path: '/', component: Home },
  { path: '/button', component: () => import('./pages/button.vue') },
  { path: '/input', component: () => import('./pages/input.vue') },
  { path: '/textarea', component: () => import('./pages/textarea.vue') },
  { path: '/checkbox', component: () => import('./pages/checkbox.vue') },
  { path: '/switch', component: () => import('./pages/switch.vue') },
  { path: '/label', component: () => import('./pages/label.vue') },
  { path: '/card', component: () => import('./pages/card.vue') },
  { path: '/separator', component: () => import('./pages/separator.vue') },
  { path: '/badge', component: () => import('./pages/badge.vue') },
  { path: '/avatar', component: () => import('./pages/avatar.vue') },
  { path: '/dialog', component: () => import('./pages/dialog.vue') },
  { path: '/tabs', component: () => import('./pages/tabs.vue') },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})
