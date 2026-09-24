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
  description="Dynamic dev with static shipping, decoupled frontends, and Language as OS — three ideas have grown into a platform: the AutoUI dual-backend architecture has matured, the AutoOS virtual desktop is becoming usable, and four flagship apps plus 28 system apps are all written in Auto. This time, Auto starts to become itself."
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
      <ul class="philosophy-list">
        <li class="sub">Most languages make you choose: scripting gives instant iteration but leaves performance at the door; systems languages deliver native speed but tax every idea with a compile cycle. Auto refuses to choose.</li>
        <li class="sub">In development, the AutoVM interprets your code — instant startup, hot reload, with REPL and LSP at your side.</li>
        <li class="sub">At release, the same source transpiles to native Rust via a2r (or to C via a2c).</li>
        <li class="sub">Not just Rust — this dynamic-static pairing is the shared design origin of every ecosystem across Auto's map.</li>
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
      <ul class="philosophy-list">
        <li class="sub">Decoupling isn't a UI trick — it's the meta-pattern Auto applies at every layer.</li>
        <li class="sub">Outermost is AutoUI: its contract is independent of any host framework, and the same .at switches rendering arms between Vue, iced, ArkTS, and Jetpack Compose.</li>
        <li class="sub">Moving inward, the pattern repeats — all the way up to the operating system itself, where shell and kernel are just another front and back.</li>
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
      <ul class="philosophy-list">
        <li class="sub">An operating system's three duties — sharing, abstraction, and services — are precisely the three questions a language must answer.</li>
        <li class="sub">Auto writes its answers into the language itself: the Task/Msg Actor model is the scheduler; the view/mut/move memory triple is memory management; the io/net/http/fs standard library is the system-call surface; multi-platform rendering arms are the device drivers.</li>
        <li class="sub">One step further: every OS module has two supply routes — implemented by Auto itself, or bridged to an existing implementation — and gets assembled per target platform.</li>
        <li><strong>Language-level modules, two supply routes</strong> — running on Windows: apps and UI transpile to Rust/iced while the kernel borrows Windows' own capabilities; running on an MCU: apps and UI can transpile to C/LVGL while the kernel becomes an RTOS written in Auto. One language, assembled differently per machine.</li>
        <li><strong>The shape of an OS is already visible</strong> — the AutoOS virtual desktop is running, and the window manager itself is an AutoUI app (next section); auto-os-config turns the shape of config files into the settings center, automatically.</li>
        <li><strong>AI compute scheduled like an OS device</strong> — the Client/Daemon architecture already powers AutoAI: ask the system for compute, instead of every app building its own AI stack.</li>
      </ul>
      <div class="philosophy-proof">scheduler · memory · syscalls · drivers — each self-implementable, each bridgeable</div>
    </article>
  </div>
</div>

<div class="features-section">
  <h2 class="section-title">Five Months, Three Steps</h2>
  <p class="section-desc">From v0.3 to v0.5 (Apr — Sep 2026), each release took on a new question to answer.</p>
  <div class="journey">
    <p class="journey-lead">v0.3 answered <em>"is it a language?"</em>; v0.4 answered <em>"can it run real things?"</em>. With v0.5, the question is the hardest one yet: <em>"can it become something you open every day?"</em></p>
    <p class="journey-lead">To answer it, we wrote 578K lines of Rust and 135K lines of Auto over the past five months — but the line counts are just footnotes. The real change: Auto no longer lives only in compilers and test corpora. It has grown an interface, a desktop, and a whole app ecosystem. Here is the report, in order.</p>
    <div class="timeline">
      <div class="timeline-item">
        <span class="timeline-version">v0.3</span>
        <div class="timeline-card">
          <h3>The language kernel stood up</h3>
          <p>Type system, ownership and borrowing, pattern matching — the foundation of a language was laid.</p>
        </div>
      </div>
      <div class="timeline-item">
        <span class="timeline-version">v0.4</span>
        <div class="timeline-card">
          <h3>Runtime and transpilers</h3>
          <p>The AutoVM became fully capable, a2r transpilation reached production grade, and the AI-agent infrastructure was built — Auto started running real applications.</p>
        </div>
      </div>
      <div class="timeline-item">
        <span class="timeline-version">v0.5</span>
        <div class="timeline-card">
          <h3>Desktop and app ecosystem</h3>
          <p>The AutoUI dual-backend architecture matured, the AutoOS virtual desktop became usable, and four flagship apps plus 28 system apps were all written in Auto.</p>
        </div>
      </div>
    </div>
  </div>
