<script setup lang="ts">
import { computed } from 'vue'
import { useData } from 'vitepress'
import IntroductionRelation from './IntroductionRelation.vue'
import { introCopy } from '../data/os-ai-introduction'
const props = defineProps<{ kind: 'os' | 'ai' | 'desktop' | 'uiDesktop' }>()
const { lang } = useData()
const zh = computed(() => lang.value.startsWith('zh'))
const c = computed(() => introCopy(zh.value).sections[props.kind])
</script>

<template>
  <header class="introduction-header">
    <h1>{{ c.title }}</h1>
    <p class="intro-lead">{{ c.intro }}</p>
    <nav class="intro-links" :aria-label="zh ? '介绍与资料入口' : 'Introduction and source links'">
      <a v-for="link in c.links" :key="link.href" :href="link.href">{{ link.label }} <span aria-hidden="true">→</span></a>
    </nav>
    <p class="intro-date">{{ zh ? '当前进展说明 · 2026-10-01' : 'Current progress described as of 2026-10-01' }}</p>
    <IntroductionRelation v-if="kind === 'ai'" kind="ai" />
  </header>
</template>

<style scoped>
.introduction-header { margin-bottom: 2rem; min-width: 0; }
h1 { font-size: clamp(1.8rem, 4vw, 2.6rem); line-height: 1.25; text-wrap: balance; }
.intro-lead { font-size: 1.08rem; line-height: 1.8; color: var(--vp-c-text-2); margin: 1.25rem 0; }
.intro-links { display: flex; flex-wrap: wrap; gap: 0.25rem 1.25rem; }
.intro-links a { display: inline-flex; align-items: center; gap: .4rem; min-height: 44px; }
.intro-date { color: var(--vp-c-text-2); font-size: .85rem; line-height: 1.7; }
.intro-links a:focus-visible { outline: 3px solid var(--vp-c-brand-1); outline-offset: 3px; }
</style>
