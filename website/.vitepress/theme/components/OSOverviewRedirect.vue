<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vitepress'
const props = defineProps<{ target: string }>()
const router = useRouter()
onMounted(() => router.go(props.target + (window.location.hash || '#desktop-showcase')))
</script>

<template>
  <section class="os-compatibility">
    <h1>AutoOS</h1>
    <p>{{ target.startsWith('/zh/') ? '桌面、应用和设置介绍已并入 OS 主页面。' : 'The desktop, applications and settings are now part of the OS overview.' }}</p>
    <a :href="target + '#desktop-showcase'">{{ target.startsWith('/zh/') ? '查看 OS 主页面 →' : 'View the OS overview →' }}</a>
  </section>
</template>

<style scoped>
.os-compatibility { max-width: var(--site-text-width, 720px); margin: 3rem auto; padding: 0 1.5rem; }
h1 { font-size: 2rem; margin-bottom: 1rem; }p { line-height: 1.8; margin-bottom: 1rem; }a { color: var(--vp-c-brand-1); display: inline-flex; align-items: center; min-height: 44px; }
</style>
