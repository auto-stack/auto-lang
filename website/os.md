---
layout: home
---

<script setup>
import OSHero from './.vitepress/theme/components/OSHero.vue'
import FeatureCard from './.vitepress/theme/components/FeatureCard.vue'
import StatCard from './.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from './.vitepress/theme/components/ShowcaseSection.vue'
</script>

<OSHero
  badge="Language as OS"
  title=": The OS in Your Code"
  description="Auto blurs the line between language and operating system. Compile-time execution, multi-target transpilers, and a unified runtime from MCU to cloud."
  primary-text="Read the Docs"
  primary-link="/docs/os"
  secondary-text="Explore LaOS"
  secondary-link="/docs/os#laos"
/>

<div class="stats-section">
  <h2 class="section-title">Breaking the Impossible Triangle</h2>
  <div class="stats-grid">
    <StatCard value="9/10" label="Development Efficiency" description="Script mode, simple generics, and intuitive syntax." color="#f59e0b" />
    <StatCard value="9/10" label="Memory Safety" description="Linear types, borrow checker, and smart casts." color="#3b82f6" />
    <StatCard value="9/10" label="Runtime Performance" description="AOT compilation, zero-cost abstractions, no GC pauses." color="#ec4899" />
    <StatCard value="6" label="Target Platforms" description="Windows, Linux, macOS, Android, HarmonyOS, MCU." color="#14b8a6" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="Language as OS (LaOS)"
    description="Traditional stacks separate language, runtime, and OS into disjoint layers. Auto fuses them into a single coherent system."
    badge="Architecture"
  >
    <ul>
      <li><strong>Microkernel language</strong>: core features (AutoVM, transpilers, CTE) act as the kernel</li>
      <li><strong>Memory subsystem</strong>: ownership + borrow checker + AutoFree + RC + escape analysis</li>
      <li><strong>I/O subsystem</strong>: unified <code>io.at</code> interface with vm/rs/c backends</li>
      <li><strong>Shell subsystem</strong>: ASH replaces Bash with structured data pipelines</li>
      <li><strong>UI subsystem</strong>: Aura renders to Web, Desktop, Mobile, and MCU (LVGL)</li>
    </ul>
    <template #visual>
      <div class="arch-diagram">
        <div class="arch-layer app">Applications</div>
        <div class="arch-arrow">↓</div>
        <div class="arch-layer lang">Auto Language</div>
        <div class="arch-arrow">↓</div>
        <div class="arch-layer runtime">Auto Runtime</div>
        <div class="arch-arrow">↓</div>
        <div class="arch-layer hw">Hardware</div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="AutoVM & Transpilers"
    description="One language, many targets. AutoVM runs scripts and embedded code, while transpilers generate native code for every platform."
    badge="Execution"
    reverse
  >
    <ul>
      <li><strong>AutoVM</strong>: standalone, comptime, and embedded modes</li>
      <li><strong>a2rs</strong>: Rust backend for systems programming</li>
      <li><strong>a2c</strong>: C backend for embedded and automotive (90% C support)</li>
      <li><strong>a2ts</strong>: TypeScript backend for web frontends</li>
      <li><strong>a2py</strong>: Python backend for AI and scripting</li>
      <li><strong>a2kt</strong>: Kotlin backend for Android / Jetpack Compose</li>
      <li><strong>a2vue / a2jet / a2ark</strong>: UI targets for Web, Desktop, Harmony</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span /><span /><span /></div>
          <span class="code-title">hello.at</span>
        </div>
        <pre class="code-body"><code><span class="comment">// Runs on all targets</span>
