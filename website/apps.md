---
layout: home
---

<script setup>
import FeatureCard from './.vitepress/theme/components/FeatureCard.vue'
import StatCard from './.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from './.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #6366f1; --page-accent-2: #8b5cf6">

<div class="landing-hero">
  <div class="badge">Built with Auto</div>
  <h1 class="title">Real Apps, <span class="accent">Real Platform</span></h1>
  <p class="description">
    These applications prove Auto is not just a language — it is a platform
    for building shells, agents, knowledge bases, and full-stack systems.
  </p>
  <div class="actions">
    <a href="/docs/releases/v0.5" class="btn btn-primary">v0.5 Release Notes</a>
    <a href="/playground" class="btn btn-secondary">Try Playground</a>
  </div>
</div>

<div class="stats-section">
  <h2 class="section-title">Applications in v0.5</h2>
  <div class="stats-grid">
    <StatCard value="4" label="Flagship Apps" description="AutoShell, AutoMusk, AutoDown, and AutoUI demos." color="#6366f1" />
    <StatCard value="100%" label="Auto Written" description="All applications are implemented in Auto itself." color="#8b5cf6" />
    <StatCard value="3" label="Platforms" description="CLI, TUI, and GUI modes across the app suite." color="#14b8a6" />
    <StatCard value="∞" label="Extensible" description="Each app is a reference for building your own." color="#ec4899" />
  </div>
</div>

<div class="apps-list">
  <div class="app-section" id="autoshell">
    <div class="app-header">
      <div class="app-icon">🐚</div>
      <div>
        <h2>AutoShell</h2>
        <span class="app-status beta">Beta</span>
      </div>
    </div>
    <p class="app-desc">
      A complete cross-platform shell that can replace Bash, Fish, and Zsh.
      Three modes — CLI, TUI, and GUI — with Warp-like AI capabilities.
    </p>
    <div class="app-features">
      <div class="app-feature">
        <h4>Structured Pipelines</h4>
        <p>No more string parsing. Commands return typed objects through Atom pipelines.</p>
      </div>
      <div class="app-feature">
        <h4>AI Integrated</h4>
        <p>F3 AI mode powered by aaid. Natural language to shell commands.</p>
      </div>
      <div class="app-feature">
        <h4>Cross-Platform</h4>
        <p>Windows, Linux, macOS. One shell, consistent behavior everywhere.</p>
      </div>
      <div class="app-feature">
        <h4>Dual Implementation</h4>
        <p>Rust version is feature-complete. Auto version is fully usable.</p>
      </div>
    </div>
  </div>

  <div class="app-section" id="automusk">
    <div class="app-header">
      <div class="app-icon">🤖</div>
      <div>
        <h2>AutoMusk</h2>
        <span class="app-status beta">Beta</span>
      </div>
    </div>
    <p class="app-desc">
      A general-purpose coding agent built on AutoPlan. Written in Auto,
      running on AutoVM, configured through auto-os-config.
    </p>
    <div class="app-features">
      <div class="app-feature">
        <h4>AutoPlan Powered</h4>
        <p>Structured planning and execution for complex coding tasks.</p>
      </div>
      <div class="app-feature">
        <h4>Multi-Provider</h4>
        <p>Works with any model served by the aaid daemon.</p>
      </div>
      <div class="app-feature">
        <h4>Self-Hosting</h4>
        <p>Implemented in Auto itself. Proof that Auto can build real agents.</p>
      </div>
      <div class="app-feature">
        <h4>Config UI</h4>
        <p>Edit roles, skills, and modes through auto-os-config.</p>
      </div>
    </div>
  </div>

  <div class="app-section" id="autodown">
    <div class="app-header">
      <div class="app-icon">📄</div>
      <div>
        <h2>AutoDown</h2>
        <span class="app-status alpha">Alpha</span>
      </div>
    </div>
    <p class="app-desc">
      An Auto dialect that combines Markdown and YAML. Express arbitrary
      knowledge bases with structured, parseable documents.
    </p>
    <div class="app-features">
      <div class="app-feature">
        <h4>Markdown + YAML</h4>
        <p>Familiar syntax with structured data capabilities.</p>
      </div>
      <div class="app-feature">
        <h4>Parseable</h4>
        <p>Documents are data. Query, transform, and generate from them.</p>
      </div>
      <div class="app-feature">
        <h4>Knowledge Base</h4>
        <p>Build wikis, docs, and structured content systems.</p>
      </div>
      <div class="app-feature">
        <h4>Auto Dialect</h4>
        <p>Part of the Auto ecosystem. Compiles with the same toolchain.</p>
      </div>
    </div>
  </div>

  <div class="app-section" id="autoui-demos">
    <div class="app-header">
      <div class="app-icon">🎨</div>
      <div>
        <h2>AutoUI Demos</h2>
        <span class="app-status alpha">Alpha</span>
      </div>
    </div>
    <p class="app-desc">
      Dozens of demos and a complete Widgets Gallery, all implemented in Auto.
      Proving that AutoUI can build real interfaces.
    </p>
    <div class="app-features">
      <div class="app-feature">
        <h4>Widgets Gallery</h4>
        <p>46+ components, 24 blocks, charts, and layouts.</p>
      </div>
      <div class="app-feature">
        <h4>Full-Stack Examples</h4>
        <p>015-notes: Vue frontend + Rust backend, generated from Auto.</p>
      </div>
      <div class="app-feature">
        <h4>Desktop Demos</h4>
        <p>Rust/iced applications with hot reload and DevTools.</p>
      </div>
      <div class="app-feature">
        <h4>Mobile Prototypes</h4>
        <p>Android and HarmonyOS demo applications.</p>
      </div>
    </div>
  </div>
