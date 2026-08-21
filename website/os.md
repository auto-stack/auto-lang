---
layout: home
---

<script setup>
import OSHero from './.vitepress/theme/components/OSHero.vue'
import FeatureCard from './.vitepress/theme/components/FeatureCard.vue'
import StatCard from './.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from './.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #14b8a6; --page-accent-2: #3b82f6">

<OSHero
  badge="AutoOS Architecture"
  title=": Client / Daemon OS Foundation"
  description="AutoOS is evolving into a full operating system layer. Client/Daemon architecture, unified configuration, and two future paths: standalone distro and embedded virtual desktop."
  primary-text="Read the Docs"
  primary-link="/docs/os"
  secondary-text="Explore auto-os-config"
  secondary-link="/docs/os#auto-os-config"
/>

<div class="stats-section">
  <h2 class="section-title">AutoOS Foundation</h2>
  <div class="stats-grid">
    <StatCard value="1" label="Config Daemon" description="auto-os-config — one daemon for every config module." color="#14b8a6" />
    <StatCard value="2" label="Future Paths" description="Standalone AutoOS distro or embedded virtual desktop." color="#3b82f6" />
    <StatCard value="4+" label="Config Modules" description="AI Daemon, Harness, Skills, Roles, Auto Musk, and more." color="#8b5cf6" />
    <StatCard value="0" label="Frontend Code" description="Generic editor auto-renders forms from .at file shapes." color="#f59e0b" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="auto-os-config — Unified Settings Center"
    description="One daemon, one generic editor, for every config module. A Vue 3 SPA + Rust backend that reads/writes any .at config file directly."
    badge="Config"
  >
    <ul>
      <li><strong>Unified daemon</strong> — the only config read/write service. URL → file path by convention.</li>
      <li><strong>Generic editor</strong> — renders forms from .at data shape + key-name conventions. Zero frontend code for new modules.</li>
      <li><strong>Module registry</strong> — drop-in .at files in <code>modules.d/</code> register new modules automatically.</li>
      <li><strong>Custom UX</strong> — remote Vue components via <code>createComponent(Vue)</code> factory when the generic editor isn't enough.</li>
    </ul>
    <template #visual>
      <div class="config-tree">
        <div class="config-file">~/.config/autoos/</div>
        <div class="config-item">├── ai-client.at</div>
        <div class="config-item">├── ai-daemon.at</div>
        <div class="config-item">├── auto-musk.at</div>
        <div class="config-item">├── modules.d/</div>
        <div class="config-item">│   └── my-module.at</div>
        <div class="config-item">├── roles/</div>
        <div class="config-item">└── skills/</div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Client / Daemon Architecture"
    description="AutoOS apps follow a consistent pattern: a system daemon owns shared state and resources; thin clients connect to it."
    badge="Architecture"
    reverse
  >
    <ul>
      <li><strong>aaid</strong> — AI daemon for LLM routing, concurrency, and usage tracking</li>
      <li><strong>auto-os-config-daemon</strong> — unified config read/write service</li>
      <li><strong>AutoShell daemon</strong> — shell session and job management</li>
      <li><strong>Future</strong> — window manager, file system, and device daemons</li>
    </ul>
    <template #visual>
      <div class="arch-diagram">
        <div class="arch-box clients">Clients<br /><small>AutoShell · AutoMusk · Config UI</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box daemons">Daemons<br /><small>aaid · config · shell</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box system">System<br /><small>~/.config/autoos · .at files</small></div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Two Paths to AutoOS"
    description="AutoOS is designed to grow in two directions, sharing the same core architecture."
    badge="Roadmap"
  >
    <ul>
      <li><strong>Standalone AutoOS</strong> — based on Pop!_OS and COSMIC Desktop. AutoOS ISO image with Auto-native system apps.</li>
      <li><strong>Embedded Virtual Desktop</strong> — runs inside Windows, Linux, macOS, and HarmonyOS. A virtual desktop OS built with AutoUI.</li>
    </ul>
    <template #visual>
      <div class="path-grid">
        <div class="path-card standalone">
          <h4>Standalone</h4>
          <p>Pop!_OS + COSMIC</p>
          <span>Full distro</span>
        </div>
        <div class="path-card embedded">
          <h4>Embedded</h4>
          <p>AutoUI Virtual Desktop</p>
          <span>Windows · Linux · macOS · Harmony</span>
        </div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="System Apps"
    description="AutoOS will ship with a full suite of native applications, all written in Auto."
    badge="Apps"
    reverse
  >
    <ul>
      <li><strong>Text editor</strong> — code highlighting and AutoDown support</li>
      <li><strong>Calculator</strong> — scientific and programming modes</li>
      <li><strong>Minesweeper</strong> — classic game, AutoUI implementation</li>
      <li><strong>Calendar</strong> — schedule management</li>
      <li><strong>Launcher</strong> — Everything-style file search</li>
      <li><strong>Task manager</strong> — HTOP-like system monitor</li>
      <li><strong>File browser</strong> — dual-pane, keyboard-driven</li>
      <li><strong>File comparator</strong> — Beyond Compare-style diff</li>
    </ul>
    <template #visual>
      <div class="apps-grid-visual">
        <div class="app-icon">📝</div>
        <div class="app-icon">🧮</div>
        <div class="app-icon">💣</div>
        <div class="app-icon">📅</div>
        <div class="app-icon">🔍</div>
        <div class="app-icon">📊</div>
        <div class="app-icon">📁</div>
        <div class="app-icon">🔀</div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">AutoOS Design Principles</h2>
  <div class="features-grid">
    <FeatureCard icon="⚙️" title="One Config Format" description="All system settings use .at (auto-atom) files. Consistent, parseable, and version-controllable." color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🔌" title="Daemon-First" description="Shared state lives in daemons, not in apps. Apps are thin, replaceable clients." color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🎨" title="AutoUI Native" description="System UI is built with AutoUI. One framework for desktop, web, and mobile." color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="🤖" title="AI Integrated" description="AI capabilities are system services, not app add-ons. Every app can use AI through aaid." color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="📦" title="Drop-in Modules" description="New config modules register by dropping a .at file into modules.d/. No source changes needed." color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🌐" title="Cross-Platform" description="Run as a standalone distro or embed into existing operating systems." color="rgba(139, 92, 246, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Explore AutoOS</h2>
  <p class="section-desc">Read the design documents, try auto-os-config locally, or follow the roadmap to a full operating system.</p>
  <div class="cta-actions">
    <a href="/docs/os" class="cta-btn cta-primary">Read OS Docs</a>
    <a href="/docs/releases/v0.5" class="cta-btn cta-secondary">v0.5 Release Notes</a>
  </div>
</div>

</div>

<style scoped>
.arch-box.clients { background: linear-gradient(135deg, #14b8a6, #0d9488); }
.arch-box.daemons { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.arch-box.system { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }

.path-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  width: 100%;
  max-width: 420px;
}

.path-card {
  padding: 1.5rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  text-align: center;
}

.path-card h4 {
  margin: 0 0 0.5rem;
  color: hsl(var(--foreground));
}

.path-card p {
  margin: 0 0 0.5rem;
  font-size: 0.875rem;
  color: hsl(var(--muted-foreground));
}

.path-card span {
  font-size: 0.8rem;
  color: #14b8a6;
  font-weight: 600;
}

.path-card.standalone {
  border-color: rgba(20, 184, 166, 0.3);
  background: rgba(20, 184, 166, 0.05);
}

.path-card.embedded {
  border-color: rgba(59, 130, 246, 0.3);
  background: rgba(59, 130, 246, 0.05);
}

.apps-grid-visual {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 0.75rem;
  width: 100%;
  max-width: 320px;
}

.app-icon {
  width: 100%;
  aspect-ratio: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.5rem;
  border-radius: var(--radius);
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
}

@media (max-width: 768px) {
  .path-grid {
    grid-template-columns: 1fr;
  }
}
</style>