<span class="keyword">fn</span> <span class="function">main</span>() {
    <span class="keyword">let</span> msg = <span class="string">"Hello, OS!"</span>;
    <span class="function">println</span>(msg);
}</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Memory & Concurrency"
    description="Safety without sacrifice. Auto combines Rust-grade memory safety with multiple automatic reclamation strategies and Actor-model concurrency."
    badge="Systems"
  >
    <ul>
      <li><strong>Linear Type + Borrow Checker</strong>: compile-time ownership tracking</li>
      <li><strong>AutoFree</strong>: deterministic automatic memory cleanup</li>
      <li><strong>RC + Escape Analysis</strong>: automatic reference counting where safe</li>
      <li><strong>Task / Msg</strong>: Actor-model concurrency with no shared state</li>
      <li><strong>~T (async)</strong>: first-class asynchronous types</li>
      <li><strong>Atom protocol</strong>: zero-copy serialization across task boundaries</li>
    </ul>
    <template #visual>
      <div class="memory-grid">
        <div class="mem-item safe">Linear</div>
        <div class="mem-item safe">Borrow</div>
        <div class="mem-item auto">AutoFree</div>
        <div class="mem-item auto">RC</div>
        <div class="mem-item auto">Escape</div>
        <div class="mem-item task">Actor</div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Cross-Platform Adapters"
    description="From automotive MCUs to cloud servers, Auto adapts to the platform rather than forcing the platform to adapt to Auto."
    badge="Portability"
    reverse
  >
    <ul>
      <li><strong>Windows / Linux / macOS</strong>: Rust-native backend</li>
      <li><strong>SOC / Robotics</strong>: Rust / C++ (ROS) backend</li>
      <li><strong>MCU / Embedded</strong>: C backend on FreeRTOS</li>
      <li><strong>Mobile / Web</strong>: a2vue, a2jet, a2ark transpilers</li>
      <li><strong>Inter-device COM</strong>: RPC via Auto Virtual Bus</li>
      <li><strong>Vehicle development</strong>: full-stack from HPC to sensor MCU</li>
    </ul>
    <template #visual>
      <div class="platform-grid">
        <div class="plat-item desktop">Desktop</div>
        <div class="plat-item mobile">Mobile</div>
        <div class="plat-item soc">SOC / ROS</div>
        <div class="plat-item mcu">MCU</div>
        <div class="plat-item web">Web</div>
        <div class="plat-item vehicle">Vehicle</div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">LaOS Subsystems</h2>
  <div class="features-grid">
    <FeatureCard icon="⚙️" title="Compile-Time Execution" description="Execute Auto code at compile time. Generate code, validate invariants, and configure the system before it runs." color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="🔌" title="Auto Virtual Bus" description="Cross-process and cross-device RPC using the Atom protocol. Remote objects feel local." color="rgba(99, 102, 241, 0.15)" />
    <FeatureCard icon="🖥️" title="AutoShell (ASH)" description="Structured shell with Atom pipelines. No more string parsing — commands return typed objects." color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🎨" title="Aura UI" description="Declarative UI framework that transpiles to Vue, Jetpack Compose, and ArkTS. One codebase, all screens." color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="📦" title="AutoMan" description="Universal project manager and package system. Build, test, and deploy across all targets." color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🛡️" title="SafeClaw Security" description="NanoClaw transpiled to safe runtime. IronClaw features plus Aura UI for secure agent interfaces." color="rgba(59, 130, 246, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Ready to build the next-generation OS?</h2>
  <p class="section-desc">Learn how Auto unifies language, runtime, and operating system into a single toolchain.</p>
  <div class="cta-actions">
    <a href="/docs/os" class="cta-btn cta-primary">Read OS Docs</a>
    <a href="/docs/" class="cta-btn cta-secondary">Explore Features</a>
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

.arch-diagram {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  max-width: 320px;
}

.arch-layer {
  width: 100%;
  padding: 1rem;
  border-radius: var(--radius);
  text-align: center;
  font-weight: 700;
  color: white;
}

.arch-layer.app { background: linear-gradient(135deg, #f59e0b, #d97706); }
.arch-layer.lang { background: linear-gradient(135deg, #6366f1, #8b5cf6); }
.arch-layer.runtime { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.arch-layer.hw { background: linear-gradient(135deg, #14b8a6, #0d9488); }

.arch-arrow {
  color: hsl(var(--muted-foreground));
  font-size: 1.2rem;
}

.memory-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  width: 100%;
  max-width: 320px;
}

.mem-item {
  padding: 0.875rem;
  border-radius: var(--radius);
  text-align: center;
  font-weight: 600;
  font-size: 0.85rem;
  color: white;
}

.mem-item.safe { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.mem-item.auto { background: linear-gradient(135deg, #6366f1, #4f46e5); }
.mem-item.task { background: linear-gradient(135deg, #ec4899, #db2777); grid-column: span 2; }

.platform-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  width: 100%;
  max-width: 320px;
}

.plat-item {
  padding: 0.875rem;
  border-radius: var(--radius);
  text-align: center;
  font-weight: 600;
  font-size: 0.85rem;
  color: white;
}

.plat-item.desktop { background: linear-gradient(135deg, #6366f1, #4f46e5); }
.plat-item.mobile { background: linear-gradient(135deg, #ec4899, #db2777); }
.plat-item.soc { background: linear-gradient(135deg, #f59e0b, #d97706); }
.plat-item.mcu { background: linear-gradient(135deg, #14b8a6, #0d9488); }
.plat-item.web { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.plat-item.vehicle { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  }
}
</style>