</div>

<div class="features-section vd-section">
  <span class="kicker">AutoOS · Headline</span>
  <h2 class="section-title">The AutoOS Virtual Desktop</h2>
  <p class="section-desc">A virtual compositor inside a single OS window fits AutoUI apps into a cross-platform desktop — available on Windows today, with Linux desktop environments and HarmonyOS on the roadmap.</p>
  <div class="vd-intro">
    <p class="narrative">This time, the first thing we want to show you isn't a language feature — it's a desktop you can use. The window manager itself is an AutoUI app: not a single system window API was called; dragging, focus, and the taskbar are all ordinary Auto code. Both light and dark themes are in place, and the four flagship apps plus 28 system apps ship inside it.</p>
    <ul class="vd-points">
      <li><strong>WM-as-App</strong> — the chrome, dragging, focus, and taskbar of virtual windows are all written as ordinary AutoUI apps; dual-backend consistency is guaranteed by construction.</li>
      <li><strong>Light & dark themes</strong> — the desktop and the main apps ship both themes, switchable at runtime (both shown below).</li>
      <li><strong>The desktop is the app container</strong> — the start menu lists everything: four flagship apps and 20+ system apps open as real windows on the desktop, in the full-width shots below.</li>
      <li><strong>Unified Settings Center</strong> — auto-os-config: one daemon auto-renders config forms from .at file shapes, onboarding new modules with zero frontend code.</li>
      <li><strong>AutoTerm terminal infrastructure</strong> — PTY + an alacritty emulation core, the terminal foundation of AutoOS.</li>
    </ul>
  </div>
  <figure class="big-shot">
    <!-- exists: dark main view -->
    <img src="/v05/desktop-hero.png" alt="AutoOS Virtual Desktop (dark): taskbar, start menu, and multiple windows" />
    <figcaption>Dark theme, main view — taskbar, start menu, and multiple windows</figcaption>
  </figure>
  <figure class="big-shot">
    <!-- exists: light-theme desktop -->
    <img src="/v05/desktop-light.png" alt="AutoOS Virtual Desktop (light theme)" />
    <figcaption>The same desktop in the light theme</figcaption>
  </figure>
  <figure class="big-shot">
    <div class="shot-placeholder">TODO screenshot · several system apps open as windows on the desktop (suggestion: music-player + file-manager + settings, dark)</div>
    <figcaption>Desktop tour I — system apps running as windows</figcaption>
  </figure>
  <figure class="big-shot">
    <div class="shot-placeholder">TODO screenshot · start menu + a game app window (suggestion: FreeCell / Minesweeper / Tetris; light theme welcome)</div>
    <figcaption>Desktop tour II — start menu and games</figcaption>
  </figure>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="AutoUI: One Contract, Two Renderers"
    description="Dual Web + desktop support: the UI contract is independent of any host framework, and the same .at source generates both the Vue (Web) and iced (desktop) ends."
    badge="AutoUI · Headline"
    reverse
  >
    <p class="narrative">The desktop runs because of an earlier decision: pulling the UI contract out of the renderer. We worked on this architecture for two years, and v0.5 is where we can finally say it was right — the same .at now genuinely grows both a Web end and a desktop end at once, with identical behavior.</p>
    <ul>
      <li><strong>Contract-layer decoupling</strong> — component declarations, state, and the event protocol depend on no renderer; swapping rendering arms is like swapping backends, freely switching between Vue / iced / ArkTS / Jetpack Compose.</li>
      <li><strong>Dual-backend parity</strong> — the same tests run through both the Vue and iced rendering paths with identical output; what you see is what ships, beyond the Web.</li>
      <li><strong>Hot reload is the soul of development</strong> — the moment you save is the moment you see, with running state intact; the desktop runs dual VM / a2r hot-reload tracks.</li>
      <li><strong>Tokenized light/dark theming</strong> — one theme declaration, one look across both ends; the example ecosystem defaults to dark, one CLI flag flips to light.</li>
    </ul>
    <template #visual>
      <div class="arch-diagram">
        <div class="arch-node arch-src"><strong>One .at</strong><span>Components · State · Event contract</span></div>
        <div class="arch-arms">
          <div class="arch-arm"><em>a2ts emit</em><div class="arch-node arch-web"><strong>Vue</strong><span>Web · the browser is the canvas</span></div></div>
          <div class="arch-arm"><em>a2r transpile</em><div class="arch-node arch-desk"><strong>iced</strong><span>Desktop · native windows</span></div></div>
        </div>
        <div class="arch-foot">One contract · one behavior · one look</div>
      </div>
      <div class="gallery-links">
        <a href="/ui/gallery/index.html" target="_self" class="gallery-link-btn">🧩 Widgets Gallery<span>46+ components, alive and interactive</span></a>
        <a href="/ui/demos/" target="_self" class="gallery-link-btn">🗂️ Demo Apps Gallery<span>28 system apps, playable online</span></a>
        <a href="/ui/charts/index.html" target="_self" class="gallery-link-btn">📊 Charts Gallery<span>Area · bar · line · donut</span></a>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">Four Flagship Apps, 100% Built in Auto</h2>
  <p class="section-desc">The desktop answers "what can Auto run?" — these four apps answer "what can Auto do?". An agent, a shell, a knowledge base, an editor: four entirely different application paradigms, all written in Auto, each with its own landing page.</p>
  <div class="flagship-list">
    <article class="flagship-item">
      <div class="flagship-text">
        <h3>🤖 AutoMusk</h3>
        <p>The agent for developing Auto apps, backed by the auto-ai architecture: Client/Daemon centrally schedules LLM compute. Driven by the AutoPlan mode, its five frontend views are generated from a single .at source.</p>
        <a class="flagship-link" href="/apps/automusk/">Landing page →</a>
      </div>
      <!-- exists: desktop shape screenshot -->
      <img src="/v05/automusk-app.png" alt="AutoMusk main interface" />
    </article>
    <article class="flagship-item reverse">
      <div class="flagship-text">
        <h3>🐚 AutoShell</h3>
        <p>The structured shell for the AI era, combining the best of AutoLang + NuShell + Fish + Warp: commands exchange typed objects instead of text streams. Backed by the auto-term terminal infrastructure, with a built-in security sandbox and 79 agent tools.</p>
        <a class="flagship-link" href="/apps/autoshell/">Landing page →</a>
      </div>
      <div class="shot-placeholder">TODO screenshot · AutoShell main interface (dark theme)</div>
    </article>
    <article class="flagship-item">
      <div class="flagship-text">
        <h3>📄 AutoDown</h3>
        <p>The Auto language knowledge base: a Markdown+YAML dialect and Jade Garden, an Obsidian-like vault, with the jade-edit editor built in. Frontend and backend logic live in one .at source, with Web and desktop shapes.</p>
        <a class="flagship-link" href="/apps/autodown/">Landing page →</a>
      </div>
      <!-- exists: desktop shape screenshot -->
      <img src="/v05/autodown-desktop.png" alt="AutoDown desktop edition" />
    </article>
    <article class="flagship-item reverse">
      <div class="flagship-text">
        <h3>📝 AutoEdit</h3>
        <p>A text editor for the Auto language with a Zed-class experience — an editor written in Auto, editing Auto. The ultimate form of dogfooding.</p>
        <!-- landing page TBD: <a class="flagship-link" href="/apps/autoedit/">Landing page →</a> -->
      </div>
      <div class="shot-placeholder">TODO screenshot · AutoEdit editor (dark theme)</div>
    </article>
  </div>
