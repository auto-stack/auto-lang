<script setup lang="ts">
// PLAN-715 T-06：v0.5 发布页共享落地组件（EN/ZH 同构渲染）。
// 顺序：主视觉 → 三项亮点 → 桌面 → AutoUI 双端 → 旗舰应用 →
//       系统应用/语言工具链/Playground → 理念及历程 → 展望 → 开始使用。
// 章节导航 SectionNav 吸顶；首图 eager+尺寸占位，其余图 lazy；
// 理念完整论述用原生 details（键盘可开合、hash 可定位）。
import { computed } from 'vue'
import { useRoute } from 'vitepress'
import { RELEASE_V05, releaseNavSections } from '../data/release-v05'
import EvidenceImage from './EvidenceImage.vue'
import ScreenshotGallery from './ScreenshotGallery.vue'
import AutoShellPreview from './AutoShellPreview.vue'
import FeatureCard from './FeatureCard.vue'
import StatCard from './StatCard.vue'
import SectionNav from './SectionNav.vue'

const route = useRoute()
const zh = computed(() => route.path === '/zh' || route.path.startsWith('/zh/'))
const c = computed(() => (zh.value ? RELEASE_V05.zh : RELEASE_V05.en))
const navSections = computed(() => releaseNavSections(zh.value))
const prefix = computed(() => (zh.value ? '/zh' : ''))
const t = computed(() => zh.value ? {
  zoom: '放大查看', close: '关闭', original: '打开原图', gallery: '桌面视图',
  detailsHint: '展开完整论述', arch: 'AutoUI 架构', compare: '同一示例的两端',
  launchCompare: '一源两端对照', flagshipLink: '专题页 →',
} : {
  zoom: 'View full size', close: 'Close', original: 'Open original', gallery: 'Desktop views',
  detailsHint: 'Expand full story', arch: 'AutoUI architecture', compare: 'One example, two ends',
  launchCompare: 'One source, two arms', flagshipLink: 'Landing page →',
})
</script>

