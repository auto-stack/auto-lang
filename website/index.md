---
layout: home
---

<script setup>
import { introCopy } from './.vitepress/theme/data/os-ai-introduction'
const copy = introCopy(false)
import { onMounted } from 'vue'
import HomeHero from './.vitepress/theme/components/HomeHero.vue'
import HomeDemo from './.vitepress/theme/components/HomeDemo.vue'
import AutoShellPreview from './.vitepress/theme/components/AutoShellPreview.vue'
import FeatureCard from './.vitepress/theme/components/FeatureCard.vue'
import ScreenshotSlot from './.vitepress/theme/components/ScreenshotSlot.vue'
import { applicationCopy } from './.vitepress/theme/data/applications'
const applications = applicationCopy(false).apps
const captureIds = { autoedit: 'SHOT-01', automusk: 'SHOT-04', jadeedit: 'SHOT-07' }
const icons = ['🌐', '🦀', '🐍', '🎨', '🤖', '💻']
onMounted(() => {
  if (sessionStorage.getItem('auto-lang-checked')) return;
  sessionStorage.setItem('auto-lang-checked', '1');
  if (navigator.language.toLowerCase().startsWith('zh')) {
    window.location.replace('/zh/');
  }
});
</script>

<div class="landing-page" style="--page-accent-1: #6366f1; --page-accent-2: #8b5cf6">

<HomeHero
  badge="v0.5 milestone"
  badge-link="/v05/"
  title=": AI × Lang × OS"
  description="Auto is a dynamic-meets-static, cross-ecosystem language.<br>Script instantly on the AutoVM — ship the same source as Rust."
  primary-text="Get Started"
  primary-link="/docs/language/overview#running-and-building"
  secondary-text="Try Online"
  secondary-link="/playground"
/>

<p class="section-desc release-status">Milestone introductions and development material are available; release artifacts and download links await candidate freeze.</p>

<HomeDemo />

<div class="pillars-section">
  <h2 class="section-title">One Language, Every Layer</h2>
  <p class="section-desc">v0.5 connects the language, cross-renderer UI, virtual desktop and applications; readiness varies by path.</p>
  <div class="pillars-grid">
    <FeatureCard icon="🌐" title="Language" description="Actor concurrency, Rust-like generics, comptime metaprogramming, and memory safety." color="rgba(99, 102, 241, 0.15)" link="/docs/language" />
    <FeatureCard icon="🦀" title="Rust" description="AutoVM as a Rust scripting environment. a2r emits Rust; supported interfaces depend on concrete signatures and verified corpus." color="rgba(222, 165, 132, 0.15)" link="/rust" />
    <FeatureCard icon="🐍" title="Python" description="Call Python code directly from AutoVM. a2py transpiles Auto to Python." color="rgba(59, 130, 246, 0.15)" link="/python" />
    <FeatureCard icon="🎨" title="UI" description="Vue/Web and iced/desktop are the main paths; Harmony and Jetpack Compose are at feasibility-demo maturity." color="rgba(168, 85, 247, 0.15)" link="/ui" />
    <FeatureCard icon="🤖" title="AI" :description="copy.home.ai" color="rgba(236, 72, 153, 0.15)" link="/ai" />
    <FeatureCard icon="💻" title="OS" :description="copy.home.os" color="rgba(20, 184, 166, 0.15)" link="/os" />
  </div>
</div>

<div class="apps-section">
  <h2 class="section-title">Built with Auto</h2>
  <p class="section-desc">Applications for code, commands, development tasks and knowledge; each topic describes current progress.</p>
  <div class="apps-grid">
    <div v-for="app in applications" :key="app.key" class="app-card">
      <div class="app-shot"><AutoShellPreview v-if="app.key === 'autoshell'" lang="en" /><ScreenshotSlot v-else :capture-id="captureIds[app.key]" /></div>
      <h3>{{ app.name }}</h3>
      <p>{{ app.summary }}</p>
      <a :href="'/apps/' + app.key + '/'">Learn more →</a>
    </div>
  </div>
  <p class="section-desc">AutoDown supplies document and editor foundations; Jade Garden is an existing knowledge-base project, described separately from JadeEdit. <a href="/apps/autodown/">Related resources →</a></p>
</div>

<div class="apps-section">
  <h2 class="section-title">Script Now, Ship as Rust — Verified</h2>
  <p class="section-desc">The same Auto source runs on AutoVM instantly and transpiles to idiomatic Rust. Behavior agreement is enforced by a three-way parity harness (AutoVM = a2r-transpiled Rust = native Rust), not claimed.</p>
  <div class="cta-actions">
    <a href="/script-as-rust" class="cta-btn cta-primary">How Auto Scripts Ship as Rust</a>
    <a href="https://github.com/zhaopuming/auto-lang/blob/master/parity/docs/parity-dashboard.html" class="cta-btn cta-secondary">Parity Dashboard (the evidence)</a>
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">What's New in v0.5</h2>
  <p class="section-desc">Rust/Python interoperability, Vue/iced UI, AutoAI and the AutoOS application ecosystem; see the notes for support boundaries.</p>
  <div class="cta-actions">
    <a href="/docs/releases/v0.5" class="cta-btn cta-primary">Read Release Notes</a>
    <a href="/playground" class="cta-btn cta-secondary">Open Playground</a>
  </div>
</div>

<div class="icp-footer">
  <a href="https://beian.miit.gov.cn/" target="_blank">粤ICP备2026054131号-1</a>
</div>

</div>

<style scoped>
.pillars-section {
  padding: 4rem 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.pillars-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1.5rem;
}

.apps-section {
  padding: 4rem 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.apps-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 1.5rem;
}

.app-card {
  padding: 1.5rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  min-width: 0;
}

.app-shot {
  margin: -0.5rem -0.5rem 1rem;
  min-width: 0;
  overflow: hidden;
  border-radius: calc(var(--radius) - 2px);
}

.app-shot img {
  display: block;
  width: 100%;
  height: auto;
  border: 1px solid hsl(var(--border));
  border-radius: calc(var(--radius) - 2px);
}

.app-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.08);
}

.dark .app-card:hover {
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.3);
}

.app-card h3 {
  margin: 0 0 0.5rem;
  font-size: 1.25rem;
  color: hsl(var(--foreground));
}

.app-card p {
  margin: 0 0 1rem;
  color: hsl(var(--muted-foreground));
  font-size: 0.95rem;
  line-height: 1.6;
}

.app-card a {
  color: #6366f1;
  text-decoration: none;
  font-weight: 600;
  font-size: 0.95rem;
}

.app-card a:hover {
  text-decoration: underline;
}

.icp-footer {
  padding: 2rem;
  text-align: center;
  font-size: 0.875rem;
  color: hsl(var(--muted-foreground));
}

.icp-footer a {
  color: hsl(var(--muted-foreground));
  text-decoration: none;
}

.icp-footer a:hover {
  text-decoration: underline;
}
</style>
