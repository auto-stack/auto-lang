<script setup lang="ts">
import { computed, ref } from 'vue'
import { useData } from 'vitepress'
import rows from '../data/demos.json'
import EvidenceImage from './EvidenceImage.vue'

const { lang } = useData()
const zh = computed(() => lang.value.startsWith('zh'))
const selected = ref('all')
const groups = [
  { id: 'daily', zh: '工作与生活', en: 'Work and daily life' },
  { id: 'reading', zh: '阅读与交流', en: 'Reading and communication' },
  { id: 'media', zh: '媒体与创作', en: 'Media and creation' },
  { id: 'system', zh: '系统与数据', en: 'System and data' },
  { id: 'games', zh: '小游戏', en: 'Games' },
  { id: 'structure', zh: '应用结构', en: 'Application structure' },
]
const visibleGroups = computed(() => groups.filter(group => selected.value === 'all' || group.id === selected.value))
const count = computed(() => selected.value === 'all' ? rows.length : rows.filter(row => row.category === selected.value).length)
const copy = (row: typeof rows[number]) => zh.value ? row.zh : row.en
const href = (path: string) => (zh.value ? '/zh' : '') + path
</script>

<template>
  <main class="demo-directory">
    <header class="demo-intro">
      <p class="eyebrow">Auto · Apps</p>
      <h1>{{ zh ? '系统应用与 Demo' : 'System applications and demos' }}</h1>
      <p class="lead">{{ zh ? '从日常工具认识 Auto 应用：先看真实界面，再了解用途、操作和运行条件。这份目录收录 28 个应用与示例，它们的系统集成和完成度分别说明。' : 'Explore Auto applications through real interfaces, then learn their purpose, operations, and runtime requirements. This catalog covers 28 applications and examples, with integration and readiness described individually.' }}</p>
      <nav :aria-label="zh ? '相关介绍' : 'Related introductions'">
        <a :href="href('/apps')">{{ zh ? '四个主应用' : 'Four main applications' }} →</a>
        <a :href="href('/autoos/')">{{ zh ? 'AutoOS 虚拟桌面' : 'AutoOS virtual desktop' }} →</a>
      </nav>
    </header>
    <div class="demo-filters" role="group" :aria-label="zh ? '按用途筛选' : 'Filter by purpose'">
      <button type="button" :aria-pressed="selected === 'all'" @click="selected = 'all'">{{ zh ? '全部' : 'All' }} <span>{{ rows.length }}</span></button>
      <button v-for="group in groups" :key="group.id" type="button" :aria-pressed="selected === group.id" @click="selected = group.id">{{ zh ? group.zh : group.en }} <span>{{ rows.filter(row => row.category === group.id).length }}</span></button>
    </div>
    <p class="result-count" aria-live="polite" aria-atomic="true">{{ zh ? `显示 ${count} 项 · 点击图片可以放大` : `${count} items · select an image to enlarge it` }}</p>
    <section v-for="group in visibleGroups" :id="group.id" :key="group.id" class="demo-category" :aria-labelledby="'demo-' + group.id">
      <h2 :id="'demo-' + group.id">{{ zh ? group.zh : group.en }}</h2>
      <div class="demo-grid">
        <article v-for="row in rows.filter(item => item.category === group.id)" :key="row.id" class="demo-card" :data-demo-id="row.id">
          <EvidenceImage :src="row.image.src" :alt="copy(row).name + (zh ? '真实运行界面' : ' actual running interface')" :caption="copy(row).caption" :width="row.image.width" :height="row.image.height" :framed="false" :zoom-label="zh ? '放大图片' : 'Enlarge image'" :close-label="zh ? '关闭' : 'Close'" :original-label="zh ? '查看原图' : 'View original'" />
          <div class="demo-card-body">
            <h3><a :href="href('/apps/demos/' + row.slug + '/')">{{ copy(row).name }}</a></h3>
            <p>{{ copy(row).summary }}</p>
            <a class="demo-detail-link" :href="href('/apps/demos/' + row.slug + '/')">{{ zh ? '用途与操作' : 'Purpose and operations' }} <span aria-hidden="true">→</span></a>
          </div>
        </article>
      </div>
    </section>
    <section class="demo-context" aria-labelledby="demo-context-title">
      <h2 id="demo-context-title">{{ zh ? '怎样继续了解' : 'Ways to explore further' }}</h2>
      <p>{{ zh ? 'v0.5 的这组页面提供静态介绍与实拍图。网页虚拟桌面与应用在线体验安排在 v0.5.1；能显示 UI 的 Playground 将先从少量示例验证，再考虑接入更多应用。' : 'These v0.5 pages provide static introductions and real captures. The web virtual desktop and online app experiences are planned for v0.5.1. A UI Playground will first be evaluated with a few examples before broader application integration.' }}</p>
      <p>{{ zh ? '照片、视频与文件应用需要实际资源，监视器与终端需要宿主能力。有些应用使用内置演示数据，逐项介绍会说明；这 28 项不是同一个完成度的产品清单，也不等于桌面注册应用的固定数量。' : 'Photo, video, and file tools need real resources; monitoring and terminals need host capabilities. Some examples use built-in data, identified in their introductions. These 28 entries have different readiness levels and do not define a fixed desktop registry count.' }}</p>
      <a href="/ui/demos/">{{ zh ? '开发示例画廊（部分功能需要后台）' : 'Development example gallery (some features need a backend)' }} →</a>
      <p class="material-date">{{ zh ? '介绍资料时点：2026-10-01。新拍图注明运行形态；留存图另注明来源与版本。' : 'Descriptions as of 2026-10-01. New captures identify their runtime; retained images identify their origin and version separately.' }}</p>
    </section>
  </main>