<template>
  <div class="release-landing landing-page" style="--page-accent-1: #6366f1; --page-accent-2: #a855f7">

    <!-- 1. 主视觉：短简介 + 双按钮 + 真实桌面主图（eager） -->
    <section class="rel-hero">
      <div class="badge">{{ c.hero.badge }}</div>
      <h1 class="title"><span class="accent">{{ c.hero.titlePre }}</span>{{ c.hero.title }}</h1>
      <p class="description">{{ c.hero.description }}</p>
      <div class="actions">
        <a :href="c.hero.primaryLink" class="btn btn-primary">{{ c.hero.primaryText }}</a>
        <a :href="c.hero.secondaryLink" class="btn btn-secondary">{{ c.hero.secondaryText }}</a>
      </div>
      <figure class="hero-shot">
        <EvidenceImage
          :src="c.hero.heroShot.src"
          :alt="c.hero.heroShot.alt"
          :caption="c.hero.heroShot.caption"
          :zoom-label="t.zoom"
          :close-label="t.close"
          :original-label="t.original"
          loading="eager"
          :width="c.hero.heroShot.width"
          :height="c.hero.heroShot.height"
        />
      </figure>
    </section>

    <SectionNav :items="navSections" :label="zh ? '本页章节' : 'On this page'" />

    <!-- 2. 三项亮点（一句话 + 例子；完整论述在理念区） -->
    <section id="highlights" class="rel-section">
      <h2 class="section-title">{{ c.highlightsLead.title }}</h2>
      <p class="section-desc">{{ c.highlightsLead.desc }}</p>
      <div class="highlights-grid">
        <article
          v-for="h in c.highlights"
          :key="h.name"
          class="highlight-card"
          :style="{ '--ph': h.color, '--ph2': h.color2 }"
        >
          <div class="highlight-icon">{{ h.icon }}</div>
          <h3 class="highlight-name">{{ h.name }}</h3>
          <p class="highlight-sub">{{ h.nameSub }}</p>
          <p class="highlight-tagline">{{ h.tagline }}</p>
          <p class="highlight-example">{{ h.example }}</p>
          <a class="highlight-more" :href="'#philosophy'">{{ t.detailsHint }} ↓</a>
        </article>
      </div>
    </section>

    <!-- 3. 桌面 -->
    <section id="desktop" class="rel-section rel-alt">
      <span class="kicker">{{ c.desktop.kicker }}</span>
      <h2 class="section-title">{{ c.desktop.title }}</h2>
      <p class="section-desc">{{ c.desktop.desc }}</p>
      <div class="desktop-body">
        <p class="narrative">{{ c.desktop.narrative }}</p>
        <ul class="rel-points">
          <li v-for="(p, i) in c.desktop.points" :key="i"><strong>{{ p.split(' — ')[0] }}</strong><span> — {{ p.split(' — ').slice(1).join(' — ') }}</span></li>
        </ul>
      </div>
      <div class="desktop-gallery">
        <ScreenshotGallery
          :shots="c.desktop.shots"
          :group-label="t.gallery"
          :zoom-label="t.zoom"
          :close-label="t.close"
          :original-label="t.original"
          :loading="'eager'"
        />
      </div>
    </section>

    <!-- 4. AutoUI 双端 -->
    <section id="autoui" class="rel-section">
      <div class="autoui-grid">
        <div class="autoui-copy">
          <span class="kicker">{{ c.autoui.badge }}</span>
          <h2 class="autoui-title">{{ c.autoui.title }}</h2>
          <p class="autoui-desc">{{ c.autoui.desc }}</p>
          <p class="narrative">{{ c.autoui.narrative }}</p>
          <ul class="rel-points">
            <li v-for="(p, i) in c.autoui.points" :key="i"><strong>{{ p.split(' — ')[0] }}</strong><span> — {{ p.split(' — ').slice(1).join(' — ') }}</span></li>
          </ul>
          <div class="arch-diagram" :aria-label="t.arch">
            <div class="arch-node arch-src"><strong>{{ c.autoui.arch.src }}</strong><span>{{ c.autoui.arch.srcSub }}</span></div>
            <div class="arch-arms">
              <div v-for="arm in c.autoui.arch.arms" :key="arm.name" class="arch-arm">
                <em>{{ arm.emit }}</em>
                <div class="arch-node"><strong>{{ arm.name }}</strong><span>{{ arm.sub }}</span></div>
              </div>
            </div>
            <div class="arch-foot">{{ c.autoui.arch.foot }}</div>
          </div>
          <div class="gallery-links">
            <a v-for="g in c.autoui.galleryLinks" :key="g.href" :href="g.href" target="_self" class="gallery-link-btn">
              <span class="gl-title">{{ g.title }}</span><span class="gl-sub">{{ g.sub }}</span>
            </a>
          </div>
        </div>
        <div class="autoui-visual">
          <h3 class="visual-caption">{{ t.launchCompare }}</h3>
          <p class="visual-note">{{ c.autoui.compareDesc }}</p>
          <div class="kanban-pair">
            <figure>
              <EvidenceImage
                :src="c.autoui.kanbanWeb.src"
                :alt="c.autoui.kanbanWeb.alt"
                :caption="c.autoui.kanbanWeb.caption"
                :zoom-label="t.zoom" :close-label="t.close" :original-label="t.original"
                :width="c.autoui.kanbanWeb.width" :height="c.autoui.kanbanWeb.height"
              />
              <figcaption>{{ c.autoui.kanbanWeb.label }}</figcaption>
            </figure>
            <figure>
              <EvidenceImage
                :src="c.autoui.kanbanDesktop.src"
                :alt="c.autoui.kanbanDesktop.alt"
                :caption="c.autoui.kanbanDesktop.caption"
                :zoom-label="t.zoom" :close-label="t.close" :original-label="t.original"
                :width="c.autoui.kanbanDesktop.width" :height="c.autoui.kanbanDesktop.height"
              />
              <figcaption>{{ c.autoui.kanbanDesktop.label }}</figcaption>
            </figure>
          </div>
        </div>
      </div>
    </section>

    <!-- 5. 旗舰应用 -->
    <section id="flagship" class="rel-section rel-alt">
      <h2 class="section-title">{{ c.flagship.title }}</h2>
      <p class="section-desc">{{ c.flagship.desc }}</p>
      <div class="flagship-list">
        <article v-for="item in c.flagship.items" :key="item.name" class="flagship-item" :class="{ reverse: item.kind === 'autoshell' }">
          <div class="flagship-text">
            <h3>{{ item.emoji }} {{ item.name }}</h3>
            <p>{{ item.desc }}</p>
            <a v-if="item.href" class="flagship-link" :href="item.href">{{ item.linkLabel }}</a>
          </div>
          <div class="flagship-visual">
            <AutoShellPreview v-if="item.kind === 'autoshell'" :lang="zh ? 'zh' : 'en'" />
            <img v-else-if="item.kind === 'image' && item.image" :src="item.image.src" :alt="item.image.alt" loading="lazy"
              :width="item.image.width" :height="item.image.height" />
            <p v-else class="flagship-noimage">{{ zh ? '（专题页与截图待补齐——不放置占位空图。）' : '(Dedicated page and screenshots pending — no placeholder image.)' }}</p>
          </div>
        </article>
      </div>
    </section>

    <!-- 6. 系统应用 / 语言工具链 / Playground -->
    <section id="ecosystem" class="rel-section">
      <h2 class="section-title">{{ c.systemApps.title }}</h2>
      <p class="section-desc">{{ c.systemApps.desc }} <a :href="c.systemApps.galleryHref" target="_self" class="inline-link">{{ c.systemApps.galleryLabel }}</a></p>
      <div class="rel-cards">
        <FeatureCard v-for="card in c.systemApps.cards" :key="card.title" v-bind="card" color="rgba(99, 102, 241, 0.12)" />
      </div>

      <div class="sub-showcase">
        <div class="sub-copy">
          <span class="kicker">{{ c.playground.badge }}</span>
          <h3 class="sub-title">{{ c.playground.title }}</h3>
          <p class="sub-desc">{{ c.playground.desc }}</p>
          <p class="narrative">{{ c.playground.narrative }}</p>
          <ul class="rel-points">
            <li v-for="(p, i) in c.playground.points" :key="i"><strong>{{ p.split(' — ')[0] }}</strong><span> — {{ p.split(' — ').slice(1).join(' — ') }}</span></li>
          </ul>
          <div class="rel-cards rel-cards-tight">
            <StatCard v-for="s in c.playground.stats" :key="s.label" v-bind="s" />
          </div>
        </div>
      </div>

      <div class="sub-showcase">
        <div class="sub-copy">
          <span class="kicker">{{ c.language.badge }}</span>
          <h3 class="sub-title">{{ c.language.title }}</h3>
          <p class="sub-desc">{{ c.language.desc }}</p>
          <p class="narrative">{{ c.language.narrative }}</p>
          <ul class="rel-points">
            <li v-for="(p, i) in c.language.points" :key="i"><strong>{{ p.split(' — ')[0] }}</strong><span> — {{ p.split(' — ').slice(1).join(' — ') }}</span></li>
          </ul>
        </div>
        <div class="matrix" aria-label="bootstrap matrix">
          <div class="matrix-head">{{ c.language.matrix.head }}</div>
          <div class="matrix-grid">
            <div v-for="cell in c.language.matrix.cells" :key="cell.name" class="matrix-cell" :class="cell.kind">
              <strong>{{ cell.name }}</strong><span>{{ cell.sub }}</span>
            </div>
          </div>
          <div class="matrix-foot">{{ c.language.matrix.foot }}</div>
        </div>
      </div>

      <h3 class="sub-title toolchain-title">{{ c.toolchain.title }}</h3>
      <p class="sub-desc">{{ c.toolchain.desc }}</p>
      <div class="rel-cards">
        <FeatureCard v-for="card in c.toolchain.cards" :key="card.title" v-bind="card" color="rgba(99, 102, 241, 0.12)" />
      </div>
      <p class="section-desc"><a :href="c.toolchain.notesHref" class="inline-link">{{ c.toolchain.notesLabel }}</a></p>
    </section>

    <!-- 7. 理念及历程（统计后移至此；完整论述 details 展开） -->
    <section id="philosophy" class="rel-section rel-alt">
      <h2 class="section-title">{{ c.philosophy.title }}</h2>
      <p class="section-desc">{{ c.philosophy.desc }}</p>
      <div class="philosophy-details">
        <details v-for="h in c.highlights" :key="h.name" class="philosophy-item" :id="'ph-' + h.nameSub.split(' ')[0].toLowerCase()">
          <summary>{{ h.icon }} {{ h.name }} — {{ h.tagline }}</summary>
          <div class="philosophy-body">
            <p class="philosophy-proof-line">{{ h.proof }}</p>
            <ul>
              <li v-for="(d, i) in h.details" :key="i">{{ d }}</li>
            </ul>
          </div>
        </details>
      </div>

      <h3 class="sub-title journey-title">{{ c.philosophy.journeyTitle }}</h3>
      <p class="sub-desc">{{ c.philosophy.journeyDesc }}</p>
      <div class="journey">
        <p v-for="(lead, i) in c.philosophy.leads" :key="i" class="journey-lead">{{ lead }}</p>
        <div class="rel-cards rel-cards-tight stats-row">
          <StatCard v-for="s in c.philosophy.stats" :key="s.label" v-bind="s" />
        </div>
        <p class="hero-stats-note">{{ c.philosophy.statsNote }}</p>
        <div class="timeline">
          <div v-for="step in c.philosophy.timeline" :key="step.version" class="timeline-item">
            <span class="timeline-version">{{ step.version }}</span>
            <div class="timeline-card">
              <h4>{{ step.title }}</h4>
              <p>{{ step.text }}</p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- 8. 展望 -->
    <section id="roadmap" class="rel-section">
      <h2 class="section-title">{{ c.roadmap.title }}</h2>
      <p class="section-desc">{{ c.roadmap.desc }}</p>
      <div class="rel-cards">
        <FeatureCard v-for="card in c.roadmap.cards" :key="card.title" v-bind="card" color="rgba(99, 102, 241, 0.12)" />
      </div>
    </section>

    <!-- 9. 开始使用 -->
    <section id="get-started" class="cta-section">
      <h2 class="section-title">{{ c.cta.title }}</h2>
      <p class="section-desc">{{ c.cta.desc }}</p>
      <div class="cta-actions">
        <a :href="c.cta.primaryLink" class="cta-btn cta-primary">{{ c.cta.primaryText }}</a>
        <a :href="c.cta.secondaryLink" class="cta-btn cta-secondary">{{ c.cta.secondaryText }}</a>
      </div>
    </section>
  </div>
