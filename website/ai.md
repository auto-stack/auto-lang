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
  badge="AutoAI Architecture"
  title=": Client / Daemon AI Infrastructure"
  description="A unified AI infrastructure for all AutoOS applications. Concurrency arbitration, API key vault, model routing, and usage tracking — all through a single daemon."
  primary-text="Read the Docs"
  primary-link="/docs/ai"
  secondary-text="Try Playground"
  secondary-link="/playground"
/>

<div class="stats-section">
  <h2 class="section-title">AutoAI by the Numbers</h2>
  <div class="stats-grid">
    <StatCard value="4" label="Core Crates" description="ai-config, auto-ai-daemon, auto-ai-client, auto-ai-agent." color="#ec4899" />
    <StatCard value="2" label="Agent Apps" description="AutoAI-Cli and AutoMusk, both built on the shared infrastructure." color="#a855f7" />
    <StatCard value="1" label="Daemon" description="aaid — the single LLM gateway for all AutoOS apps." color="#6366f1" />
    <StatCard value="∞" label="Models" description="Provider-agnostic routing across OpenAI, Anthropic, Zhipu, and more." color="#14b8a6" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="Client / Daemon Architecture"
    description="AutoAI introduces a clean separation between applications and LLM providers. Apps never talk to LLMs directly — every request goes through the aaid daemon."
    badge="Architecture"
  >
    <ul>
      <li><strong>auto-ai-daemon (aaid)</strong> — the single LLM gateway. Owns all provider knowledge, concurrency pools, key vault, and usage tracking.</li>
      <li><strong>auto-ai-client</strong> — thin HTTP client. Sends canonical requests, receives canonical responses. No provider knowledge.</li>
      <li><strong>ai-config</strong> — shared wire types and provider configuration. Canonical ContentBlock model.</li>
      <li><strong>auto-ai-agent</strong> — Profession library, ReAct loop, Workflow engine. Validates models via ai-config.</li>
    </ul>
    <template #visual>
      <div class="arch-diagram">
        <div class="arch-box apps">Apps<br /><small>AutoAI-Cli · AutoMusk · Forge</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box client">auto-ai-client<br /><small>canonical HTTP</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box daemon">aaid daemon<br /><small>concurrency · keys · routing</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box providers">LLM Providers<br /><small>OpenAI · Anthropic · Zhipu</small></div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="AutoMusk — General Coding Agent"
    description="A general-purpose coding agent built on AutoPlan, implemented in Auto itself."
    badge="Agent"
    reverse
  >
    <ul>
      <li><strong>AutoPlan integration</strong> — structured planning and execution</li>
      <li><strong>Multi-provider support</strong> — works with any model served by aaid</li>
      <li><strong>Self-hosting</strong> — written in Auto, runs on AutoVM</li>
      <li><strong>Config via auto-os-config</strong> — unified settings management</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">automusk.at</span>
        </div>
        <pre class="code-body"><code><span class="keyword">use</span> auto_ai_agent::Agent;
<span class="keyword">use</span> auto_ai_client::AiClient;

<span class="keyword">fn</span> <span class="function">main</span>() {
    <span class="keyword">let</span> client = AiClient::new();
    <span class="keyword">let</span> agent = Agent::from_profession(
        <span class="string">"coder"</span>, client
    );
    agent.run(<span class="string">"fix the parser bug"</span>);
}</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="AutoAI-Cli — Terminal Coding Agent"
    description="A lightweight terminal coding agent for quick tasks and shell integration."
    badge="CLI"
  >
    <ul>
      <li><strong>Terminal-native</strong> — coding help without leaving the shell</li>
      <li><strong>Same infrastructure</strong> — uses auto-ai-client and aaid</li>
      <li><strong>AutoShell integration</strong> — F3 AI mode inside AutoShell</li>
      <li><strong>Low overhead</strong> — start fast, answer fast</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">terminal</span>
        </div>
        <pre class="code-body"><code><span class="prompt">$</span> aictl status
