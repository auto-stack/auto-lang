---
layout: home
---

<script setup>
import FeatureCard from './.vitepress/theme/components/FeatureCard.vue'
import StatCard from './.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from './.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="rust-hero">
  <div class="badge">Rust Integration <span class="beta">Beta</span></div>
  <h1 class="title">Auto <span class="accent">×</span> Rust</h1>
  <p class="description">
    Use Auto as a scripting environment for Rust. Call any Rust code from AutoVM,
    or transpile Auto to production-grade Rust with A2R.
  </p>
  <div class="actions">
    <a href="/docs/design/auto-as-rust-script-strategy" class="btn btn-primary">Read the Strategy</a>
    <a href="/playground" class="btn btn-secondary">Try in Playground</a>
  </div>
</div>

<div class="stats-section">
  <h2 class="section-title">Deep Rust Integration</h2>
  <div class="stats-grid">
    <StatCard value="Any" label="Rust Code" description="AutoVM can call almost any Rust code directly." color="#dea584" />
    <StatCard value="A2R" label="Transpiler" description="Auto → Rust with behavior parity to AutoVM." color="#f59e0b" />
    <StatCard value="2" label="Stdlib Modes" description="Auto-native stdlib or Rust-native stdlib." color="#3b82f6" />
    <StatCard value="100%" label="Backend Freedom" description="Call any Rust crate from your Auto code." color="#10b981" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="AutoVM as a Rust Scripting Environment"
    description="AutoVM is no longer just a language VM — it is a full Rust scripting environment."
    badge="Scripting"
  >
    <ul>
      <li><strong>Call any Rust code</strong> from Auto scripts</li>
      <li><strong>Hot reload</strong> — iterate on Rust logic without recompiling the host</li>
      <li><strong>Zero FFI boilerplate</strong> — AutoVM handles marshalling automatically</li>
      <li><strong>Embed Rust libraries</strong> as first-class Auto modules</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">script.at</span>
        </div>
        <pre class="code-body"><code><span class="keyword">use</span> rust::std::collections::HashMap;

<span class="keyword">fn</span> <span class="function">main</span>() {
    <span class="keyword">let</span> map = HashMap::new();
    map.insert(<span class="string">"hello"</span>, <span class="string">"rust"</span>);
    <span class="function">println</span>(map.get(<span class="string">"hello"</span>));
}</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="A2R Transpiler"
    description="Compile Auto to Rust with behavior parity to AutoVM. Ship production binaries from your Auto source."
    badge="Transpilation"
    reverse
  >
    <ul>
      <li><strong>Behavior parity</strong> — A2R output matches AutoVM semantics</li>
      <li><strong>Axum HTTP generation</strong> — <code>#[api]</code> → full Axum servers</li>
      <li><strong>Tauri IPC</strong> — <code>#[tauri::command]</code> and Channel support</li>
      <li><strong>Escape analysis</strong> — optimal borrow/clone/Rc selection</li>
      <li><strong>Merge mode</strong> — multi-file project transpilation</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">a2r output</span>
        </div>
        <pre class="code-body"><code><span class="comment">// Auto source</span>
<span class="keyword">fn</span> <span class="function">add</span>(a: int, b: int) -> int { a + b }

<span class="comment">// Generated Rust</span>
<span class="keyword">fn</span> <span class="function">add</span>(a: i64, b: i64) -> i64 { a + b }</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Dual Standard Library Modes"
    description="Choose the stdlib that fits your deployment target."
    badge="Stdlib"
  >
    <ul>
      <li><strong>Auto-native stdlib</strong> — io, net, http, fs, and more. Targets Rust today, Python and others tomorrow.</li>
      <li><strong>Rust-embedded mode</strong> — use Rust stdlib and any crates.io library directly. No re-implementation needed.</li>
    </ul>
    <template #visual>
      <div class="mode-grid">
        <div class="mode-card auto">
          <h4>Auto Stdlib</h4>
          <p>Portable across targets</p>
          <code>use std::http</code>
        </div>
        <div class="mode-card rust">
          <h4>Rust Stdlib</h4>
          <p>Full crates.io access</p>
          <code>use rust::std::net</code>
        </div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">Why Auto + Rust?</h2>
  <div class="features-grid">
    <FeatureCard icon="🚀" title="Scripting Speed" description="Write Rust-powered scripts in Auto without setting up a Cargo project for every task." color="rgba(222, 165, 132, 0.15)" />
    <FeatureCard icon="📦" title="Crate Ecosystem" description="Tap into crates.io without leaving Auto. Axum, Tokio, Serde — all accessible." color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🔄" title="Iterative Workflow" description="Prototype in AutoVM, ship with A2R. Same code, two execution modes." color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🛡️" title="Memory Safety" description="Auto's ownership system maps naturally to Rust's borrow checker." color="rgba(16, 185, 129, 0.15)" />
    <FeatureCard icon="⚡" title="Zero-Cost Abstraction" description="A2R generates idiomatic Rust with no runtime overhead." color="rgba(139, 92, 246, 0.15)" />
    <FeatureCard icon="🔧" title="Escape Analysis" description="Compiler picks borrow, clone, or Rc based on actual usage patterns." color="rgba(236, 72, 153, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Start using Auto with Rust</h2>
  <p class="section-desc">Read the design documents, run the cookbook examples, or try the Rust scripting mode in the Playground.</p>
  <div class="cta-actions">
    <a href="/docs/design/auto-as-rust-script-strategy" class="cta-btn cta-primary">Read Design Docs</a>
    <a href="/docs/releases/v0.5" class="cta-btn cta-secondary">v0.5 Release Notes</a>
  </div>
