---
layout: home
---

<script setup>
import HomeHero from '../.vitepress/theme/components/HomeHero.vue'
import FeatureCard from '../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #6366f1; --page-accent-2: #a855f7">

<HomeHero
  badge="The biggest update yet"
  title=": v0.5 Is Here"
  description="Auto has grown from a language into a platform: the AutoOS desktop, four flagship apps, scripting for two ecosystems, and Bootstrap² achieved — this time, Auto starts to become itself."
  primary-text="Read the Release Notes"
  primary-link="/docs/releases/v0.5"
  secondary-text="Open Playground"
  secondary-link="/playground"
  hide-code
>
  <div class="hero-stats">
    <StatCard value="100 Billion" label="AI R&D Tokens" description="Platform-scale engineering written with deep AI involvement." color="#6366f1" />
    <StatCard value="5,700+" label="Commits" description="5,711 commits since v0.3, every one merged through dual test gates." color="#8b5cf6" />
    <StatCard value="578K+" label="Lines of Rust" description="Compiler, AutoVM, the iced desktop shell, and system services." color="#14b8a6" />
    <StatCard value="135K+" label="Lines of Auto" description="Bootstrap libraries, apps, and corpus — the platform's own language is growing." color="#ec4899" />
  </div>
  <p class="hero-stats-note">From v0.3 to v0.5 (Apr — Sep 2026), the quantified footprint of the Auto platform.</p>
</HomeHero>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="The AutoOS Virtual Desktop"
    description="The window manager itself is an AutoUI app. A virtual compositor inside a single OS window fits AutoUI apps into a cross-platform desktop — available on Windows today, with Linux desktop environments and HarmonyOS on the roadmap."
    badge="AutoOS"
  >
    <ul>
      <li><strong>WM-as-App</strong> — the chrome, dragging, focus, and taskbar of virtual windows are all written as ordinary AutoUI apps; dual-backend consistency is guaranteed by construction.</li>
      <li><strong>Unified Settings Center</strong> — auto-os-config: one daemon auto-renders config forms from .at file shapes, onboarding new modules with zero frontend code.</li>
      <li><strong>First real app</strong> — the auto-kanban board, written almost 100% in Auto, rendered on both Web and desktop.</li>
      <li><strong>AutoTerm terminal infrastructure</strong> — PTY + an alacritty emulation core, the terminal foundation of AutoOS.</li>
    </ul>
    <template #visual>
      <div class="shot-stack">
        <img src="/v05/desktop-hero.png" alt="AutoOS Virtual Desktop: taskbar, start menu, and multiple windows" class="shot-main" />
        <div class="shot-pair">
          <img src="/v05/kanban-desktop.png" alt="auto-kanban desktop edition" />
          <img src="/v05/autoos-config-agents.png" alt="AutoOS Unified Settings Center, Agents module" />
        </div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">Four Flagship Apps, 100% Built in Auto</h2>
  <p class="section-desc">Each app has its own landing page — proof that the platform works, and a reference for building your own.</p>
  <div class="features-grid">
    <FeatureCard icon="🤖" title="AutoMusk" description="A general-purpose coding agent driven by the AutoPlan mode — its five frontend views are generated from a single .at source. An agent written in Auto." color="rgba(236, 72, 153, 0.15)" link="/apps/automusk/" />
    <FeatureCard icon="🐚" title="AutoShell" description="The structured shell for the AI era. Commands exchange typed objects instead of text streams, with a built-in security sandbox and 79 agent tools." color="rgba(245, 158, 11, 0.15)" link="/apps/autoshell/" />
    <FeatureCard icon="📄" title="AutoDown" description="A Markdown+YAML dialect and Jade Garden, an Obsidian-like knowledge base. Frontend and backend logic live in one .at source, with Web and desktop shapes." color="rgba(99, 102, 241, 0.15)" link="/apps/autodown/" />
    <FeatureCard icon="🎨" title="AutoUI Apps" description="A 46+ component gallery, dozens of demos, and full-stack examples — all in Auto. A live gallery is embedded on this very page." color="rgba(139, 92, 246, 0.15)" link="/apps/autoui/" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="The New Playground: An Auto Lab in Your Browser"
    description="Nearly all Auto example code is browsable out of the box, with new Debug support — corpus, golden samples, and book snippets run right in your browser."
    badge="Playground"
    reverse
  >
    <ul>
      <li><strong>1280+ corpus notes</strong> — 460 VM golden samples, 158 AAVM bootstrap corpus files, 634 book fences, and 28 demo examples.</li>
      <li><strong>Debug support</strong> — pair it with a local backend to watch the AutoVM execute, no longer just a black box.</li>
      <li><strong>Run / transpile online</strong> — the same source, interpreted or transpiled to Rust/Python for parity checks.</li>
    </ul>
    <template #visual>
      <div class="stats-grid playground-stats">
        <StatCard value="460" label="VM Golden Samples" description="Corpus as tests." color="#6366f1" />
        <StatCard value="634" label="Book Fences" description="Eight books you can run as you read." color="#8b5cf6" />
        <StatCard value="Debug" label="Debug Support" description="Watch it execute." color="#14b8a6" />
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Language Progress: Two-Ecosystem Scripting + Bootstrap²"
    description="v0.5 moves Auto into both the Rust and Python ecosystems, and Auto's toolchain starts to be written in Auto itself."
    badge="Language"
  >
    <ul>
      <li><strong>Rust ecosystem (Alpha)</strong> — Auto as a scripting language for Rust: the AutoVM can call over 90% of Rust code, and Rust transpiled by a2r behaves essentially the same as the AutoVM.</li>
      <li><strong>Python ecosystem (PreAlpha)</strong> — ordinary Python scripts callable directly, with PyTorch support opening up the AI development environment.</li>
      <li><strong>Bootstrap² (2×2 matrix)</strong> — host side (avm, a2r) × bootstrap side (aavm, aa2r): a VM and a transpiler written in Auto are here, and the toolchain has started to run itself.</li>
    </ul>
    <template #visual>
      <div class="matrix">
        <div class="matrix-head">One .at corpus → four execution paths</div>
        <div class="matrix-grid">
          <div class="matrix-cell host"><strong>avm</strong><span>Rust host interpreter</span></div>
          <div class="matrix-cell host"><strong>a2r</strong><span>Transpiled to native Rust</span></div>
          <div class="matrix-cell boot"><strong>aavm</strong><span>VM written in Auto</span></div>
          <div class="matrix-cell boot"><strong>aa2r</strong><span>Transpiler written in Auto</span></div>
        </div>
        <div class="matrix-foot">No Rust-only core left in the bootstrap loop</div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">Looking Ahead to v0.6</h2>
  <p class="section-desc">Next stop: from platform to operating system.</p>
  <div class="features-grid">
    <FeatureCard icon="🖥️" title="AutoOS" description="A standalone Linux distro (based on Pop!_OS/COSMIC); a polished cross-platform virtual desktop; AutoWeb-based remote desktop." color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="📱" title="AutoUI" description="HarmonyOS ecosystem support; initial Android / iOS support via Jetpack Compose." color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="🤖" title="ROS2 Ecosystem" description="Robotics node development on Rust/Python, plugging into DORA-RS and beyond." color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🎮" title="Godot Ecosystem" description="Fuller GDScript support and native Scene/Node capabilities, fitting into the Godot development workflow." color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🔩" title="MCU Ecosystem" description="Embedded: Auto transpiles to C, and the AutoMan build system takes Auto into the microcontroller world." color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🧠" title="AutoAI" description="AutoMusk ready for daily use (Beta); a communication architecture and resource scheduling for AI apps." color="rgba(99, 102, 241, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Try Auto Today</h2>
  <p class="section-desc">Run scripts instantly, ship as Rust — from the browser to your machine, up and running in five minutes.</p>
  <div class="cta-actions">
    <a href="/docs/" class="cta-btn cta-primary">Get Started</a>
    <a href="/playground" class="cta-btn cta-secondary">Open Playground</a>
  </div>
</div>

</div>

<style scoped>
.hero-stats {
  width: min(1080px, 92vw);
  margin-left: 50%;
  transform: translateX(-50%);
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(230px, 1fr));
  gap: 1rem;
}