</template>

<style scoped>
.release-landing {
  position: relative;
}

.rel-section {
  padding: 4rem 2rem;
  max-width: var(--site-max-width, 1200px);
  margin: 0 auto;
  scroll-margin-top: calc(var(--site-nav-height, 56px) + 56px);
}

.rel-alt {
  background: hsl(var(--muted) / 0.35);
  max-width: none;
  border-top: 1px solid hsl(var(--border));
  border-bottom: 1px solid hsl(var(--border));
}

.rel-alt > * {
  max-width: var(--site-max-width, 1200px);
  margin-left: auto;
  margin-right: auto;
}

/* hero */
.rel-hero {
  position: relative;
  overflow: hidden;
  padding: 4.5rem 2rem 3rem;
  text-align: center;
}

.rel-hero .description {
  max-width: var(--site-text-width, 720px);
  margin-left: auto;
  margin-right: auto;
}

.hero-shot {
  margin: 2.5rem auto 0;
  max-width: 1080px;
  text-align: left;
}

/* highlights */
.highlights-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 300px), 1fr));
  gap: 1.25rem;
  align-items: stretch;
}

.highlight-card {
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
  padding: 1.6rem 1.4rem 1.4rem 1.7rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border) / 0.7);
  background: hsl(var(--card));
  min-width: 0;
}