</div>

<div class="features-section">
  <h2 class="section-title">28 System App Demos</h2>
  <p class="section-desc">Whether a desktop deserves to be opened every day depends on what apps it ships with. AutoOS's answer is 28. The stars below are complete; 20+ more (calculator, clock, todo, weather, notes, chat, book reader, kanban, photo gallery...) have runnable demos at various stages of polish — all collected in the <a href="/ui/demos/" target="_self">Demo Apps Gallery</a>.</p>
  <div class="features-grid">
    <FeatureCard icon="🎵" title="Music Player" description="A complete local music player on both Web and desktop: playlist, progress, cover art — nothing missing." color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🎬" title="Video Player" description="Plays local video on Web and desktop, with progress bar, volume, and fullscreen." color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🗂️" title="File Manager" description="A complete file browser: directory tree, preview, multi-select operations — a real window on the virtual desktop." color="rgba(14, 165, 233, 0.15)" />
    <FeatureCard icon="🚀" title="Launcher" description="The app launcher — the entry point to all 28 system apps, and the foundation of the AutoOS start menu." color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🃏" title="FreeCell" description="The classic card game fully recreated, running on both AutoUI backends." color="rgba(34, 197, 94, 0.15)" />
    <FeatureCard icon="💣" title="Minesweeper" description="Minesweeper, fully playable — logic, timer, and difficulty levels." color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🧱" title="Tetris" description="Tetris: falling, rotation, line clears, scoring, smooth keyboard control." color="rgba(139, 92, 246, 0.15)" />
    <FeatureCard icon="🖥️" title="Sys Monitor" description="System monitor: KPI curves + process table, fed by real system data." color="rgba(99, 102, 241, 0.15)" />
  </div>
  <!-- TODO screenshots (one main shot per star app, dark theme is enough; no need to dual-theme):
  <div class="apps-shot-grid">
    <img src="/v05/app-music-player.png" alt="Music Player: playback and playlist" />
    <img src="/v05/app-video-player.png" alt="Video Player: local video playing" />
    <img src="/v05/app-file-manager.png" alt="File Manager: browsing and preview" />
    <img src="/v05/app-launcher.png" alt="Launcher: the app launcher" />
    <img src="/v05/app-freecell.png" alt="FreeCell" />
    <img src="/v05/app-minesweeper.png" alt="Minesweeper" />
    <img src="/v05/app-tetris.png" alt="Tetris" />
    <img src="/v05/app-sys-monitor.png" alt="Sys Monitor: KPIs and processes" />
  </div>
  -->
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="The New Playground: An Auto Lab in Your Browser"
    description="Nearly all Auto example code is browsable out of the box, with new Debug support — corpus, golden samples, and book snippets run right in your browser."
    badge="Playground"
    reverse
  >
    <p class="narrative">Beyond the language and the desktop, we also moved the lab into the browser. It used to take a repo clone to try an example; now 1280+ corpus snippets run on demand right on the page — with the AutoVM's internal state visible while they execute.</p>
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
    title="Language Progress: Full-Ecosystem Scripting + Curved Bootstrap"
    description="v0.5 moves Auto into the Rust, Python, and Web ecosystems all at once, and Auto's toolchain starts to be written in Auto — and run by Auto."
    badge="Language"
  >
    <p class="narrative">Last but not least, the language itself. v0.5's answer is full-ecosystem scripting: Rust, Python, and Web advancing in parallel — and deep in the toolchain, the bootstrap loop closed, along a curved route.</p>
    <ul>
      <li><strong>Rust ecosystem (90%)</strong> — run Auto as a scripting language for Rust: the AutoVM can call over 90% of Rust code, and the same source transpiles to native Rust via a2r — dynamic development, static release, a drop-in replacement for today's Rust workflow.</li>
      <li><strong>Python ecosystem (66%)</strong> — call Python scripts directly (e.g. PyTorch, the AI environment ready to use), or translate Auto into Python.</li>
      <li><strong>Vue / TS / JS ecosystem (80%)</strong> — can recreate most websites built with Vue; another way to write frontend.</li>
      <li><strong>Curved bootstrap</strong> — host side (avm, a2r) × bootstrap side (aavm, aa2r): since Auto doesn't yet have a compiler backend that emits binaries, bootstrapping takes a curved route — the VM and transpiler written in Auto are transpiled to Rust by <strong>aa2r</strong> and compiled into binaries, then run the VM and transpiler written in Auto.</li>
    </ul>
    <template #visual>
      <div class="matrix">
        <div class="matrix-head">Curved bootstrap: [avm, a2r] × [aavm, aa2r] × aa2r</div>
        <div class="matrix-grid">
          <div class="matrix-cell host"><strong>avm</strong><span>Rust host interpreter</span></div>
          <div class="matrix-cell host"><strong>a2r</strong><span>Transpiled to native Rust</span></div>
          <div class="matrix-cell boot"><strong>aavm</strong><span>VM written in Auto</span></div>
          <div class="matrix-cell boot"><strong>aa2r</strong><span>Transpiler written in Auto</span></div>
        </div>
        <div class="matrix-foot">One .at corpus, four execution paths, identical output; the last mile of bootstrap borrows the Rust compiler — transpiled by aa2r, compiled to binary</div>
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

