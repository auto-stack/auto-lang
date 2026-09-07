---
layout: home
---

<script setup>
import FeatureCard from '../../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #6366f1; --page-accent-2: #14b8a6">

<div class="landing-hero">
  <div class="badge">AutoOS Flagship App · Alpha</div>
  <h1 class="title">AutoDown: <span class="accent">A Knowledge Base That Thinks</span></h1>
  <p class="description">
    A Markdown+YAML dialect of the Auto language, and Jade Garden, an Obsidian-like knowledge base built with it.
    Frontend and backend logic are all one .at source — the largest showcase of "one codebase, two shapes" in the Auto ecosystem.
  </p>
  <div class="actions">
    <a href="/v05/" class="btn btn-primary">Back to v0.5 Release Highlights</a>
    <a href="/apps" class="btn btn-secondary">View All Apps</a>
  </div>
</div>

<div class="stats-section">
  <h2 class="section-title">AutoDown at a Glance</h2>
  <div class="stats-grid">
    <StatCard value="32K+" label="Lines of .at Source" description="411 .at files — the largest Auto codebase in the ecosystem." color="#6366f1" />
    <StatCard value="2" label="Runtime Shapes" description="One source: Web (Vue) and desktop (iced) shapes." color="#14b8a6" />
    <StatCard value="23" label="e2e Parity Checks" description="Verified equal across Playwright's dual backends (vue/VM)." color="#8b5cf6" />
    <StatCard value="9+" label="Backend Modules" description="parser/links/search/tasks/agenda/query/srs/linkgraph…" color="#ec4899" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="Jade Garden: One Source Tree, Two Shapes"
    description="29 widgets are generated as Vue SFCs from .at via a2ts; the same DSL is interpreted by the AutoVM and rendered natively by iced. Web and desktop aren't developed twice — they're developed once."
    badge="Dual Shape"
  >
    <ul>
      <li><strong>Three-pane layout</strong> — source tree, editor, and preview, linked.</li>
      <li><strong>Dark & light themes</strong> — full dark/light support on desktop.</li>
      <li><strong>Streaming demo</strong> — edit/view/stream showcase modes.</li>
    </ul>
    <template #visual>
      <div class="shot-stack">
        <img src="/v05/autodown-desktop.png" alt="Jade Garden desktop shape (iced)" class="shot-main" />
        <div class="shot-pair">
          <img src="/v05/autodown-web.png" alt="Jade Garden web shape (Vue)" />
        </div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Documents as Data"
    description="AutoDown documents can be parsed, queried, transformed, and generated — a knowledge base isn't just for humans to read."
    badge="Dialect"
    reverse
  >
    <ul>
      <li><strong>Markdown + YAML</strong> — familiar writing syntax with structured-data power.</li>
      <li><strong>Knowledge graph</strong> — the links and linkgraph modules build a citation network across documents.</li>
      <li><strong>Tasks & agenda</strong> — the tasks/agenda modules turn TODOs into queryable data.</li>
      <li><strong>Spaced repetition</strong> — the srs module turns notes into study cards.</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">note.down — Markdown + YAML</span>
        </div>
        <pre class="code-body"><code><span class="keyword">meta</span>:
  <span class="function">title</span>: <span class="string">"v0.5 Release Retro"</span>
  <span class="function">tags</span>: [release, v05]
<span class="keyword">#</span> Release by the Numbers
<span class="function">commits</span>: 5711
<span class="function">highlights</span>:
  - AutoOS Virtual Desktop
  - Bootstrap² matrix achieved</code></pre>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">Why AutoDown Matters</h2>
  <div class="features-grid">
    <FeatureCard icon="🌱" title="Single Source of Truth" description="Frontend and backend logic all in .at, with Rust as just the shell — the highest form of full-stack Auto." color="rgba(99, 102, 241, 0.15)" />
    <FeatureCard icon="🔁" title="Dual-Backend Parity" description="23 e2e checks hold equally on the vue and VM backends; consistency is locked in by tests." color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🧠" title="Knowledge as Data" description="Parseable, queryable, generatable — the knowledge base is AutoOS's memory layer." color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🛠️" title="Same-Stack Toolchain" description="Built with the same compiler, VM, and transpilers as the language itself — no special channels." color="rgba(245, 158, 11, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Keep Exploring</h2>
  <div class="cta-actions">
    <a href="/v05/" class="cta-btn cta-primary">Back to v0.5 Release Highlights</a>
    <a href="/apps" class="cta-btn cta-secondary">View All Apps</a>
  </div>
</div>

</div>