.highlight-card::before {
  content: '';
  position: absolute;
  left: 0; top: 0; bottom: 0;
  width: 4px;
  background: linear-gradient(180deg, var(--ph, #6366f1), var(--ph2, #a855f7));
}

.highlight-icon {
  width: 44px; height: 44px;
  border-radius: 10px;
  display: flex; align-items: center; justify-content: center;
  font-size: 1.3rem;
  background: color-mix(in srgb, var(--ph) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--ph) 25%, transparent);
}

.highlight-name {
  margin: 0;
  font-size: 1.3rem;
  font-weight: 700;
  color: hsl(var(--foreground));
}

.highlight-sub {
  margin: -0.4rem 0 0;
  font-family: var(--vp-font-family-mono);
  font-size: 0.68rem;
  letter-spacing: 0.12em;
  color: var(--ph);
}

.highlight-tagline {
  margin: 0;
  font-size: 1rem;
  line-height: 1.75;
  color: hsl(var(--foreground) / 0.9);
}

.highlight-example {
  margin: 0;
  font-family: var(--vp-font-family-mono);
  font-size: 0.76rem;
  line-height: 1.7;
  color: var(--ph);
}

.highlight-more {
  margin-top: auto;
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--ph);
  text-decoration: none;
}

.highlight-more:hover {
  text-decoration: underline;
}

