<template>
  <div v-if="book" class="reader-context">
    <span class="rc-book">{{ zh ? '📖 ' + t(book.title) : '📖 ' + t(book.title) }}</span>
    <a class="rc-toc" :href="href(book.href)">{{ zh ? '本书目录' : 'Book contents' }}</a>
  </div>
</template>

<script setup lang="ts">
// PLAN-718 T-03：书页阅读上下文（计划 §5.2"回当前书目录入口"）。仅在
// /books/<id>/ 章节页（非书目首页）显示一条简短的书籍/目录上下文；docs 页与
// hub 不渲染。数据与 LearningHub 同源 learning-navigation.mjs。
import { computed } from 'vue'
import { useData, useRoute } from 'vitepress'
import { BOOKS, localeHref } from '../../../content/learning-navigation.mjs'

const route = useRoute()
const { lang } = useData()
const zh = computed(() => lang.value.startsWith('zh'))

const book = computed(() => {
  // route.path 可能带 .html/index/尾斜杠（navigation.ts normalizePath 同款归一）。
  // 仅章节路径命中（书目首页归一后恰为 /books/<id>，不显示）。
  const norm = route.path.replace(/\.html$/, '').replace(/\/index$/, '').replace(/\/$/, '')
  const m = norm.match(/^\/(zh\/)?books\/([^/]+)$/)
  if (m) return null
  const idMatch = norm.match(/^\/(zh\/)?books\/([^/]+)/)
  if (!idMatch) return null
  return BOOKS.find((b) => b.id === idMatch[2]) ?? null
})

function t(pair: { en: string; zh: string }): string {
  return zh.value ? pair.zh : pair.en
}

function href(path: string): string {
  return localeHref(path, zh.value)
}
</script>

<style scoped>
.reader-context {
  display: flex;
  align-items: baseline;
  gap: 0.75rem;
  margin: 0 0 0.75rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px dashed hsl(var(--border));
  font-size: 0.85rem;
}

.rc-book {
  color: hsl(var(--muted-foreground));
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rc-toc {
  flex-shrink: 0;
  margin-left: auto;
  min-height: 32px;
  display: inline-flex;
  align-items: center;
  color: var(--vp-c-brand-1);
  font-weight: 600;
  text-decoration: none;
}

.rc-toc:hover {
  text-decoration: underline;
}

.rc-toc:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 2px;
  border-radius: 4px;
}
</style>