</div>

<style scoped>
.rust-hero {
  padding: 6rem 2rem 4rem;
  text-align: center;
  max-width: 900px;
  margin: 0 auto;
}

.badge {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border-radius: 9999px;
  background: rgba(222, 165, 132, 0.1);
  border: 1px solid rgba(222, 165, 132, 0.3);
  color: #dea584;
  font-size: 0.875rem;
  font-weight: 500;
  margin-bottom: 1.5rem;
}

.badge .beta {
  padding: 0.125rem 0.5rem;
  border-radius: 9999px;
  background: rgba(222, 165, 132, 0.2);
  font-size: 0.75rem;
  font-weight: 700;
}

.title {
  font-size: clamp(2.5rem, 5vw, 4rem);
  font-weight: 800;
  line-height: 1.1;
  letter-spacing: -0.02em;
  color: hsl(var(--foreground));
  margin: 0 0 1.5rem;
}

.accent {
  background: linear-gradient(120deg, #dea584 30%, #f59e0b 70%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.description {
  font-size: 1.25rem;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
  max-width: 700px;
  margin: 0 auto 2.5rem;
}

.actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
  flex-wrap: wrap;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.5rem;
  border-radius: var(--radius);
  font-weight: 600;
  font-size: 0.95rem;
  transition: all 0.2s ease;
  text-decoration: none;
}

.btn-primary {
  background: linear-gradient(135deg, #dea584 0%, #f59e0b 100%);
  color: white;
  box-shadow: 0 4px 14px rgba(222, 165, 132, 0.3);
}

.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(222, 165, 132, 0.4);
}

.btn-secondary {
  background: hsl(var(--card));
  color: hsl(var(--foreground));
  border: 1px solid hsl(var(--border));
}

.btn-secondary:hover {
  background: hsl(var(--accent));
}

.stats-section {
  padding: 4rem 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.section-title {
  font-size: 2rem;
  font-weight: 700;
  text-align: center;
  margin-bottom: 2.5rem;
  color: hsl(var(--foreground));
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 1.5rem;
}

.showcase-wrapper {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
}

.features-section {
  padding: 4rem 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1.5rem;
}

.cta-section {
  padding: 4rem 2rem;
  text-align: center;
  background: linear-gradient(180deg, transparent 0%, rgba(222, 165, 132, 0.05) 100%);
}

.section-desc {
  font-size: 1.1rem;
  color: hsl(var(--muted-foreground));
  max-width: 500px;
  margin: 0 auto 2rem;
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
  background: linear-gradient(135deg, #dea584 0%, #f59e0b 100%);
  color: white;
  box-shadow: 0 4px 14px rgba(222, 165, 132, 0.3);
}

.cta-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(222, 165, 132, 0.4);
}

.cta-secondary {
  background: hsl(var(--card));
  color: hsl(var(--foreground));
  border: 1px solid hsl(var(--border));
}

.cta-secondary:hover {
  background: hsl(var(--accent));
}

.code-window {
  background: #1e1e2e;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  width: 100%;
  max-width: 420px;
  text-align: left;
}

.code-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  background: #181825;
  border-bottom: 1px solid #313244;
}

.code-dots {
  display: flex;
  gap: 0.4rem;
}

.code-dots span {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.code-dots span:nth-child(1) { background: #ff5f56; }
.code-dots span:nth-child(2) { background: #ffbd2e; }
.code-dots span:nth-child(3) { background: #27c93f; }

.code-title {
  font-size: 0.8rem;
  color: #6c7086;
  font-family: 'JetBrains Mono', monospace;
}

.code-body {
  padding: 1.25rem;
  margin: 0;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 0.85rem;
  line-height: 1.6;
  color: #cdd6f4;
  overflow-x: auto;
}

.keyword { color: #cba6f7; }
.function { color: #89b4fa; }
.string { color: #a6e3a1; }
.comment { color: #6c7086; font-style: italic; }

.mode-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  width: 100%;
  max-width: 420px;
}

.mode-card {
  padding: 1.5rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  text-align: center;
}

.mode-card h4 {
  margin: 0 0 0.5rem;
  color: hsl(var(--foreground));
}

.mode-card p {
  margin: 0 0 1rem;
  font-size: 0.875rem;
  color: hsl(var(--muted-foreground));
}

.mode-card code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  color: #dea584;
  background: rgba(222, 165, 132, 0.1);
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
}

@media (max-width: 768px) {
  .mode-grid {
    grid-template-columns: 1fr;
  }
}
</style>