.highlight-more:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 2px;
}

/* desktop */
.desktop-body {
  max-width: var(--site-text-width, 720px);
  margin: 0 auto;
  text-align: left;
}

.desktop-gallery {
  max-width: 1080px;
  margin: 2rem auto 0;
}

/* autoui */
.autoui-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 3rem;
  align-items: start;
}

.autoui-copy { min-width: 0; }

.autoui-title {
  font-size: clamp(1.5rem, 3vw, 2rem);
  font-weight: 700;
  margin: 0.5rem 0;
  color: hsl(var(--foreground));
}

.autoui-desc {
  font-size: 1rem;
  line-height: 1.75;
  color: hsl(var(--muted-foreground));
  margin: 0 0 1rem;
}

.autoui-visual { min-width: 0; }

.visual-caption {
  font-size: 1.05rem;
  font-weight: 700;
  margin: 0 0 0.4rem;
  color: hsl(var(--foreground));
}

.visual-note {
  font-size: 0.85rem;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
  margin: 0 0 1rem;
}

.kanban-pair {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 1.25rem;
}

.kanban-pair figure {
  margin: 0;
}

.kanban-pair figcaption {
  margin-top: 0.4rem;
  text-align: center;
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--page-accent-1);
}

/* flagship */
.flagship-list {
  display: flex;
  flex-direction: column;
  gap: 2.25rem;
  margin: 2rem auto 0;
}

.flagship-item {
  display: grid;
  grid-template-columns: 5fr 7fr;
  gap: 2rem;
  align-items: center;
}

.flagship-item.reverse {
  grid-template-columns: 7fr 5fr;
}

.flagship-item.reverse .flagship-text { order: 2; }
.flagship-item.reverse .flagship-visual { order: 1; }

.flagship-text { min-width: 0; }

.flagship-text h3 {
  margin: 0 0 0.5rem;
  font-size: 1.3rem;
  color: hsl(var(--foreground));
}

.flagship-text p {
  margin: 0 0 0.75rem;
  font-size: 0.95rem;
  line-height: 1.8;
  color: hsl(var(--muted-foreground));
}

.flagship-link {
  font-size: 0.88rem;
  font-weight: 600;
  color: var(--page-accent-1);
  text-decoration: none;
}

.flagship-link:hover { text-decoration: underline; }
.flagship-link:focus-visible { outline: 3px solid var(--vp-c-brand-1); outline-offset: 2px; }

.flagship-visual { min-width: 0; }

.flagship-visual > img {
  width: 100%;
  height: auto;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.28);
  display: block;
}

.flagship-noimage {
  padding: 1rem;
  border: 1px dashed hsl(var(--border));
  border-radius: var(--radius);
  color: hsl(var(--muted-foreground));
  font-size: 0.85rem;
  text-align: center;
}

/* ecosystem shared blocks */
.rel-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 300px), 1fr));
  gap: 1.5rem;
}

.rel-cards-tight {
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 220px), 1fr));
  gap: 1rem;
}

.inline-link {
  color: var(--vp-c-brand-1);
  font-weight: 600;
  text-decoration: none;
}

.inline-link:hover { text-decoration: underline; }
.inline-link:focus-visible { outline: 3px solid var(--vp-c-brand-1); outline-offset: 2px; }

.sub-showcase {
  margin-top: 3rem;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 2.5rem;
  align-items: start;
}

.sub-copy { min-width: 0; }

.sub-title {
  font-size: 1.4rem;
  font-weight: 700;
  margin: 0.5rem 0 0.5rem;
  color: hsl(var(--foreground));
}

.toolchain-title { margin-top: 3.5rem; }

