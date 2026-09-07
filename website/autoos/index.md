---
layout: home
---

<script setup>
import OSHero from '../.vitepress/theme/components/OSHero.vue'
import FeatureCard from '../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #14b8a6; --page-accent-2: #3b82f6">

<OSHero
  badge="AutoOS · v0.5"
  title="A Desktop in a Window"
  description="The window manager itself is an AutoUI app. The virtual desktop fits AutoUI apps into a unified cross-platform desktop shell — the same window semantics from Windows to Linux to HarmonyOS."
  primary-text="Read the OS Docs"
  primary-link="/docs/os"
  secondary-text="Virtual Desktop Design"
  secondary-link="/docs/design/autoui/virtual-desktop"
/>

<div class="stats-section">
  <h2 class="section-title">Virtual Desktop · Key Facts</h2>
  <div class="stats-grid">
    <StatCard value="1" label="OS Window" description="On Win/Mac the host is a single-window virtual compositor; app windows are virtual windows inside it." color="#14b8a6" />
    <StatCard value="100%" label="Written in AutoUI" description="Chrome, dragging, resize, focus, taskbar — the window manager itself is an AutoUI app." color="#3b82f6" />
    <StatCard value="4" label="Leaf Seams" description="One set of WM code: iced element tree, offscreen texture, Wayland surface, DOM node." color="#8b5cf6" />
    <StatCard value="2" label="Growth Paths" description="A standalone AutoOS distro (Pop!_OS/COSMIC) or a virtual desktop embedded in existing systems." color="#ec4899" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="One Desktop for All Your Apps"
    description="Taskbar, start menu, multiple virtual windows, theming — all drawn by AutoUI. This is not a window manager with a skin; it is itself an AutoUI app."
    badge="Desktop Shell"
  >
    <ul>
      <li><strong>Start menu & taskbar</strong> — app launching, focus switching, window summoning: a complete set of desktop interactions.</li>
      <li><strong>Multiple virtual windows</strong> — DualApp shows several windows on screen, with dragging, resizing, and focus all interactive.</li>
      <li><strong>Theming</strong> — wallpaper, light/dark themes, and accent tiers switch in real time.</li>
    </ul>
    <template #visual>
      <div class="shot-stack">
        <img src="/v05/desktop-hero.png" alt="AutoOS Virtual Desktop: taskbar, start menu, and multiple windows" class="shot-main" />
        <div class="shot-pair">
          <img src="/v05/desktop-multiwindow.png" alt="Multiple virtual windows" />
          <img src="/v05/desktop-light.png" alt="Light-themed desktop" />
        </div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Unified Settings Center — auto-os-config"
    description="One daemon, one generic editor, serving every config module. URLs map to .at files by convention, and forms render automatically from data shapes — new modules need zero frontend code."
    badge="Settings Center"
  >
    <ul>
      <li><strong>Module registry</strong> — drop a .at file into modules.d/ and it registers automatically.</li>
      <li><strong>AI daemon management</strong> — aaid, Roles, Skills, and Musk configs manageable out of the box.</li>
      <li><strong>Theme system</strong> — accent tiers (indigo/coral/ocean/sage/amber) switch in real time.</li>
    </ul>
    <template #visual>
      <div class="shot-pair">
        <img src="/v05/autoos-config-agents.png" alt="Settings Center, Agents module" />
        <img src="/v05/autoos-config-skills.png" alt="Settings Center, Skills module" />
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="The First Real App — auto-kanban"
    description="A config-driven, general-purpose read-only kanban board, written almost 100% in Auto (13 .at files, zero hand-written Rust). One source, rendered on both Web and desktop."
    badge="App"
    reverse
  >
    <ul>
      <li><strong>Config-driven</strong> — the board's look is described by .at config; switch scenarios without touching code.</li>
      <li><strong>Dual-track parity</strong> — the Vue web client and the iced desktop client are verified item by item.</li>
      <li><strong>First of the v0.6 system apps</strong> — it is the first member of AutoOS's built-in app matrix.</li>
    </ul>
    <template #visual>
      <div class="shot-pair">
        <img src="/v05/kanban-web.png" alt="auto-kanban web edition" />
        <img src="/v05/kanban-desktop.png" alt="auto-kanban desktop edition" />
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Two Roads to AutoOS"
    description="One shared core architecture, growing in two directions."
    badge="Roadmap"
  >
    <ul>
      <li><strong>Embedded Virtual Desktop</strong> — runs inside Windows, Linux, macOS, and HarmonyOS; v0.6 polishes the cross-platform experience and adds AutoWeb-based remote desktop.</li>
      <li><strong>Standalone AutoOS Distro</strong> — an ISO image built on Pop!_OS and COSMIC Desktop, replacing stock apps with system apps written in Auto.</li>
    </ul>
    <template #visual>
      <div class="arch-diagram">
        <div class="arch-box clients">Apps<br /><small>Kanban · Settings Center · Terminal · System app matrix</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box daemons">Virtual desktop shell<br /><small>WM-as-App · single-window virtual compositor</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box system">Host<br /><small>Windows / Linux (Smithay) / HarmonyOS / distro</small></div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">The v0.6 System App Matrix (Preview)</h2>
  <div class="features-grid">
    <FeatureCard icon="📝" title="Text Editor" description="Code highlighting + AutoDown support." color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🧮" title="Calculator" description="Scientific and programming modes." color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🔍" title="Launcher" description="Everything-style instant file search." color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="📊" title="Task Manager" description="HTOP-like system monitoring." color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="📁" title="File Browser" description="Dual-pane and keyboard-driven." color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🔀" title="File Comparator" description="Beyond Compare-style diffing." color="rgba(139, 92, 246, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Explore AutoOS</h2>
  <div class="cta-actions">
    <a href="/docs/os" class="cta-btn cta-primary">Read the OS Docs</a>
    <a href="/v05/" class="cta-btn cta-secondary">Back to v0.5 Release Highlights</a>
  </div>
</div>

</div>

<style scoped>
.shot-stack {
  width: 100%;
  max-width: 480px;
}

.shot-main {
  width: 100%;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
}

.shot-pair {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  width: 100%;
  max-width: 480px;
  margin-top: 0.75rem;
}

.shot-pair img {
  width: 100%;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
}
</style>
