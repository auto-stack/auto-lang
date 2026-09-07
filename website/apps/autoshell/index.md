---
layout: home
---

<script setup>
import FeatureCard from '../../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #f59e0b; --page-accent-2: #3b82f6">

<div class="landing-hero">
  <div class="badge">AutoOS Flagship App · Beta</div>
  <h1 class="title">AutoShell: <span class="accent">The Structured Shell</span></h1>
  <p class="description">
    The cross-platform shell for the AI era. Typed objects, not strings, flow between commands;
    with a built-in security sandbox and 79 agent tools — it is both your daily shell and a safe execution layer for AI agents.
  </p>
  <div class="actions">
    <a href="/v05/" class="btn btn-primary">Back to v0.5 Release Highlights</a>
    <a href="/apps" class="btn btn-secondary">View All Apps</a>
  </div>
</div>

<div class="stats-section">
  <h2 class="section-title">ASH at a Glance</h2>
  <div class="stats-grid">
    <StatCard value="18" label="Semantic Types" description="Atom objects flow through the pipeline, not strings." color="#f59e0b" />
    <StatCard value="80+" label="Built-in Commands" description="Consistent behavior across Windows, Linux, and macOS." color="#3b82f6" />
    <StatCard value="79" label="Agent Tools" description="Fully described by JSON Schema for safe calls from AI agents." color="#8b5cf6" />
    <StatCard value="F1-F4" label="Four Modes" description="Shell / AutoScript / AI translate / AI chat, one keypress away." color="#14b8a6" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="Structured Pipelines"
    description="Say goodbye to the string craft of grep/awk/sed. Commands return typed objects; filter and sort go straight to fields."
    badge="Pipeline"
  >
    <ul>
      <li><strong>Typed data flow</strong> — 18 semantic types pass between commands unchanged.</li>
      <li><strong>Native format conversion</strong> — from_json / to_csv / from_yaml / from_toml / from_xml, no jq required.</li>
      <li><strong>AutoLang built in</strong> — closures, try/catch, and a type system; scripting power comes from Auto.</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">ash — structured pipeline</span>
        </div>
        <pre class="code-body"><code><span class="prompt">❯</span> ls | <span class="function">filter</span> .size &gt; 10.mb | <span class="function">sort</span> .name | <span class="function">to_csv</span>
<span class="output">name,size,modified
logs,1.2.gb,2026-09-05
target,3.8.gb,2026-09-07</span>
<span class="prompt">❯</span> from_json package.json | <span class="function">get</span> .dependencies | <span class="function">length</span>
<span class="output">42</span></code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="A Double Life in the AI Era"
    description="For humans, a highly efficient shell; for agents, an execution layer with safety boundaries."
    badge="AI Ready"
    reverse
  >
    <ul>
      <li><strong>Security sandbox</strong> — --sandbox / --read-only / --no-network / --audit lock down every agent step.</li>
      <li><strong>Agent CLI</strong> — ash agent describe-tools emits the JSON Schema for all 79 tools; check/run closes the validate-then-execute loop.</li>
      <li><strong>F3/F4 AI modes</strong> — translate natural language straight into shell commands or chat directly, powered by the aaid daemon.</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">ash — agent loop</span>
        </div>
        <pre class="code-body"><code><span class="prompt">❯</span> ash agent describe-tools | <span class="function">length</span>
<span class="output">79 tools with JSON Schema</span>
<span class="prompt">❯</span> ash --sandbox --read-only agent run plan.at
<span class="output">[check] 12 steps validated
[run]   sandboxed execution, 0 writes</span></code></pre>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">Three Shapes, One Shell</h2>
  <div class="features-grid">
    <FeatureCard icon="⌨️" title="CLI" description="The purest form of pipes and scripts, replacing Bash/Fish/Zsh." color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🖥️" title="TUI" description="A full-screen interactive interface, keyboard-driven." color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🪟" title="GUI" description="A graphical shell described in AutoUI, sharing the desktop stack." color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="🌍" title="Cross-Platform" description="Consistent behavior on Windows, Linux, and macOS." color="rgba(20, 184, 166, 0.15)" />
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
