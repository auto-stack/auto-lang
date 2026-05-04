---
layout: home
---

<script setup>
import AIHero from './.vitepress/theme/components/AIHero.vue'
import FeatureCard from './.vitepress/theme/components/FeatureCard.vue'
import StatCard from './.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from './.vitepress/theme/components/ShowcaseSection.vue'
</script>

<AIHero
  badge="Made BY AI · Made FOR AI · Made of AI"
  title="AI-Native by Design"
  description="A programming language built with AI, optimized for AI, and composed of AI. From token-efficient syntax to agent-native architecture."
  primary-text="Read the Docs"
  primary-link="/docs/ai"
  secondary-text="Try Playground"
  secondary-link="/playground"
/>

<div class="stats-section">
  <h2 class="section-title">Developed at AI Speed</h2>
  <div class="stats-grid">
    <StatCard value="10B" label="Tokens Consumed" description="Zhipu AI MAX plan powered the entire development cycle." color="#6366f1" />
    <StatCard value="1300+" label="Commits" description="Rapid iteration over just two months of development." color="#8b5cf6" />
    <StatCard value="200K" label="Lines of Rust" description="Production-grade compiler, VM, and transpiler infrastructure." color="#ec4899" />
    <StatCard value="1+6" label="Compilers" description="One interpreter plus six transpilers (a2c, a2rs, a2ts, a2py, a2kt, a2vue)." color="#14b8a6" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="Made BY AI"
    description="Auto was born from a radical experiment: can a modern systems language be built almost entirely by AI?"
    badge="Development"
  >
    <ul>
      <li>Entire compiler frontend and backend generated via LLM collaboration</li>
      <li>Two-month sprint from concept to working transpilers</li>
      <li>Human oversight on architecture, AI execution on implementation</li>
      <li>Proves AI can tackle complex systems programming tasks</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span /><span /><span /></div>
          <span class="code-title">generated.at</span>
        </div>
        <pre class="code-body"><code><span class="keyword">fn</span> <span class="function">main</span>() {
    <span class="comment">// AI-generated core parser</span>
    <span class="keyword">let</span> ast = parser::parse(src);
    <span class="function">println</span>(ast.dump());
}</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Made FOR AI"
    description="Every syntax decision reduces parsing entropy and token consumption, making Auto the most AI-friendly language to generate."
    badge="Syntax Design"
    reverse
  >
    <ul>
      <li><strong>Postfix typing</strong>: <code>radius float</code> — entity first, constraint second</li>
      <li><strong>Semantic determinism</strong>: explicit over implicit, zero ambiguity</li>
      <li><strong>Dual-mode errors</strong>: human-friendly (miette) vs structured JSON for AI</li>
      <li><strong>Intention annotations</strong>: contract programming via annotations</li>
      <li><strong>80-90% token savings</strong> compared to traditional languages for equivalent logic</li>
    </ul>
    <template #visual>
      <div class="comparison-table">
        <div class="comp-row">
          <span class="comp-lang">C</span>
          <code class="comp-code">float radius;</code>
        </div>
        <div class="comp-row">
          <span class="comp-lang">Auto</span>
          <code class="comp-code highlight">radius float</code>
        </div>
        <div class="comp-note">AI sees the identifier first, then the type constraint</div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Made of AI"
    description="Auto is not just a tool for AI — it becomes the substrate that AI agents run on."
    badge="Agent Architecture"
  >
    <ul>
      <li><strong>SafeClaw</strong>: transpiled agent cluster (AutoAgent + AutoTool + AutoSkill + AutoKnowledge)</li>
      <li><strong>AI compilation acceleration</strong>: direct transpilation to CUDA, CANN, ROCm, MUSA</li>
      <li><strong>Atom protocol</strong>: LLM outputs structured data that Auto executes safely</li>
      <li><strong>Generate Once, Use Everywhere</strong>: one codebase targets all GPU platforms</li>
    </ul>
    <template #visual>
      <div class="gpu-grid">
        <div class="gpu-item nvidia">CUDA</div>
        <div class="gpu-item huawei">CANN</div>
        <div class="gpu-item amd">ROCm</div>
        <div class="gpu-item moore">MUSA</div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">Core AI Advantages</h2>
  <div class="features-grid">
    <FeatureCard icon="🧠" title="Low Parsing Entropy" description="Simplified AST structure drastically reduces hallucination and context window pressure for LLMs." color="rgba(99, 102, 241, 0.15)" />
    <FeatureCard icon="🔍" title="Physical Transparency" description="Memory layout is visible and predictable. AI can precisely calculate data footprint and cache behavior." color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="⚡" title="Code for AI, Summary for Human" description="Auto separates machine-optimized representations from human-readable summaries." color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🛡️" title="SafeClaw Agents" description="A transpiled agent framework where Auto code becomes the runtime for autonomous AI systems." color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🎯" title="Intention Annotation" description="Annotate intent, not implementation. The compiler and AI collaborate to fill in the details." color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🔄" title="Reverse Translation" description="Generate massive training corpora by reverse-transpiling existing codebases into Auto." color="rgba(59, 130, 246, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Ready to explore AI-native programming?</h2>
  <p class="section-desc">Dive into the detailed documentation or try Auto in the interactive playground.</p>
  <div class="cta-actions">
    <a href="/docs/ai" class="cta-btn cta-primary">Read AI Docs</a>
    <a href="/playground" class="cta-btn cta-secondary">Open Playground</a>
  </div>
</div>

<style scoped>
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
  background: linear-gradient(180deg, transparent 0%, rgba(99, 102, 241, 0.05) 100%);
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

.comparison-table {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: var(--radius);
  padding: 1.5rem;
  width: 100%;
  max-width: 420px;
}

.comp-row {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 0;
  border-bottom: 1px solid hsl(var(--border));
}

.comp-row:last-child {
  border-bottom: none;
}

.comp-lang {
  font-weight: 600;
  width: 60px;
  color: hsl(var(--muted-foreground));
}

.comp-code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.9rem;
  color: hsl(var(--foreground));
}

.comp-code.highlight {
  color: #a855f7;
  font-weight: 600;
}

.comp-note {
  margin-top: 1rem;
  font-size: 0.8rem;
  color: hsl(var(--muted-foreground));
  text-align: center;
}

.gpu-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  width: 100%;
  max-width: 320px;
}

.gpu-item {
  padding: 1rem;
  border-radius: var(--radius);
  text-align: center;
  font-weight: 700;
  font-size: 0.9rem;
  color: white;
}

.gpu-item.nvidia { background: linear-gradient(135deg, #76b900, #558b00); }
.gpu-item.huawei { background: linear-gradient(135deg, #cf0a2c, #9a0721); }
.gpu-item.amd { background: linear-gradient(135deg, #ed1c24, #b71518); }
.gpu-item.moore { background: linear-gradient(135deg, #3b82f6, #1d4ed8); }

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  }
}
</style>