.sub-desc {
  font-size: 1rem;
  line-height: 1.75;
  color: hsl(var(--muted-foreground));
  margin: 0 0 1rem;
}

/* matrix */
.matrix { min-width: 0; width: 100%; max-width: 440px; margin: 0 auto; }

.matrix-head, .matrix-foot {
  font-size: 0.85rem;
  color: hsl(var(--muted-foreground));
  text-align: center;
  margin-bottom: 0.75rem;
}

.matrix-foot { margin: 0.75rem 0 0; color: #a855f7; font-weight: 600; }

.matrix-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
}

.matrix-cell {
  padding: 1.25rem 1rem;
  border-radius: var(--radius);
  text-align: center;
  border: 1px solid hsl(var(--border));
  min-width: 0;
}

.matrix-cell strong {
  display: block;
  font-size: 1.5rem;
  font-family: var(--vp-font-family-mono);
  color: hsl(var(--foreground));
}

.matrix-cell span { font-size: 0.8rem; color: hsl(var(--muted-foreground)); }
.matrix-cell.host { background: rgba(99, 102, 241, 0.08); border-color: rgba(99, 102, 241, 0.3); }
.matrix-cell.boot { background: rgba(168, 85, 247, 0.1); border-color: rgba(168, 85, 247, 0.4); }

/* philosophy */
.philosophy-details {
  max-width: 860px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
}

.philosophy-item {
  border: 1px solid hsl(var(--border));
  border-radius: var(--radius);
  background: hsl(var(--card));
  scroll-margin-top: calc(var(--site-nav-height, 56px) + 64px);
}

.philosophy-item summary {
  padding: 1rem 1.25rem;
  cursor: pointer;
  font-weight: 600;
  font-size: 0.98rem;
  line-height: 1.7;
  color: hsl(var(--foreground));
  border-radius: var(--radius);
  min-height: 44px;
}

.philosophy-item summary:hover { background: hsl(var(--accent) / 0.6); }

.philosophy-item summary:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 2px;
}

.philosophy-item[open] summary {
  border-bottom: 1px dashed hsl(var(--border));
}

.philosophy-body {
  padding: 1rem 1.5rem 1.25rem;
}

.philosophy-proof-line {
  font-family: var(--vp-font-family-mono);
  font-size: 0.78rem;
  color: var(--page-accent-1);
  margin: 0 0 0.75rem;
}

