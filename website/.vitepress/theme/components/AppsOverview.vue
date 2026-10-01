<script setup lang="ts">
import { computed } from 'vue'
import { useData } from 'vitepress'
import { applicationCopy } from '../data/applications'
const { lang } = useData()
const zh = computed(() => lang.value.startsWith('zh'))
const c = computed(() => applicationCopy(zh.value))
const href = (path: string) => (zh.value ? '/zh' : '') + path
</script>

<template>
  <div class="apps-overview">
    <header class="apps-intro">
      <p class="eyebrow">Auto · Apps</p>
      <h1>{{ c.title }}</h1>
      <p class="lead">{{ c.lead }}</p>
      <nav :aria-label="zh ? '本页入口' : 'On this page'">
        <a href="#main-apps">{{ c.mainTitle }}</a>
        <a href="#system-apps">{{ c.systemTitle }}</a>
      </nav>
    </header>
    <section id="main-apps" aria-labelledby="main-apps-title">
      <h2 id="main-apps-title">{{ c.mainTitle }}</h2>
      <p class="section-lead">{{ c.mainLead }}</p>
      <div class="main-app-grid">
        <article v-for="(app, index) in c.apps" :key="app.key" class="main-app-card">
          <!-- Capture slot: insert the approved product overview image here later. -->
          <div class="app-role"><span>{{ String(index + 1).padStart(2, '0') }}</span>{{ app.role }}</div>
          <h3>{{ app.name }}</h3>
          <p>{{ app.summary }}</p>
          <ul><li v-for="use in app.uses" :key="use">{{ use }}</li></ul>
          <a :href="href('/apps/' + app.key + '/')" :aria-label="c.details + ' · ' + app.name">{{ c.details }} <span aria-hidden="true">→</span></a>
        </article>
      </div>
    </section>
    <section id="system-apps" aria-labelledby="system-apps-title" class="system-app-section">
      <h2 id="system-apps-title">{{ c.systemTitle }}</h2>
      <p class="section-lead">{{ c.systemLead }}</p>
      <dl class="category-list"><div v-for="category in c.categories" :key="category[0]"><dt>{{ category[0] }}</dt><dd>{{ category[1] }}</dd></div></dl>
      <a class="section-link" :href="href('/apps/demos/')">{{ c.systemLink }} <span aria-hidden="true">→</span></a>
    </section>
    <section aria-labelledby="app-ecosystem-title">
      <h2 id="app-ecosystem-title">{{ c.relatedTitle }}</h2>
      <div class="ecosystem-links"><div v-for="item in c.related" :key="item.href"><a :href="href(item.href)">{{ item.name }} <span aria-hidden="true">→</span></a><p>{{ item.text }}</p></div></div>
    </section>
    <section class="app-resources" aria-labelledby="app-resources-title">
      <h2 id="app-resources-title">{{ c.resources }}</h2>
      <p>{{ c.resourceLead }}</p>
      <div class="resource-links"><a v-for="link in c.resourceLinks" :key="link[0]" :href="href(link[0])">{{ link[1] }} <span aria-hidden="true">→</span></a></div>
      <p class="content-date">{{ c.date }}</p>
    </section>
  </div>
</template>

<style scoped>
.apps-overview { max-width: var(--site-max-width); margin: 0 auto; padding: 3.5rem 1.5rem; color: var(--vp-c-text-1); }
.apps-intro { padding-bottom: 3rem; max-width: 850px; }
.eyebrow { color: var(--vp-c-brand-1); font-size: .85rem; font-weight: 600; margin: 0 0 .75rem; }
h1 { font-size: clamp(2rem, 5vw, 3rem); line-height: 1.2; margin: 0 0 1.25rem; }
.lead { font-size: 1.12rem; line-height: 1.85; color: var(--vp-c-text-2); }
nav, .resource-links { display: flex; flex-wrap: wrap; gap: .5rem 1.5rem; }
a { color: var(--vp-c-brand-1); display: inline-flex; align-items: center; gap: .4rem; min-height: 44px; font-weight: 600; }
a:hover { text-decoration: underline; }
a:focus-visible { outline: 3px solid var(--vp-c-brand-1); outline-offset: 4px; border-radius: 3px; }
section { padding: 2.5rem 0; border-top: 1px solid var(--vp-c-divider); scroll-margin-top: calc(var(--site-nav-height) + 16px); }
h2 { font-size: 1.65rem; line-height: 1.4; margin: 0 0 .75rem; }
.section-lead { max-width: var(--site-text-width); color: var(--vp-c-text-2); margin: 0 0 1.75rem; line-height: 1.8; }
.main-app-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 1.25rem; }
.main-app-card { min-width: 0; padding: 1.75rem; border: 1px solid var(--vp-c-divider); border-radius: var(--site-radius); background: var(--vp-c-bg-soft); display: flex; flex-direction: column; }
.app-role { font-size: .85rem; color: var(--vp-c-text-2); display: flex; gap: .75rem; align-items: center; }
.app-role span { color: var(--vp-c-brand-1); font-variant-numeric: tabular-nums; }
h3 { font-size: 1.6rem; line-height: 1.3; margin: .75rem 0; }
.main-app-card p { line-height: 1.8; color: var(--vp-c-text-2); margin: 0; }
ul { list-style: disc; padding-left: 1.25rem; line-height: 1.9; margin: 1rem 0; }
.main-app-card a { align-self: flex-start; margin-top: auto; }
.category-list { margin: 1.5rem 0; }
.category-list div { display: grid; grid-template-columns: 180px minmax(0, 1fr); gap: 1rem; padding: .85rem 0; border-bottom: 1px solid var(--vp-c-divider); }
dt { font-weight: 600; } dd { margin: 0; color: var(--vp-c-text-2); line-height: 1.7; }
.ecosystem-links { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 2rem; }
.ecosystem-links div { min-width: 0; }.ecosystem-links p, .app-resources p { line-height: 1.8; color: var(--vp-c-text-2); }
.app-resources { padding-bottom: 0; }.content-date { font-size: .85rem; margin-top: 1.5rem; }
@media (max-width: 640px) { .apps-overview { padding: 2.5rem 1.25rem; }.main-app-grid, .ecosystem-links { grid-template-columns: 1fr; }.main-app-card { padding: 1.25rem; }.category-list div { grid-template-columns: 1fr; gap: .25rem; }.apps-intro { padding-bottom: 1.5rem; }section { padding: 2rem 0; } }
</style>
