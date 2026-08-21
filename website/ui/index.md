---
layout: home
---

<script setup>
import FeatureCard from '../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #8b5cf6; --page-accent-2: #ec4899">

<div class="landing-hero">
  <div class="badge">AutoUI <span class="alpha">Alpha</span></div>
  <h1 class="title">One UI, <span class="accent">Every Platform</span></h1>
  <p class="description">
    Write your UI once in Auto. Generate Vue for the web, Rust/iced for desktop,
    and native code for mobile platforms — all from the same source.
  </p>
  <div class="actions">
    <a href="/ui/gallery/index.html" target="_self" class="btn btn-primary">Open Component Gallery</a>
    <a href="/docs/ui" class="btn btn-secondary">Read UI Docs</a>
  </div>
</div>

<div class="stats-section">
  <h2 class="section-title">AutoUI in v0.5</h2>
  <div class="stats-grid">
    <StatCard value="46+" label="Vue Components" description="shadcn-vue replicas, production-ready." color="#8b5cf6" />
    <StatCard value="24" label="Blocks" description="Pre-built page blocks: login, dashboard, sidebar, and more." color="#3b82f6" />
    <StatCard value="4" label="Platforms" description="Web (Vue/Tauri), Desktop (iced), Android, HarmonyOS." color="#14b8a6" />
    <StatCard value="2" label="DevTools" description="F12 inspector and MCP for AI-driven UI development." color="#ec4899" />
  </div>
</div>

<div class="platforms-section">
  <h2 class="section-title">Supported Platforms</h2>
  <div class="platforms-grid">
    <a href="/ui/gallery/index.html" target="_self" class="platform-card web">
      <div class="platform-icon">🌐</div>
      <h3>Web <span class="status beta">Beta</span></h3>
      <p>Vue 3 + Tauri. Replicates most Vue-based websites. Production-ready.</p>
      <span class="platform-link">Explore →</span>
    </a>
    <div class="platform-card desktop">
      <div class="platform-icon">🖥️</div>
      <h3>Desktop <span class="status alpha">Alpha</span></h3>
      <p>Rust/iced backend. VM and A2R both supported — equivalent to hot reload.</p>
      <span class="platform-note">DevTool included</span>
    </div>
    <div class="platform-card android">
      <div class="platform-icon">🤖</div>
      <h3>Android <span class="status demo">Demo</span></h3>
      <p>Jetpack Compose backend. Feasibility validated with running demos.</p>
      <span class="platform-note">v0.6 target</span>
    </div>
    <div class="platform-card harmony">
      <div class="platform-icon">🌏</div>
      <h3>Harmony <span class="status demo">Demo</span></h3>
      <p>ArkTS backend. Feasibility validated with running demos.</p>
      <span class="platform-note">v0.6 target</span>
    </div>
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="Web — Vue & Tauri"
    description="The most mature AutoUI backend. Generate Vue 3 applications from Auto view blocks, or wrap them in Tauri for desktop-like web apps."
    badge="Web"
  >
    <ul>
      <li><strong>46+ shadcn-vue components</strong> — buttons, tables, forms, dialogs, and more</li>
      <li><strong>Blocks</strong> — pre-built login, dashboard, sidebar, and settings pages</li>
      <li><strong>Charts</strong> — area, bar, line, and donut charts with Unovis</li>
      <li><strong>Tauri IPC</strong> — <code>#[tauri::command]</code> and Channel support for desktop integration</li>
    </ul>
    <template #visual>
      <div class="gallery-links">
        <a href="/ui/gallery/index.html" target="_self" class="gallery-link">
          <span class="gallery-icon">🧩</span>
          <span>Component Gallery</span>
        </a>
        <a href="/ui/blocks/index.html" target="_self" class="gallery-link">
          <span class="gallery-icon">🧱</span>
          <span>Blocks Gallery</span>
        </a>
        <a href="/ui/charts/index.html" target="_self" class="gallery-link">
          <span class="gallery-icon">📊</span>
          <span>Charts Gallery</span>
        </a>
        <a href="/ui/a2ui/index.html" target="_self" class="gallery-link">
          <span class="gallery-icon">⚡</span>
          <span>A2UI Demo</span>
        </a>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Desktop — Rust/iced"
    description="Cross-platform desktop applications with native performance. Both AutoVM and A2R execution modes are supported."
    badge="Desktop"
    reverse
  >
    <ul>
      <li><strong>VM mode</strong> — run desktop UIs directly in AutoVM with hot reload</li>
      <li><strong>A2R mode</strong> — transpile to Rust/iced for production binaries</li>
      <li><strong>Chrome-like DevTool</strong> — inspect component tree, properties, and console</li>
      <li><strong>MCP protocol</strong> — let AI agents query and manipulate AutoUI interfaces</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">desktop.at</span>
        </div>
        <pre class="code-body"><code><span class="keyword">view</span> MainWindow {
    <span class="keyword">column</span>(padding: 20) {
        <span class="keyword">text</span>(<span class="string">"Hello, Desktop!"</span>)
        <span class="keyword">button</span>(<span class="string">"Click me"</span>, on_click: handle)
    }
}</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Mobile — Android & Harmony"
    description="Native mobile UIs from the same Auto source. Currently in feasibility validation, targeting usable betas in v0.6."
    badge="Mobile"
  >
    <ul>
      <li><strong>Android</strong> — Jetpack Compose backend. Demo apps validated.</li>
      <li><strong>HarmonyOS</strong> — ArkTS backend. Demo apps validated.</li>
      <li><strong>Shared widgets</strong> — same AutoUI component definitions across all platforms</li>
      <li><strong>Widgets Gallery</strong> — dozens of Auto-implemented demos proving the concept</li>
    </ul>
    <template #visual>
      <div class="mobile-grid">
        <div class="mobile-item android">Android</div>
        <div class="mobile-item harmony">Harmony</div>
        <div class="mobile-item compose">Compose</div>
        <div class="mobile-item arkts">ArkTS</div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">AutoUI Advantages</h2>
  <div class="features-grid">
    <FeatureCard icon="📝" title="Single Source" description="One Auto view block compiles to Vue, iced, Compose, and ArkTS. No platform-specific rewrites." color="rgba(139, 92, 246, 0.15)" />
    <FeatureCard icon="🔥" title="Hot Reload" description="VM mode gives you instant feedback. A2R mode gives you production performance." color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🔍" title="Built-in DevTools" description="F12 inspector with component tree, property editor, and console. Chrome DevTools, but for AutoUI." color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🤖" title="MCP for AI" description="AI agents can query and manipulate your UI through the MCP protocol. Automated UI testing and development." color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🎨" title="Tailwind Bridge" description="Use Tailwind-style utility classes in AutoUI. Familiar styling, native output." color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="📦" title="Widget Ecosystem" description="Dozens of demos and a full Widgets Gallery, all implemented in Auto." color="rgba(99, 102, 241, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Build your first AutoUI app</h2>
  <p class="section-desc">Start with the Component Gallery, or dive into the documentation to build a full application.</p>
  <div class="cta-actions">
    <a href="/ui/gallery/index.html" target="_self" class="cta-btn cta-primary">Open Gallery</a>
    <a href="/docs/ui" class="cta-btn cta-secondary">Read UI Docs</a>
  </div>
</div>

</div>

<style scoped>
.platforms-section {
  padding: 4rem 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.platforms-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 1.5rem;
}

.platform-card {
  padding: 1.5rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  text-decoration: none;
  color: inherit;
}

.platform-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.08);
}