.philosophy-body ul {
  list-style: disc;
  padding-left: 1.3rem;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.philosophy-body li {
  font-size: 0.9rem;
  line-height: 1.75;
  color: hsl(var(--muted-foreground));
}

/* journey */
.journey { max-width: 820px; margin: 1.5rem auto 0; }

.journey-title { margin-top: 3rem; }

.journey-lead {
  margin: 0 0 1.1rem;
  font-size: 1.02rem;
  line-height: 1.9;
  color: hsl(var(--foreground));
}

.stats-row { margin: 1.5rem 0 0; }

.hero-stats-note {
  margin: 0.75rem 0 0;
  font-size: 0.85rem;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
}

.timeline {
  position: relative;
  margin-top: 2rem;
  padding-left: 1.4rem;
  display: flex;
  flex-direction: column;
  gap: 1.4rem;
}

.timeline::before {
  content: '';
  position: absolute;
  left: 0; top: 6px; bottom: 6px;
  width: 2px;
  background: linear-gradient(180deg, var(--page-accent-1, #6366f1), var(--page-accent-2, #a855f7));
  opacity: 0.35;
  border-radius: 2px;
}

.timeline-item { position: relative; }

.timeline-item::before {
  content: '';
  position: absolute;
  left: -1.4rem;
  top: 0.45rem;
  width: 10px; height: 10px;
  margin-left: -4px;
  border-radius: 50%;
  background: hsl(var(--background));
  border: 2px solid var(--page-accent-1, #6366f1);
}

.timeline-version {
  display: inline-block;
  font-family: var(--vp-font-family-mono);
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  color: var(--page-accent-1, #6366f1);
  margin-bottom: 0.2rem;
}

.timeline-card {
  padding: 0.9rem 1.2rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border) / 0.7);
  background: hsl(var(--card));
}

.timeline-card h4 {
  margin: 0 0 0.3rem;
  font-size: 1.02rem;
  color: hsl(var(--foreground));
}

.timeline-card p {
  margin: 0;
  font-size: 0.9rem;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
}

/* shared bits mirrored from landing.css usage in old page */
.kicker {
  display: inline-block;
  padding: 0.3rem 0.85rem;
  border-radius: 9999px;
  background: color-mix(in srgb, var(--page-accent-1, #6366f1) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--page-accent-1, #6366f1) 20%, transparent);
  color: var(--page-accent-1, #6366f1);
  font-size: 0.8rem;
  font-weight: 600;
}

.narrative {
  margin: 0 0 0.75rem;
  padding-left: 0.9rem;
  border-left: 2px solid;
  border-image: linear-gradient(180deg, var(--page-accent-1, #6366f1), var(--page-accent-2, #a855f7)) 1;
  font-size: 0.95rem;
  line-height: 1.8;
  color: hsl(var(--foreground) / 0.92);
}

.rel-points {
  list-style: none;
  padding: 0;
  margin: 0.75rem 0 0;
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
}

.rel-points li {
  position: relative;
  padding-left: 1.3rem;
  font-size: 0.9rem;
  line-height: 1.75;
  color: hsl(var(--muted-foreground));
  min-width: 0;
}

.rel-points li::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0.55rem;
  width: 6px; height: 6px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--page-accent-1, #6366f1), var(--page-accent-2, #a855f7));
}

.rel-points strong { color: hsl(var(--foreground)); }

.arch-diagram {
  width: 100%;
  max-width: 440px;
  margin-top: 1.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
}

.arch-node {
  width: 100%;
  padding: 0.9rem 1rem;
  border-radius: var(--radius);
  text-align: center;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  min-width: 0;
}

.arch-node strong {
  display: block;
  font-family: var(--vp-font-family-mono);
  font-size: 1.05rem;
  color: hsl(var(--foreground));
}

.arch-node span { font-size: 0.78rem; color: hsl(var(--muted-foreground)); }
.arch-src { border-color: rgba(99, 102, 241, 0.4); background: rgba(99, 102, 241, 0.08); }

.arch-arms {
  width: 100%;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
}

.arch-arm {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.4rem;
  min-width: 0;
}

.arch-arm em {
  font-style: normal;
  font-family: var(--vp-font-family-mono);
  font-size: 0.72rem;
  color: hsl(var(--muted-foreground));
}

.arch-foot {
  font-size: 0.82rem;
  font-weight: 600;
  color: #6366f1;
  text-align: center;
}

.gallery-links {
  width: 100%;
  max-width: 440px;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-top: 1rem;
}

.gallery-link-btn {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.7rem 1rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  font-size: 0.9rem;
  font-weight: 600;
  color: hsl(var(--foreground));
  text-decoration: none;
  transition: transform 0.15s ease, border-color 0.15s ease;
  min-width: 0;
}

.gallery-link-btn:hover {
  transform: translateY(-2px);
  border-color: rgba(99, 102, 241, 0.5);
  text-decoration: none;
}

.gallery-link-btn:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 2px;
}

.gl-sub {
  font-size: 0.75rem;
  font-weight: 400;
  color: hsl(var(--muted-foreground));
  text-align: right;
}

.section-title {
  position: relative;
  display: inline-block;
}

.section-title::after {
  content: '';
  display: block;
  width: 42px;
  height: 3px;
  margin: 0.55rem auto 0;
  border-radius: 2px;
  background: linear-gradient(90deg, var(--page-accent-1, #6366f1), var(--page-accent-2, #a855f7));
}

.rel-alt .section-title {
  display: block;
  text-align: center;
}

@media (max-width: 900px) {
  .autoui-grid,
  .sub-showcase {
    grid-template-columns: 1fr;
    gap: 1.5rem;
  }

  .flagship-item,
  .flagship-item.reverse {
    grid-template-columns: 1fr;
    gap: 1rem;
  }

  .flagship-item.reverse .flagship-text { order: 1; }
  .flagship-item.reverse .flagship-visual { order: 2; }

  .rel-section { padding: 3rem 1.25rem; }
}

@media (prefers-reduced-motion: reduce) {
  .gallery-link-btn { transition: none !important; transform: none !important; }
}
</style>
