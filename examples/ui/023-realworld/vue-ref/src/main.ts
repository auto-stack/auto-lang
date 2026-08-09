import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import './style.css'
import { auth } from './stores/auth'

// Plan 405 vue-ref entry. Hash-free history router (auto codegen uses hash,
// but the prototype only validates interactions — router mode is immaterial).

// Restore session from localStorage on boot (RealWorld standard).
const stored = localStorage.getItem('rw_user')
if (stored) {
  try { Object.assign(auth, JSON.parse(stored)) } catch { /* ignore */ }
}

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'home', component: () => import('./views/Home.vue') },
    { path: '/login', name: 'login', component: () => import('./views/Login.vue') },
    { path: '/register', name: 'register', component: () => import('./views/Register.vue') },
    { path: '/article/:slug', name: 'article', component: () => import('./views/ArticleDetail.vue') },
    { path: '/settings', name: 'settings', component: () => import('./views/Settings.vue') },
  ],
})

createApp(App).use(router).mount('#app')
