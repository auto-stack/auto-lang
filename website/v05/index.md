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
  description="Dynamic dev with static shipping, decoupled frontends, and Language as OS — three ideas have grown into a platform: the AutoOS desktop, four flagship apps, two-ecosystem scripting, and Bootstrap² achieved. This time, Auto starts to become itself."
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

<div class="features-section">
  <h2 class="section-title">Three Design Philosophies</h2>
  <p class="section-desc">Dynamic dev, decoupled frontends, Language as OS — every section that follows is evidence.</p>
  <div class="philosophy-grid">
    <article class="philosophy-card" style="--ph: #6366f1; --ph2: #38bdf8">
      <div class="philosophy-icon">🌓</div>
      <h3 class="philosophy-name">Dynamic Dev, Static Ship</h3>
      <p class="philosophy-en">One Source · Every Ecosystem</p>
      <p class="philosophy-tagline">A scripting language while you build, a systems language when you ship — the same source, in every ecosystem.</p>
      <p class="philosophy-text">Most languages make you choose: scripting gives you instant iteration but leaves performance at the door; systems languages deliver native speed but tax every idea with a compile cycle. Auto refuses to choose. In development, the AutoVM interprets your code — instant startup, hot reload, with REPL and LSP at your side. At release, the same source transpiles to native Rust via a2r (or to C via a2c). And across Auto's whole map, this dynamic-static pairing is the shared design origin of every ecosystem — not just Rust's.</p>
      <ul class="philosophy-list">
        <li><strong>A dynamic-static pairing for every ecosystem</strong> — UI development (VM/iced hot-reload preview ↔ transpiled release), embedded MCUs (the VM doubles as the simulator ↔ a2c cross-compiles and flashes), Godot (emit GDScript for in-editor hot tweaks ↔ stitch static C/Rust into the engine), scientific computing (use.py calls PyTorch directly ↔ a2r ships the service). Before entering any ecosystem, two questions must be answered: how does it run dynamically, and how does it ship statically?</li>
        <li><strong>Hot reload is the soul of the dynamic side</strong> — the value of dynamic mode is more than fast startup: edit without restarting, keep your running state, and see exactly what you just changed. The AutoUI desktop runs dual VM/a2r tracks with hot reload — the moment you save is the moment you see.</li>
        <li><strong>Consistency guarded by machines</strong> — the parity harness runs every test through AutoVM, transpiled Rust, and native Rust, requiring identical output; 20+ replicated third-party Rust libraries serve as standing regression corpus.</li>
      </ul>
      <div class="philosophy-proof"><code>auto run</code> (interpreted) ↔ <code>a2r</code> (native Rust) · dynamic-static across ecosystems · three-backend parity</div>
    </article>

    <article class="philosophy-card" style="--ph: #14b8a6; --ph2: #6366f1">
      <div class="philosophy-icon">🔌</div>
      <h3 class="philosophy-name">Decoupled at Every Layer</h3>
      <p class="philosophy-en">Frontend Expresses · Backend Delivers</p>
      <p class="philosophy-tagline">One architectural law from the UI down to the OS kernel — both ends of the seam stay replaceable.</p>
      <p class="philosophy-text">Decoupling isn't a UI trick — it's the meta-pattern Auto applies at every layer. Outermost is AutoUI: its contract is independent of any host framework, and the same .at switches rendering arms between Vue, iced, ArkTS, and Jetpack Compose. Moving inward, the pattern repeats — all the way up to the operating system itself, where shell and kernel are just another front and back.</p>
      <ul class="philosophy-list">
        <li><strong>OS shell ↔ OS kernel, backend swappable</strong> — on Windows, the frontend is the virtual desktop and the backend is the Windows kernel; in the future AutoOS distro, the AutoOS desktop couples directly to a Linux kernel; and the backend can switch to OpenHarmony. Change the kernel, not your apps.</li>
        <li><strong>One pattern, recurring at every layer</strong> — AutoUI ↔ rendering engines (Vue / iced / ArkTS / Jetpack); the AutoOS app architecture: auto-ui ↔ auto-compositor (RenderQueue, lock-free over shared memory); AutoAI: agents and AI-apps ↔ ai-daemon (LLM resources scheduled centrally).</li>
        <li><strong>The dividend: one language across the seam</strong> — the AutoDown knowledge base is 411 files and 32K lines of frontend + backend logic in a single .at source, rendering to web and desktop; AutoMusk's five frontend views are generated the same way, verified 148-for-148.</li>
      </ul>
      <div class="philosophy-proof">frontend · seam · backend — UI↔renderer / app↔compositor / agent↔daemon / shell↔kernel</div>
    </article>

    <article class="philosophy-card" style="--ph: #ec4899; --ph2: #a855f7">
      <div class="philosophy-icon">🖥️</div>
      <h3 class="philosophy-name">Language as OS</h3>
      <p class="philosophy-en">LAOS</p>
      <p class="philosophy-tagline">The language mirrors the OS architecture — every OS module becomes a language-level component.</p>
      <p class="philosophy-text">An operating system's three duties — sharing, abstraction, and services — are precisely the three questions a language must answer. Auto writes its answers into the language itself: the Task/Msg Actor model is the scheduler; the view/mut/move memory triple is memory management; the io/net/http/fs standard library is the system-call surface; multi-platform rendering arms are the device drivers. One step further: every OS module has two supply routes — implemented by Auto itself, or bridged to an existing implementation — and gets assembled per target platform.</p>
      <ul class="philosophy-list">
        <li><strong>Language-level modules, two supply routes</strong> — running on Windows: apps and UI transpile to Rust/iced while the kernel borrows Windows' own capabilities; running on an MCU: apps and UI can transpile to C/LVGL while the kernel becomes an RTOS written in Auto. One language, assembled differently per machine.</li>
        <li><strong>The shape of an OS is already visible</strong> — the AutoOS virtual desktop is running, and the window manager itself is an AutoUI app (next section); auto-os-config turns the shape of config files into the settings center, automatically.</li>
        <li><strong>AI compute scheduled like an OS device</strong> — the Client/Daemon architecture already powers AutoAI: ask the system for compute, instead of every app building its own AI stack.</li>
      </ul>
      <div class="philosophy-proof">scheduler · memory · syscalls · drivers — each self-implementable, each bridgeable</div>
    </article>
  </div>