.philosophy-list li.sub {
  padding-left: 1.45rem;
  font-size: 0.8rem;
  line-height: 1.6;
  color: hsl(var(--muted-foreground) / 0.85);
}

.philosophy-list li.sub::before {
  left: 0.25rem;
  top: 0.5rem;
  width: 4px;
  height: 4px;
  background: transparent;
  border: 1px solid color-mix(in srgb, var(--ph, #6366f1) 55%, transparent);
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

/* AutoUI architecture diagram */
.arch-diagram {
  width: 100%;
  max-width: 440px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
}

.arch-node {
  width: 100%;
  padding: 0.9rem 1rem;
  border-radius: var(--radius);
  text-align: center;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
}

.arch-node strong {
  display: block;
  font-family: 'JetBrains Mono', monospace;
  font-size: 1.05rem;
  color: hsl(var(--foreground));
}

.arch-node span {
  font-size: 0.78rem;
  color: hsl(var(--muted-foreground));
}

.arch-src {
  border-color: rgba(99, 102, 241, 0.4);
  background: rgba(99, 102, 241, 0.08);
}

.arch-arms {
  width: 100%;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
}

.arch-arm {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.4rem;
}

.arch-arm em {
  font-style: normal;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.72rem;
  color: hsl(var(--muted-foreground));
}

.arch-web {
  border-color: rgba(20, 184, 166, 0.4);
  background: rgba(20, 184, 166, 0.08);
}

.arch-desk {
  border-color: rgba(168, 85, 247, 0.4);
  background: rgba(168, 85, 247, 0.08);
}

.arch-foot {
  font-size: 0.82rem;
  font-weight: 600;
  color: #6366f1;
  text-align: center;
}

/* Gallery link buttons */
.gallery-links {
  width: 100%;
  max-width: 440px;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-top: 0.75rem;
}

.gallery-link-btn {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.7rem 1rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  font-size: 0.9rem;
  font-weight: 600;
  color: hsl(var(--foreground));
  transition: transform 0.15s ease, border-color 0.15s ease;
}

.gallery-link-btn:hover {
  transform: translateY(-2px);
  border-color: rgba(99, 102, 241, 0.5);
  text-decoration: none;
}

.gallery-link-btn span {
  font-size: 0.75rem;
  font-weight: 400;
  color: hsl(var(--muted-foreground));
  text-align: right;
}

/* 28-apps screenshot grid (placeholder until shots land) */
.apps-shot-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 0.75rem;
  margin-top: 1rem;
}

.apps-shot-grid img,
.shot-pair-wide img {
  width: 100%;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
}

.shot-pair-wide {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  margin-top: 1rem;
}

/* Narrative lead-ins inside showcase sections */
.narrative {
  margin: 0 0 0.25rem;
  padding-left: 0.9rem;
  border-left: 2px solid;
  border-image: linear-gradient(180deg, var(--page-accent-1, #6366f1), var(--page-accent-2, #a855f7)) 1;
  font-size: 0.95rem;
  line-height: 1.8;
  color: hsl(var(--foreground) / 0.92);
}

/* Journey narrative + timeline */
.journey {
  max-width: 780px;
  margin: 0 auto;
}

.journey-lead {
  margin: 0 0 1.1rem;
  font-size: 1.06rem;
  line-height: 1.95;
  color: hsl(var(--foreground));
}

.journey-lead em {
  font-style: normal;
  font-weight: 600;
  background: linear-gradient(135deg, var(--page-accent-1, #6366f1), var(--page-accent-2, #a855f7));
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}

.timeline {
  position: relative;
  margin-top: 2rem;
  padding-left: 1.4rem;
  display: flex;
  flex-direction: column;
  gap: 1.4rem;
}

.timeline::before {
  content: '';
  position: absolute;
  left: 0;
  top: 6px;
  bottom: 6px;
  width: 2px;
  background: linear-gradient(180deg, var(--page-accent-1, #6366f1), var(--page-accent-2, #a855f7));
  opacity: 0.35;
  border-radius: 2px;
}

.timeline-item {
  position: relative;
}

.timeline-item::before {
  content: '';
  position: absolute;
  left: -1.4rem;
  top: 0.45rem;
  width: 10px;
  height: 10px;
  margin-left: -4px;
  border-radius: 50%;
  background: hsl(var(--background));
  border: 2px solid var(--page-accent-1, #6366f1);
}

.timeline-version {
  display: inline-block;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  color: var(--page-accent-1, #6366f1);
  margin-bottom: 0.2rem;
}

.timeline-card {
  padding: 0.9rem 1.2rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border) / 0.7);
  background: hsl(var(--card));
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.timeline-card:hover {
  transform: translateX(4px);
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.12);
}

.timeline-card h3 {
  margin: 0 0 0.3rem;
  font-size: 1.05rem;
  color: hsl(var(--foreground));
}

.timeline-card p {
  margin: 0;
  font-size: 0.9rem;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
}

/* Section title decoration */
.section-title {
  position: relative;
  display: inline-block;
}

.section-title::after {
  content: '';
  display: block;
  width: 42px;
  height: 3px;
  margin: 0.55rem auto 0;
  border-radius: 2px;
  background: linear-gradient(90deg, var(--page-accent-1, #6366f1), var(--page-accent-2, #a855f7));
}

/* Soft glow behind hero */
.landing-page {
  position: relative;
}

.landing-page::before {
  content: '';
  position: absolute;
  top: -80px;
  left: 50%;
  transform: translateX(-50%);
  width: min(1100px, 95vw);
  height: 480px;
  background:
    radial-gradient(ellipse 60% 55% at 30% 40%, rgba(99, 102, 241, 0.14), transparent 70%),
    radial-gradient(ellipse 55% 50% at 72% 30%, rgba(168, 85, 247, 0.12), transparent 70%);
  pointer-events: none;
  z-index: 0;
}

.landing-page > * {
  position: relative;
  z-index: 1;
}

/* Kicker pill (standalone badge) */
.kicker {
  display: inline-block;
  padding: 0.375rem 0.875rem;
  border-radius: 9999px;
  background: color-mix(in srgb, var(--page-accent-1, #6366f1) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--page-accent-1, #6366f1) 20%, transparent);
  color: var(--page-accent-1, #6366f1);
  font-size: 0.8rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
}

/* Virtual desktop full-width section */
.vd-section .section-title {
  margin-top: 0.25rem;
}

.vd-intro {
  max-width: 780px;
  margin: 0 auto 1.5rem;
  text-align: left;
}

.vd-points {
  list-style: none;
  padding: 0;
  margin: 1rem 0 0;
  display: flex;
  flex-direction: column;
  gap: 0.7rem;
}

.vd-points li {
  position: relative;
  padding-left: 1.4rem;
  font-size: 0.92rem;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
}

.vd-points li::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0.55rem;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--page-accent-1, #6366f1), var(--page-accent-2, #a855f7));
}

.vd-points strong {
  color: hsl(var(--foreground));
}

/* Full-width screenshots */
.big-shot {
  margin: 1.75rem auto 0;
  max-width: 1080px;
}

.big-shot img,
.big-shot .shot-placeholder {
  width: 100%;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.28);
  display: block;
}

.big-shot figcaption {
  margin-top: 0.55rem;
  font-size: 0.85rem;
  color: hsl(var(--muted-foreground));
  text-align: center;
}

/* Screenshot placeholder slot */
.shot-placeholder {
  aspect-ratio: 16 / 9;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem 2rem;
  border: 1.5px dashed hsl(var(--border));
  border-radius: var(--radius);
  background: hsl(var(--card) / 0.5);
  color: hsl(var(--muted-foreground));
  font-size: 0.92rem;
  line-height: 1.7;
  text-align: center;
}

/* Flagship app rows */
.flagship-list {
  display: flex;
  flex-direction: column;
  gap: 2.25rem;
  max-width: 1080px;
  margin: 2rem auto 0;
}

.flagship-item {
  display: grid;
  grid-template-columns: 5fr 7fr;
  gap: 2rem;
  align-items: center;
}

.flagship-item.reverse {
  grid-template-columns: 7fr 5fr;
}

.flagship-item.reverse .flagship-text {
  order: 2;
}

.flagship-item.reverse img,
.flagship-item.reverse .shot-placeholder {
  order: 1;
}

.flagship-text h3 {
  margin: 0 0 0.5rem;
  font-size: 1.3rem;
  color: hsl(var(--foreground));
}

.flagship-text p {
  margin: 0 0 0.75rem;
  font-size: 0.95rem;
  line-height: 1.8;
  color: hsl(var(--muted-foreground));
}

.flagship-link {
  font-size: 0.88rem;
  font-weight: 600;
  color: var(--page-accent-1, #6366f1);
}

.flagship-item img {
  width: 100%;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.28);
  display: block;
}

@media (max-width: 768px) {
  .flagship-item,
  .flagship-item.reverse {
    grid-template-columns: 1fr;
    gap: 1rem;
  }
  .flagship-item.reverse .flagship-text {
    order: 1;
  }
  .flagship-item.reverse img,
  .flagship-item.reverse .shot-placeholder {
    order: 2;
  }
}
</style>