</div>

<div class="features-section">
  <h2 class="section-title">Why These Apps Matter</h2>
  <div class="features-grid">
    <FeatureCard icon="✅" title="Proof of Platform" description="Real applications prove Auto is production-ready, not just a toy language." color="rgba(99, 102, 241, 0.15)" />
    <FeatureCard icon="📚" title="Reference Implementations" description="Study these apps to learn how to structure your own Auto projects." color="rgba(139, 92, 246, 0.15)" />
    <FeatureCard icon="🔄" title="Dogfooding" description="Auto builds Auto. The compiler, VM, and apps are all in the same ecosystem." color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🚀" title="v0.6 Foundation" description="These apps will become system applications in the future AutoOS." color="rgba(236, 72, 153, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Start building with Auto</h2>
  <p class="section-desc">Use these applications as references, or jump into the documentation to build your own.</p>
  <div class="cta-actions">
    <a href="/docs/" class="cta-btn cta-primary">Read the Docs</a>
    <a href="/playground" class="cta-btn cta-secondary">Open Playground</a>
  </div>
</div>

</div>

<style scoped>
.apps-list {
  max-width: 1000px;
  margin: 0 auto;
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 3rem;
}

.app-section {
  padding: 2rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
}

.app-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
}

.app-icon {
  font-size: 2.5rem;
}

.app-header h2 {
  margin: 0;
  font-size: 1.75rem;
  color: hsl(var(--foreground));
}

.app-status {
  padding: 0.25rem 0.75rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  font-weight: 700;
}

.app-status.beta { background: rgba(59, 130, 246, 0.15); color: #3b82f6; }
.app-status.alpha { background: rgba(139, 92, 246, 0.15); color: #8b5cf6; }

.app-desc {
  font-size: 1.1rem;
  color: hsl(var(--muted-foreground));
  line-height: 1.7;
  margin: 0 0 1.5rem;
}

.app-features {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
}

.app-feature h4 {
  margin: 0 0 0.5rem;
  font-size: 0.95rem;
  color: hsl(var(--foreground));
}

.app-feature p {
  margin: 0;
  font-size: 0.875rem;
  color: hsl(var(--muted-foreground));
  line-height: 1.5;
}
</style>