</div>

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

.philosophy-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1.25rem;
  align-items: stretch;
}

.philosophy-card {
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 0.7rem;
  padding: 1.75rem 1.5rem 1.5rem 1.75rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border) / 0.7);
  background: hsl(var(--card));
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.philosophy-card:hover {
  transform: translateY(-3px);
  box-shadow: 0 10px 36px rgba(0, 0, 0, 0.1);
}

.philosophy-card::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  background: linear-gradient(180deg, var(--ph, #6366f1), var(--ph2, #a855f7));
}

.philosophy-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.3rem;
  background: color-mix(in srgb, var(--ph, #6366f1) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--ph, #6366f1) 25%, transparent);
}

.philosophy-name {
  margin: 0;
  font-size: 1.35rem;
  font-weight: 700;
  color: hsl(var(--foreground));
}

.philosophy-en {
  margin: -0.5rem 0 0;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.72rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--ph, #6366f1);
}

.philosophy-tagline {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.philosophy-text {
  margin: 0;
  font-size: 0.9rem;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
}

.philosophy-list {
  list-style: none;
  padding: 0;
  margin: 0.25rem 0 0;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.philosophy-list li {
  position: relative;
  padding-left: 1.2rem;
  font-size: 0.86rem;
  line-height: 1.65;
  color: hsl(var(--muted-foreground));
}

.philosophy-list li::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0.55rem;
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--ph, #6366f1);
}

.philosophy-list strong {
  color: hsl(var(--foreground));
}

.philosophy-proof {
  margin-top: auto;
  padding-top: 0.9rem;
  border-top: 1px dashed hsl(var(--border));
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.76rem;
  color: var(--ph, #6366f1);
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
