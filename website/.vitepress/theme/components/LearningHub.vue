<template>
  <div class="learning-hub">
    <!-- docs hub：主要行动（开始学习/查语法）+ 推荐卡 + 按任务选择 -->
    <template v-if="kind === 'docs'">
      <div class="lh-primary">
        <a v-for="card in data.primary" :key="card.key" class="lh-card lh-primary-card" :href="href(card.href)">
          <span class="lh-card-title">{{ t(card.title) }}</span>
          <span class="lh-card-desc">{{ t(card.desc) }}</span>
          <span class="lh-card-go" aria-hidden="true">→</span>
        </a>
      </div>
      <div class="lh-grid">
        <a v-for="card in data.cards" :key="card.key" class="lh-card" :href="href(card.href)">
          <span class="lh-card-title">{{ t(card.title) }}</span>
          <span class="lh-card-desc">{{ t(card.desc) }}</span>
        </a>
      </div>
      <section v-for="group in data.tasks" :key="group.key" class="lh-task">
        <h2 class="lh-task-title">{{ t(group.title) }}</h2>
        <ul class="lh-links">
          <li v-for="item in group.items" :key="item.href + t(item.label)">
            <a :href="href(item.href)">{{ t(item.label) }}</a>
          </li>
        </ul>
      </section>
    </template>

    <!-- books hub：主书优先 + 按背景选择；卡标题→书首页，章节入口→第一章 -->
    <template v-else>
      <div class="lh-book-card lh-book-featured">
        <a class="lh-book-main" :href="href(featured.href)">
          <span class="lh-card-kicker">{{ zh ? '主书' : 'Main book' }}</span>
          <span class="lh-card-title">{{ t(featured.title) }}</span>
          <span class="lh-card-desc">{{ t(featured.blurb) }}</span>
          <span class="lh-card-audience">{{ t(featured.audience) }}</span>
        </a>
        <a class="lh-cta" :href="href(featured.entry)">{{ zh ? '开始阅读第一章' : 'Start reading' }} →</a>
      </div>
      <h2 class="lh-task-title">{{ zh ? '按你的背景选择' : 'Pick by your background' }}</h2>
      <div class="lh-grid">
        <div v-for="book in rest" :key="book.id" class="lh-book-card">
          <a class="lh-book-main" :href="href(book.href)">
            <span class="lh-card-title">{{ t(book.title) }}</span>
            <span class="lh-card-desc">{{ t(book.blurb) }}</span>
            <span class="lh-card-audience">{{ t(book.audience) }}</span>
          </a>
          <a class="lh-cta" :href="href(book.entry)">{{ zh ? '从第一章开始' : 'Start from chapter 1' }} →</a>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
// PLAN-718 T-02：docs/books 学习入口组件。数据来自作者维护的
// website/content/learning-navigation.mjs（生成器同源校验）；纯数据渲染，
// SSR 输出完整卡片文本（静态 HTML 可读、本地搜索可索引），无浏览器 API。
import { computed } from 'vue'
import { useData } from 'vitepress'
import { DOCS_HUB, BOOKS, localeHref } from '../../../content/learning-navigation.mjs'

const props = defineProps<{
  /** 'docs' → 文档入口三层；'books' → 教程入口（主书+背景分组）。 */
  kind: 'docs' | 'books'
}>()

const { lang } = useData()
const zh = computed(() => lang.value.startsWith('zh'))

const data = DOCS_HUB
const featured = computed(() => BOOKS.find((b) => b.featured)!)
const rest = computed(() => BOOKS.filter((b) => !b.featured))

function t(pair: { en: string; zh: string }): string {
  return zh.value ? pair.zh : pair.en
}

function href(path: string): string {
  // 共享 SPA（.html）双语同 URL，不加 zh 前缀（与 data/navigation.ts 同口径）。
  if (path.endsWith('.html')) return path
  return localeHref(path, zh.value)
}
</script>

<style scoped>
.learning-hub {
  margin: 1rem 0 2rem;
}

.lh-primary {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}

.lh-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 0.75rem;
}

.lh-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  padding: 0.9rem 1rem;
  border: 1px solid hsl(var(--border));
  border-radius: var(--site-radius, 0.5rem);
  background: hsl(var(--card));
  color: hsl(var(--foreground));
  text-decoration: none;
  transition: border-color 0.2s ease, box-shadow 0.2s ease, transform 0.2s ease;
}

.lh-card:hover {
  border-color: var(--vp-c-brand-1);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.08);
  transform: translateY(-2px);
}

.lh-card:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 2px;
}

.lh-primary-card {
  border-color: var(--vp-c-brand-soft);
  background: linear-gradient(180deg, var(--vp-c-brand-soft), transparent 60%), hsl(var(--card));
}

.lh-card-kicker {
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--vp-c-brand-1);
}

.lh-card-title {
  font-size: 1.02rem;
  font-weight: 700;
  color: hsl(var(--foreground));
}

.lh-card-desc {
  font-size: 0.88rem;
  line-height: 1.6;
  color: hsl(var(--muted-foreground));
}

.lh-card-audience {
  font-size: 0.8rem;
  color: var(--vp-c-text-2);
}

.lh-card-go {
  position: absolute;
  right: 1rem;
  top: 0.9rem;
  color: var(--vp-c-brand-1);
  font-weight: 700;
}

.lh-cta {
  padding: 0.5rem 1rem;
  border-top: 1px dashed hsl(var(--border));
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--vp-c-brand-1);
  text-decoration: none;
}

.lh-cta:hover {
  text-decoration: underline;
}

.lh-cta:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: -3px;
}

.lh-book-card {
  display: flex;
  flex-direction: column;
  border: 1px solid hsl(var(--border));
  border-radius: var(--site-radius, 0.5rem);
  background: hsl(var(--card));
  transition: border-color 0.2s ease, box-shadow 0.2s ease, transform 0.2s ease;
}

.lh-book-card:hover {
  border-color: var(--vp-c-brand-1);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.08);
  transform: translateY(-2px);
}

.lh-book-main {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  padding: 0.9rem 1rem;
  color: hsl(var(--foreground));
  text-decoration: none;
  border-radius: var(--site-radius, 0.5rem) var(--site-radius, 0.5rem) 0 0;
}

.lh-book-main:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: -3px;
}

.lh-book-featured {
  border-color: var(--vp-c-brand-soft);
  background: linear-gradient(180deg, var(--vp-c-brand-soft), transparent 60%), hsl(var(--card));
  margin-bottom: 1rem;
}

.lh-task {
  margin-top: 1.5rem;
}

.lh-task-title {
  margin: 0 0 0.5rem;
  padding-bottom: 0.3rem;
  border-bottom: 1px solid hsl(var(--border));
  font-size: 1.05rem;
  font-weight: 700;
  color: hsl(var(--foreground));
}

.lh-links {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 0.25rem 1.25rem;
  margin: 0;
  padding: 0;
  list-style: none;
}

.lh-links a {
  display: inline-block;
  padding: 0.3rem 0;
  min-height: 32px;
  color: var(--vp-c-brand-1);
  font-weight: 500;
  text-decoration: none;
}

.lh-links a:hover {
  text-decoration: underline;
}

.lh-links a:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 2px;
  border-radius: 4px;
}

@media (prefers-reduced-motion: reduce) {
  .lh-card,
  .lh-book-card {
    transition: none;
  }
  .lh-card:hover,
  .lh-book-card:hover {
    transform: none;
  }
}
</style>
