---
layout: home
---

<script setup>
import { onMounted } from 'vue'
import HomeHero from './.vitepress/theme/components/HomeHero.vue'
import FeatureCard from './.vitepress/theme/components/FeatureCard.vue'
const icons = ['🌐', '🦀', '🐍', '🎨', '🤖', '💻']
onMounted(() => {
  if (sessionStorage.getItem('auto-lang-checked')) return;
  sessionStorage.setItem('auto-lang-checked', '1');
  if (navigator.language.toLowerCase().startsWith('zh')) {
    window.location.replace('/zh/');
  }
});
</script>

<HomeHero
  badge="v0.5 is now available"
  title=": Language + Runtime + AI + OS"
  description="Auto is a full-stack application platform. Write scripts, backends, UIs, AI agents, and OS components in one language — run them on AutoVM or transpile to Rust, Python, and TypeScript."
  primary-text="Get Started"
  primary-link="/docs/"
  secondary-text="Try Online"
  secondary-link="/playground"
/>

<div class="pillars-section">
  <h2 class="section-title">One Language, Every Layer</h2>
  <p class="section-desc">v0.5 turns Auto from a language into a complete platform for building modern applications.</p>
  <div class="pillars-grid">
    <FeatureCard icon="🌐" title="Language" description="Actor concurrency, Rust-like generics, comptime metaprogramming, and memory safety." color="rgba(99, 102, 241, 0.15)" link="/docs/language" />
    <FeatureCard icon="🦀" title="Rust" description="AutoVM as a Rust scripting environment. A2R transpiles Auto to production-grade Rust. Dual stdlib modes." color="rgba(222, 165, 132, 0.15)" link="/rust" />
    <FeatureCard icon="🐍" title="Python" description="Call Python code directly from AutoVM. a2py transpiles Auto to Python." color="rgba(59, 130, 246, 0.15)" link="/python" />
    <FeatureCard icon="🎨" title="UI" description="Vue and Tauri are mature. Desktop (Rust/iced) is usable. Harmony and Android demos validated." color="rgba(168, 85, 247, 0.15)" link="/ui" />
    <FeatureCard icon="🤖" title="AI" description="Client/Daemon architecture. AutoAI-Cli terminal agent. AutoMusk general-purpose coding agent." color="rgba(236, 72, 153, 0.15)" link="/ai" />
    <FeatureCard icon="💻" title="OS" description="Client/Daemon architecture, unified config system. Future: standalone AutoOS and embedded virtual desktop." color="rgba(20, 184, 166, 0.15)" link="/os" />
  </div>
</div>

<div class="apps-section">
  <h2 class="section-title">Built with Auto</h2>
  <p class="section-desc">Real applications that prove the platform works.</p>
  <div class="apps-grid">
    <div class="app-card">
      <h3>AutoShell</h3>
      <p>Cross-platform shell with CLI/TUI/GUI modes and Warp-like AI capabilities.</p>
      <a href="/apps#autoshell">Learn more →</a>
    </div>
    <div class="app-card">
      <h3>AutoMusk</h3>
      <p>General-purpose coding agent built on AutoPlan, implemented in Auto itself.</p>
      <a href="/apps#automusk">Learn more →</a>
    </div>
    <div class="app-card">
      <h3>AutoDown</h3>
      <p>An Auto dialect that combines Markdown and YAML for structured knowledge bases.</p>
      <a href="/apps#autodown">Learn more →</a>
    </div>
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">What's New in v0.5</h2>
  <p class="section-desc">The biggest milestone yet: Rust integration, Python support, dual stdlib modes, mature AutoUI, AutoAI architecture, and AutoOS foundations.</p>
  <div class="cta-actions">
    <a href="/docs/releases/v0.5" class="cta-btn cta-primary">Read Release Notes</a>
    <a href="/playground" class="cta-btn cta-secondary">Open Playground</a>
  </div>
</div>

<div class="icp-footer">
  <a href="https://beian.miit.gov.cn/" target="_blank">粤ICP备2026054131号-1</a>
</div>

<style scoped>
.pillars-section {
  padding: 4rem 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.section-title {
  font-size: 2rem;
  font-weight: 700;
  text-align: center;
  margin-bottom: 1rem;
  color: hsl(var(--foreground));
}

.section-desc {
  font-size: 1.1rem;
  color: hsl(var(--muted-foreground));
  max-width: 600px;
  margin: 0 auto 2.5rem;
  text-align: center;
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

.cta-section {
  padding: 4rem 2rem;
  text-align: center;
  background: linear-gradient(180deg, transparent 0%, rgba(99, 102, 241, 0.05) 100%);
}

.cta-actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
  flex-wrap: wrap;
}

.cta-btn {
  display: inline-flex;
  align-items: center;
  padding: 0.875rem 2rem;
  border-radius: var(--radius);
  font-weight: 600;
  text-decoration: none;
  transition: all 0.2s ease;
}

.cta-primary {
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  color: white;
  box-shadow: 0 4px 14px rgba(99, 102, 241, 0.3);
}

.cta-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(99, 102, 241, 0.4);
}

.cta-secondary {
  background: hsl(var(--card));
  color: hsl(var(--foreground));
  border: 1px solid hsl(var(--border));
}

.cta-secondary:hover {
  background: hsl(var(--accent));
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