<span class="output">Daemon: running (pid 1234)
Providers: zhipu, anthropic
Active pools: 2/8</span>

<span class="prompt">$</span> auto-ai-cli "explain this error"
<span class="output">The error occurs because...</span></code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Unified Configuration"
    description="All AutoAI apps share configuration through auto-os-config and .at files."
    badge="Config"
    reverse
  >
    <ul>
      <li><strong>~/.config/autoos/ai-client.at</strong> — providers + defaults</li>
      <li><strong>~/.config/autoos/ai-daemon.at</strong> — listen address, concurrency, real API keys</li>
      <li><strong>auto-os-config</strong> — web UI to edit all config modules</li>
      <li><strong>Env fallback</strong> — ZHIPU_API_KEY, ANTHROPIC_API_KEY, OPENAI_API_KEY</li>
    </ul>
    <template #visual>
      <div class="config-tree">
        <div class="config-file">~/.config/autoos/</div>
        <div class="config-item">├── ai-client.at</div>
        <div class="config-item">├── ai-daemon.at</div>
        <div class="config-item">├── auto-musk.at</div>
        <div class="config-item">├── roles/</div>
        <div class="config-item">└── skills/</div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">AutoAI Advantages</h2>
  <div class="features-grid">
    <FeatureCard icon="🎯" title="Single Gateway" description="One daemon owns all LLM communication. Apps stay simple and provider-agnostic." color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🔐" title="Key Vault" description="API keys live in the daemon, not in every app. Secure, centralized management." color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="⚡" title="Concurrency Arbitration" description="Semaphore pools per provider prevent rate-limit storms across apps." color="rgba(99, 102, 241, 0.15)" />
    <FeatureCard icon="📊" title="Usage Tracking" description="Per-app token and request tracking. Know exactly what each agent costs." color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🔄" title="Model Routing" description="Route requests to the best available model. Failover and fallback built in." color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🧩" title="Agent Framework" description="Profession, Workflow, and ReAct primitives for building complex agents." color="rgba(59, 130, 246, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Build AI-powered apps with Auto</h2>
  <p class="section-desc">Start the aaid daemon, link auto-ai-client, and ship your first agent in minutes.</p>
  <div class="cta-actions">
    <a href="/docs/design/15-ai-daemon-infrastructure" class="cta-btn cta-primary">Read Design Doc</a>
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
  background: linear-gradient(180deg, transparent 0%, rgba(236, 72, 153, 0.05) 100%);
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
  background: linear-gradient(135deg, #ec4899 0%, #a855f7 100%);
  color: white;
  box-shadow: 0 4px 14px rgba(236, 72, 153, 0.3);
}

.cta-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(236, 72, 153, 0.4);
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
.prompt { color: #a6e3a1; }
.output { color: #6c7086; }

.arch-diagram {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  max-width: 320px;
}

.arch-box {
  width: 100%;
  padding: 1rem 1.25rem;
  border-radius: var(--radius);
  text-align: center;
  font-weight: 700;
  font-size: 0.9rem;
  color: white;
}

.arch-box small {
  display: block;
  font-weight: 400;
  font-size: 0.75rem;
  opacity: 0.85;
  margin-top: 0.25rem;
}

.arch-box.apps { background: linear-gradient(135deg, #ec4899, #db2777); }
.arch-box.client { background: linear-gradient(135deg, #a855f7, #8b5cf6); }
.arch-box.daemon { background: linear-gradient(135deg, #6366f1, #4f46e5); }
.arch-box.providers { background: linear-gradient(135deg, #14b8a6, #0d9488); }

.arch-arrow {
  color: hsl(var(--muted-foreground));
  font-size: 1.2rem;
}

.config-tree {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: var(--radius);
  padding: 1.5rem;
  width: 100%;
  max-width: 320px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.85rem;
  line-height: 1.8;
}

.config-file {
  color: hsl(var(--foreground));
  font-weight: 700;
  margin-bottom: 0.5rem;
}

.config-item {
  color: hsl(var(--muted-foreground));
}

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  }
}
</style>