.hero-stats-note {
  margin: 0.75rem auto 0;
  font-size: 0.85rem;
  color: hsl(var(--muted-foreground));
}

.shot-stack {
  width: 100%;
  max-width: 480px;
}

.shot-main,
.shot-pair img {
  width: 100%;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
}

.shot-pair {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  margin-top: 0.75rem;
}

.playground-stats {
  max-width: 480px;
}

.matrix {
  width: 100%;
  max-width: 440px;
}

.matrix-head,
.matrix-foot {
  font-size: 0.85rem;
  color: hsl(var(--muted-foreground));
  text-align: center;
  margin-bottom: 0.75rem;
}

.matrix-foot {
  margin: 0.75rem 0 0;
  color: #a855f7;
  font-weight: 600;
}

.matrix-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
}

.matrix-cell {
  padding: 1.25rem 1rem;
  border-radius: var(--radius);
  text-align: center;
  border: 1px solid hsl(var(--border));
}

.matrix-cell strong {
  display: block;
  font-size: 1.5rem;
  font-family: 'JetBrains Mono', monospace;
  color: hsl(var(--foreground));
}

.matrix-cell span {
  font-size: 0.8rem;
  color: hsl(var(--muted-foreground));
}

.matrix-cell.host {
  background: rgba(99, 102, 241, 0.08);
  border-color: rgba(99, 102, 241, 0.3);
}

.matrix-cell.boot {
  background: rgba(168, 85, 247, 0.1);
  border-color: rgba(168, 85, 247, 0.4);
}
</style>