.dark .platform-card:hover {
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.3);
}

.platform-card.web {
  border-color: rgba(139, 92, 246, 0.3);
  background: rgba(139, 92, 246, 0.03);
}

.platform-icon {
  font-size: 2rem;
  margin-bottom: 1rem;
}

.platform-card h3 {
  margin: 0 0 0.75rem;
  font-size: 1.25rem;
  color: hsl(var(--foreground));
}

.platform-card p {
  margin: 0 0 1rem;
  color: hsl(var(--muted-foreground));
  font-size: 0.95rem;
  line-height: 1.6;
}

.status {
  padding: 0.125rem 0.5rem;
  border-radius: 9999px;
  font-size: 0.7rem;
  font-weight: 700;
}

.status.beta { background: rgba(59, 130, 246, 0.15); color: #3b82f6; }
.status.alpha { background: rgba(139, 92, 246, 0.15); color: #8b5cf6; }
.status.demo { background: rgba(245, 158, 11, 0.15); color: #f59e0b; }

.platform-link {
  color: var(--page-accent-1);
  font-weight: 600;
  font-size: 0.95rem;
}

.platform-note {
  color: hsl(var(--muted-foreground));
  font-size: 0.875rem;
}

.gallery-links {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  width: 100%;
  max-width: 420px;
}

.gallery-link {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  text-decoration: none;
  color: hsl(var(--foreground));
  font-weight: 600;
  font-size: 0.9rem;
  transition: all 0.2s ease;
}

.gallery-link:hover {
  border-color: var(--page-accent-1);
  background: color-mix(in srgb, var(--page-accent-1) 5%, transparent);
}

.gallery-icon {
  font-size: 1.25rem;
}

.mobile-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  width: 100%;
  max-width: 320px;
}

.mobile-item {
  padding: 1rem;
  border-radius: var(--radius);
  text-align: center;
  font-weight: 700;
  font-size: 0.85rem;
  color: white;
}

.mobile-item.android { background: linear-gradient(135deg, #3ddc84, #2da766); }
.mobile-item.harmony { background: linear-gradient(135deg, #007dff, #005bb5); }
.mobile-item.compose { background: linear-gradient(135deg, #4285f4, #2b5cb8); }
.mobile-item.arkts { background: linear-gradient(135deg, #f4a460, #d97706); }

@media (max-width: 768px) {
  .gallery-links {
    grid-template-columns: 1fr;
  }
}
</style>