</template>

<style scoped>
.demo-directory { max-width: var(--site-max-width); margin: 0 auto; padding: 3.5rem 1.5rem; color: var(--vp-c-text-1); }
.demo-intro { max-width: 850px; padding-bottom: 2rem; }
.eyebrow { margin: 0 0 .75rem; color: var(--vp-c-brand-1); font-size: .85rem; font-weight: 600; }
h1 { font-size: clamp(2rem, 5vw, 3rem); line-height: 1.2; margin: 0 0 1.25rem; }
.lead { color: var(--vp-c-text-2); font-size: 1.12rem; line-height: 1.85; }
nav, .demo-filters { display: flex; flex-wrap: wrap; gap: .5rem 1rem; }
a { color: var(--vp-c-brand-1); font-weight: 600; }
nav a, .demo-context > a, .demo-detail-link { display: inline-flex; align-items: center; min-height: 44px; gap: .5rem; }
a:hover { text-decoration: underline; }
a:focus-visible, button:focus-visible { outline: 3px solid var(--vp-c-brand-1); outline-offset: 4px; }
.demo-filters { border-top: 1px solid var(--vp-c-divider); padding-top: 1.5rem; }
.demo-filters button { min-height: 44px; border: 1px solid var(--vp-c-divider); padding: .5rem .9rem; border-radius: 8px; font-size: .85rem; text-align: left; background: var(--vp-c-bg-soft); }
.demo-filters button[aria-pressed="true"] { border-color: var(--vp-c-brand-1); color: var(--vp-c-brand-1); background: var(--vp-c-brand-soft); }
.demo-filters span { margin-left: .4rem; font-variant-numeric: tabular-nums; opacity: .75; }
.result-count, .material-date { color: var(--vp-c-text-2); font-size: .85rem; line-height: 1.7; }
.result-count { margin: 1rem 0 0; }
.demo-category { padding: 2rem 0; scroll-margin-top: calc(var(--site-nav-height) + 16px); }
h2 { margin: 0 0 1.25rem; font-size: 1.6rem; line-height: 1.4; }
.demo-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 1.25rem; }
.demo-card { min-width: 0; display: flex; flex-direction: column; border: 1px solid var(--vp-c-divider); border-radius: 14px; overflow: hidden; background: var(--vp-c-bg-soft); }
.demo-card :deep(.evidence-image) { border: 0; border-radius: 0; }
.demo-card :deep(.evidence-image img) { height: auto; aspect-ratio: 16 / 10 !important; object-fit: contain; background: var(--vp-c-bg-alt); }
.demo-card :deep(figcaption) { padding: 0 1.25rem; margin: .75rem 0 0; }
.demo-card :deep(.zoom-label) { padding: .65rem 1.25rem; }
.demo-card-body { flex: 1; padding: 1rem 1.25rem 1.25rem; display: flex; flex-direction: column; }
h3 { font-size: 1.25rem; line-height: 1.5; margin: 0 0 .6rem; }
h3 a { color: var(--vp-c-text-1); }
.demo-card-body p { margin: 0 0 1rem; color: var(--vp-c-text-2); font-size: .93rem; line-height: 1.8; }
.demo-detail-link { margin-top: auto; align-self: flex-start; font-size: .9rem; }
.demo-context { margin-top: 1.5rem; padding-top: 2rem; border-top: 1px solid var(--vp-c-divider); }
.demo-context p { max-width: var(--site-text-width); color: var(--vp-c-text-2); line-height: 1.8; }
@media (max-width: 1023px) { .demo-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
@media (max-width: 640px) { .demo-directory { padding: 2.5rem 1.25rem; }.demo-grid { grid-template-columns: 1fr; }.demo-filters { gap: .5rem; }.demo-category { padding: 1.5rem 0; } }
</style>
