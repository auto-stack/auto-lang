---
layout: home
---

<script setup>
import FeatureCard from './.vitepress/theme/components/FeatureCard.vue'
import StatCard from './.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from './.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="python-hero">
  <div class="badge">Python Integration <span class="alpha">Alpha</span></div>
  <h1 class="title">Auto <span class="accent">×</span> Python</h1>
  <p class="description">
    AutoVM can now call Python code directly. a2py transpiles Auto to Python.
    Bridge the gap between systems programming and the Python ecosystem.
  </p>
  <div class="actions">
    <a href="/docs/design/python-parity-roadmap" class="btn btn-primary">Read the Roadmap</a>
    <a href="/playground" class="btn btn-secondary">Try in Playground</a>
  </div>
</div>

<div class="stats-section">
  <h2 class="section-title">Python Ecosystem Access</h2>
  <div class="stats-grid">
    <StatCard value="Direct" label="VM Calls" description="Call Python code directly from AutoVM." color="#3b82f6" />
    <StatCard value="a2py" label="Transpiler" description="Auto → Python with idiomatic output." color="#10b981" />
    <StatCard value="PyO3" label="FFI" description="Rust-Python interop powered by PyO3." color="#f59e0b" />
    <StatCard value="AI/ML" label="Target" description="PyTorch, NumPy, and the Python AI stack." color="#8b5cf6" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="Call Python from AutoVM"
    description="AutoVM now has first-class Python interoperability. Import Python modules and call them as if they were native Auto code."
    badge="Interop"
  >
    <ul>
      <li><strong>Direct Python calls</strong> — <code>use.py</code> imports Python modules</li>
      <li><strong>Auto-type marshalling</strong> — automatic conversion between Auto and Python types</li>
      <li><strong>REPL Python FFI</strong> — experiment interactively</li>
      <li><strong>Multi-type support</strong> — scalars, collections, and custom objects</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">ai_script.at</span>
        </div>
        <pre class="code-body"><code><span class="keyword">use</span>.py numpy <span class="keyword">as</span> np;

<span class="keyword">fn</span> <span class="function">main</span>() {
    <span class="keyword">let</span> arr = np.array([1, 2, 3]);
    <span class="function">println</span>(arr.mean());
}</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="a2py Transpiler"
    description="Compile Auto to Python. Generate idiomatic Python code from your Auto source."
    badge="Transpilation"
    reverse
  >
    <ul>
      <li><strong>Idiomatic output</strong> — Pythonic method mapping and static methods</li>
      <li><strong><code>use</code> → <code>import</code></strong> — module system translation</li>
      <li><strong><code>@dataclass</code></strong> — Auto structs become Python dataclasses</li>
      <li><strong>Struct destructuring</strong> — pattern matching support</li>
      <li><strong>Two-phase emit</strong> — clean, readable Python output</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">a2py output</span>
        </div>
        <pre class="code-body"><code><span class="comment"># Auto source</span>
<span class="keyword">struct</span> Point { x: float, y: float }

<span class="comment"># Generated Python</span>
<span class="keyword">@dataclass</span>
<span class="keyword">class</span> <span class="function">Point</span>:
    x: float
    y: float</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Road to Full Parity"
    description="Python support is Alpha today. The roadmap targets full ecosystem access."
    badge="Roadmap"
  >
    <ul>
      <li><strong>v0.5 (now)</strong> — AutoVM calls most Python code; a2py transpiles basic programs</li>
      <li><strong>v0.6</strong> — Call any Python code; PyTorch and common AI environments</li>
      <li><strong>Future</strong> — Skip Python glue, call underlying C/C++ directly for performance</li>
    </ul>
    <template #visual>
      <div class="roadmap-grid">
        <div class="roadmap-item current">
          <span class="version">v0.5</span>
          <span class="status">Alpha</span>
        </div>
        <div class="roadmap-item">
          <span class="version">v0.6</span>
          <span class="status">Beta</span>
        </div>
        <div class="roadmap-item">
          <span class="version">Future</span>
          <span class="status">Stable</span>
        </div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">Why Auto + Python?</h2>
  <div class="features-grid">
    <FeatureCard icon="🧠" title="AI/ML Ecosystem" description="Access PyTorch, NumPy, Pandas, and the entire Python AI stack from Auto." color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🔄" title="Dual Workflow" description="Script in AutoVM with Python interop, then transpile to Python for deployment." color="rgba(16, 185, 129, 0.15)" />
    <FeatureCard icon="⚡" title="Performance Path" description="Future: bypass Python glue and call C/C++ extensions directly." color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="📊" title="Data Science" description="Combine Auto's type safety with Python's data ecosystem." color="rgba(139, 92, 246, 0.15)" />
    <FeatureCard icon="🔧" title="PyO3 Powered" description="Rust-grade FFI performance for Python interop." color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🌉" title="Bridge" description="Move between systems programming and scripting without switching languages." color="rgba(20, 184, 166, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Explore Auto + Python</h2>
  <p class="section-desc">Read the parity roadmap, run the conformance tests, or try Python interop in the Playground.</p>
  <div class="cta-actions">
    <a href="/docs/design/python-parity-roadmap" class="cta-btn cta-primary">Read Roadmap</a>
    <a href="/docs/releases/v0.5" class="cta-btn cta-secondary">v0.5 Release Notes</a>
  </div>
</div>

<style scoped>
.python-hero {
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
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: #3b82f6;
  font-size: 0.875rem;
  font-weight: 500;
  margin-bottom: 1.5rem;
}

.badge .alpha {
  padding: 0.125rem 0.5rem;
  border-radius: 9999px;
  background: rgba(59, 130, 246, 0.2);
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
  background: linear-gradient(120deg, #3b82f6 30%, #10b981 70%);
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
  background: linear-gradient(135deg, #3b82f6 0%, #10b981 100%);
  color: white;
  box-shadow: 0 4px 14px rgba(59, 130, 246, 0.3);
}

.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
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
  background: linear-gradient(180deg, transparent 0%, rgba(59, 130, 246, 0.05) 100%);
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
  background: linear-gradient(135deg, #3b82f6 0%, #10b981 100%);
  color: white;
  box-shadow: 0 4px 14px rgba(59, 130, 246, 0.3);
}

.cta-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
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

.roadmap-grid {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 1rem;
  width: 100%;
  max-width: 420px;
}

.roadmap-item {
  padding: 1.5rem 1rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.roadmap-item.current {
  border-color: #3b82f6;
  background: rgba(59, 130, 246, 0.05);
}

.roadmap-item .version {
  font-weight: 700;
  color: hsl(var(--foreground));
}

.roadmap-item .status {
  font-size: 0.8rem;
  color: #3b82f6;
  font-weight: 600;
}

@media (max-width: 768px) {
  .roadmap-grid {
    grid-template-columns: 1fr;
  }
}
</style>
