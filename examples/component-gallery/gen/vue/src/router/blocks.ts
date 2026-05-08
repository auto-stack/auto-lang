import { createRouter, createWebHashHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'blocks', component: () => import('@/pages/blocks.vue') },
  { path: '/blocks', redirect: '/' },
  { path: '/blocks/login-01', name: 'login_01', component: () => import('@/pages/blocks/login_01.vue') },
  { path: '/blocks/login-02', name: 'login_02', component: () => import('@/pages/blocks/login_02.vue') },
  { path: '/blocks/login-03', name: 'login_03', component: () => import('@/pages/blocks/login_03.vue') },
  { path: '/blocks/login-04', name: 'login_04', component: () => import('@/pages/blocks/login_04.vue') },
  { path: '/blocks/login-05', name: 'login_05', component: () => import('@/pages/blocks/login_05.vue') },
  { path: '/blocks/dashboard-01', name: 'dashboard_01', component: () => import('@/pages/blocks/dashboard_01.vue') },
  { path: '/blocks/products-01', name: 'products_01', component: () => import('@/pages/blocks/products_01.vue') },
  { path: '/blocks/sidebar-01', name: 'sidebar_01', component: () => import('@/pages/blocks/sidebar_01.vue') },
  { path: '/blocks/sidebar-02', name: 'sidebar_02', component: () => import('@/pages/blocks/sidebar_02.vue') },
  { path: '/blocks/sidebar-03', name: 'sidebar_03', component: () => import('@/pages/blocks/sidebar_03.vue') },
  { path: '/blocks/sidebar-04', name: 'sidebar_04', component: () => import('@/pages/blocks/sidebar_04.vue') },
  { path: '/blocks/sidebar-05', name: 'sidebar_05', component: () => import('@/pages/blocks/sidebar_05.vue') },
  { path: '/blocks/sidebar-06', name: 'sidebar_06', component: () => import('@/pages/blocks/sidebar_06.vue') },
  { path: '/blocks/sidebar-07', name: 'sidebar_07', component: () => import('@/pages/blocks/sidebar_07.vue') },
  { path: '/blocks/sidebar-08', name: 'sidebar_08', component: () => import('@/pages/blocks/sidebar_08.vue') },
  { path: '/blocks/sidebar-09', name: 'sidebar_09', component: () => import('@/pages/blocks/sidebar_09.vue') },
  { path: '/blocks/sidebar-10', name: 'sidebar_10', component: () => import('@/pages/blocks/sidebar_10.vue') },
  { path: '/blocks/sidebar-11', name: 'sidebar_11', component: () => import('@/pages/blocks/sidebar_11.vue') },
  { path: '/blocks/sidebar-12', name: 'sidebar_12', component: () => import('@/pages/blocks/sidebar_12.vue') },
  { path: '/blocks/sidebar-13', name: 'sidebar_13', component: () => import('@/pages/blocks/sidebar_13.vue') },
  { path: '/blocks/sidebar-14', name: 'sidebar_14', component: () => import('@/pages/blocks/sidebar_14.vue') },
  { path: '/blocks/sidebar-15', name: 'sidebar_15', component: () => import('@/pages/blocks/sidebar_15.vue') },
  { path: '/blocks/sidebar-16', name: 'sidebar_16', component: () => import('@/pages/blocks/sidebar_16.vue') },
  { path: '/blocks/sidebar-demo', name: 'sidebar_demo', component: () => import('@/pages/blocks/sidebar_demo.vue') },
  { path: '/:pathMatch(.*)*', redirect: '/' },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router
