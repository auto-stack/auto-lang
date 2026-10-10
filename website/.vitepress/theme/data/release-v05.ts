// PLAN-715 T-06：v0.5 发布页内容数据（EN/ZH 同构）。
// 内容全部自旧版 v05/index.md 按映射表（docs/reports/p715-website-ui-baseline.md §6）
// 迁移：理念长文进 details、统计后移至历程、TODO 开始菜单占位换 desktop-launcher.png、
// PLAN-756：四主应用与待拍槽更新；历史计数和来源保留，运行条件按实际范围说明。
// 实图尺寸实测（2560×1600 等）用于宽高占位防 CLS。

import { desktopShots } from './desktop-showcase'
import { applicationCopy } from './applications'

export interface ReleaseShot {
  src: string
  label: string
  alt: string
  caption: string
  width?: number
  height?: number
}

export interface ReleaseHighlight {
  icon: string
  name: string
  nameSub: string
  tagline: string
  example: string
  /** 完整论述（details 展开；旧页理念卡的子论点逐条保全） */
  detailsTitle: string
  details: string[]
  proof: string
  color: string
  color2: string
}

export interface ReleaseFlagship {
  name: string
  emoji: string
  desc: string
  href?: string
  linkLabel?: string
  image?: ReleaseShot
  /** AutoShell 保留原生主图；新候选场景用文字待拍槽。 */
  captureId?: string
  kind: 'image' | 'autoshell' | 'none' | 'pending'
}

export interface ReleaseCard {
  icon: string
  title: string
  description: string
}


function flagshipItems(zh: boolean): ReleaseFlagship[] {
  const emoji: Record<string, string> = { autoedit: '📝', autoshell: '🐚', automusk: '🤖', jadeedit: '📄' }
  const captures: Record<string, string> = { autoedit: 'SHOT-01', automusk: 'SHOT-04', jadeedit: 'SHOT-07' }
  return applicationCopy(zh).apps.map(app => ({
    name: app.name, emoji: emoji[app.key], desc: app.summary,
    href: (zh ? '/zh' : '') + '/apps/' + app.key + '/',
    linkLabel: zh ? '了解这个应用 →' : 'Explore this application →',
    captureId: captures[app.key], kind: app.key === 'autoshell' ? 'autoshell' : 'pending',
  }))
}

const en = {
  hero: {
    badge: 'v0.5 milestone',
    titlePre: 'Auto',
    title: ': The v0.5 Milestone',
    description: 'Dynamic dev with static shipping, decoupled frontends, and Language as OS — three ideas have grown into a platform. This time, Auto starts to become itself.',
    primaryText: 'Read the Release Notes',
    primaryLink: '/docs/releases/v0.5',
    secondaryText: 'See the desktop ↓',
    secondaryLink: '#desktop',
    status: 'This page introduces the v0.5 milestone. Release artifacts and download links await candidate freeze; dated captures remain historical evidence.',
    heroShot: {
      src: '/desktop-showcase/02-desktop-dark.png',
      label: 'Desktop',
      alt: 'AutoOS dark desktop with wallpaper, icons and widgets, without expanded app windows',
      caption: 'Start at the desktop — wallpaper, app icons and taskbar, with widgets from apps that have been started and minimized. Actual capture, 2026-10-01.',
      width: 2560,
      height: 1600,
    } as ReleaseShot,
  },

  highlightsLead: {
    title: 'Three Ideas, One Platform',
    desc: 'Dynamic dev, decoupled frontends, Language as OS — every section that follows is evidence.',
  },

  highlights: [
    {
      icon: '🌓',
      name: 'Dynamic Dev, Static Ship',
      nameSub: 'ONE SOURCE · EVERY ECOSYSTEM',
      tagline: 'A scripting language while you build, a systems language when you ship — the same source, in every ecosystem.',
      example: 'auto run (interpreted) ↔ a2r (native Rust)',
      detailsTitle: 'Full story: dynamic-static pairing across ecosystems',
      details: [
        'Most languages make you choose: scripting gives instant iteration but leaves performance at the door; systems languages deliver native speed but tax every idea with a compile cycle. Auto refuses to choose.',
        'In development, the AutoVM interprets your code — instant startup, hot reload, with REPL and LSP at your side.',
        'At release, a2r transpiles supported sources to Rust. The C generator has its own scope; complete MCU delivery remains a longer-term direction.',
        'Not just Rust — this dynamic-static pairing is the shared design origin of every ecosystem across Auto\'s map.',
        'A dynamic-static pairing for every ecosystem — UI development (VM/iced hot-reload preview ↔ transpiled release), MCU and Godot dynamic/static pairings still being explored in stages, scientific computing (use.py calls PyTorch directly ↔ a2r ships the service). Before entering any ecosystem, two questions must be answered: how does it run dynamically, and how does it ship statically?',
        'Hot reload is the soul of the dynamic side — the value of dynamic mode is more than fast startup: edit without restarting, keep your running state, and see exactly what you just changed. AutoUI desktop development supports hot reload; state preservation depends on the path and the change.',
        'Consistency guarded by machines — the parity harness runs every test through AutoVM, transpiled Rust, and native Rust, requiring identical output; 20+ replicated third-party Rust libraries serve as standing regression corpus.',
      ],
      proof: 'dynamic-static across ecosystems · three-backend parity',
      color: '#6366f1',
      color2: '#38bdf8',
    },
    {
      icon: '🔌',
      name: 'Decoupled at Every Layer',
      nameSub: 'FRONTEND EXPRESSES · BACKEND DELIVERS',
      tagline: 'One architectural law from the UI down to the OS kernel — both ends of the seam stay replaceable.',
      example: 'UI↔renderer / app↔compositor / agent↔daemon / shell↔kernel',
      detailsTitle: 'Full story: the seam pattern at every layer',
      details: [
        'Decoupling isn\'t a UI trick — it\'s the meta-pattern Auto applies at every layer.',
        'Outermost is AutoUI: its contract is independent of any host framework, and the same .at switches rendering arms between Vue, iced, ArkTS, and Jetpack Compose.',
        'Moving inward, the pattern repeats — all the way up to the operating system itself, where shell and kernel are just another front and back.',
        'The current virtual desktop uses the host window system, services, kernel, and drivers. Independent Linux or OpenHarmony integration requires additional platform work; it is a future operating form.',
        'One pattern, recurring at every layer — AutoUI ↔ rendering engines (Vue / iced / ArkTS / Jetpack); the AutoOS app architecture: auto-ui ↔ auto-compositor (RenderQueue, lock-free over shared memory); AutoAI: agents and AI-apps ↔ ai-daemon (LLM resources scheduled centrally).',
        'Shared definitions connect applications and services. AutoMusk combines Auto UI sources, generated code, and Rust infrastructure; its workflow remains distinct from the model daemon.',
      ],
      proof: 'frontend · seam · backend',
      color: '#14b8a6',
      color2: '#6366f1',
    },
    {
      icon: '🖥️',
      name: 'Language as OS',
      nameSub: 'LAOS',
      tagline: 'Use Auto to organize system components, adapt to ecosystems, and develop consistent public interfaces and experiences.',
      example: 'desktop · apps · configuration · shell · services · platform adapters',
      detailsTitle: 'Implementation idea, current form, and longer-term direction',
      details: [
        'Language as OS (LaOS) is the implementation idea: use Auto to organize UI, composition, desktop, applications, configuration, launch, monitoring, shells, and AI services.',
        'The current runtime and underlying facilities rely substantially on Rust and existing ecosystems. Kernels, drivers, and host services are distinct from the language runtime.',
        'OS over OS is the current product form: a virtual desktop alongside the host desktop, with its own applications and platform-specific host integration.',
        'An independent system using Linux or OpenHarmony is a future direction. An AutoOS kernel and a self-implemented native language foundation are longer-term research, not current deliveries.',
        'Consistent commands, UI, services, notifications, launch, and communication across ecosystems are incremental goals, not a claim of complete platform compatibility.',
        'AI + Lang + OS is the longer-term human-centered direction for knowledge, work, and everyday life: shared AI, language, and OS/application facilities carry tasks and outcomes.',
      ],
      proof: 'LaOS → OS over OS → AI + Lang + OS',
      color: '#ec4899',
      color2: '#a855f7',
    },
  ] as ReleaseHighlight[],

  desktop: {
    navLabel: 'Desktop',
    kicker: 'AutoOS · Headline',
    title: 'The AutoOS Virtual Desktop',
    desc: 'A virtual compositor inside a single OS window fits AutoUI apps into a cross-platform desktop — available on Windows today, with Linux desktop environments and HarmonyOS on the roadmap.',
    narrative: 'This time, the first thing we want to show you isn\'t a language feature — it\'s a desktop you can use. The window manager itself is an AutoUI app: virtual-window behavior is implemented in Auto above the host window; dragging, focus, and the taskbar are all ordinary Auto code. Both light and dark themes are in place, and apps are progressively integrated, with varying maturity.',
    points: [
      'WM-as-App — the chrome, dragging, focus, and taskbar of virtual windows are all written as ordinary AutoUI apps; a shared contract supports consistency across renderers.',
      'Light & dark views — the first two captures compare the desktop themes; the later views show Launcher, games and an editor workspace.',
      'Application integration — the captures show Launcher, three games, AutoEdit, Todo, and Calendar. Integration and maturity vary by application.',
      'Shared settings — auto-os-config organizes its UI in Auto and derives forms from supported data shapes. Modules can require additional rules and integration.',
      'AutoTerm terminal infrastructure — PTY + an alacritty emulation core, the terminal foundation of AutoOS.',
    ],
    galleryLabel: 'Desktop views',
    shots: desktopShots(false),
  },

  autoui: {
    navLabel: 'AutoUI',
    badge: 'AutoUI · Headline',
    title: 'AutoUI: One Contract, Two Renderers',
    desc: 'Dual Web + desktop support: the UI contract is independent of any host framework, and the same .at source generates both the Vue (Web) and iced (desktop) ends.',
    narrative: 'The desktop runs because of an earlier decision: pulling the UI contract out of the renderer. We worked on this architecture for two years, and v0.5 is where we can finally say it was right — the same .at now genuinely grows both a Web end and a desktop end at once, with behavior checked by corpus tests.',
    points: [
      'Contract-layer decoupling — component declarations, state, and the event protocol depend on no renderer; swapping rendering arms is like swapping backends, using Vue / iced as the main paths, with ArkTS / Jetpack Compose at feasibility-demo maturity.',
      'Cross-renderer verification — Vue and iced have component, event and layout checks; verified corpus protects consistency while known differences continue to be addressed.',
      'Hot reload is the soul of development — desktop development supports hot reload; state preservation depends on the path and the change.',
      'Tokenized light/dark theming — one theme declaration, one look across both ends; the example ecosystem defaults to dark, one CLI flag flips to light.',
    ],
    compareTitle: 'One example, two ends',
    compareDesc: 'The matching Vue and native desktop views will use one kanban fixture and source revision. These capture slots await candidate verification. The example is separate from Musk Kanban; images alone do not establish behavioral parity.',
    arch: {
      src: 'One .at',
      srcSub: 'Components · State · Event contract',
      arms: [
        { emit: 'a2ts emit', name: 'Vue', sub: 'Web · the browser is the canvas' },
        { emit: 'a2r transpile', name: 'iced', sub: 'Desktop · native windows' },
      ],
      foot: 'One contract · one behavior · one look',
    },
    galleryLinks: [
      { href: '/ui/gallery/index.html', title: '🧩 Widgets Gallery', sub: '46+ components, alive and interactive' },
      { href: '/apps#system-apps', title: '🗂️ System apps and demos', sub: '28 introductions with captures and per-app conditions' },
      { href: '/ui/charts/index.html', title: '📊 Charts Gallery', sub: 'Area · bar · line · donut' },
    ],
  },

  flagship: {
    navLabel: 'Apps',
    title: 'Four Main Applications',
    desc: 'AutoEdit for code and text, AutoShell for commands and scripts, AutoMusk for AI-assisted development, and JadeEdit for documents and knowledge. Features, prerequisites and readiness are described individually.',
    items: flagshipItems(false),
  },

  systemApps: {
    navLabel: 'System apps',
    title: '28 System App and Demo Introductions',
    desc: 'The directory groups 28 introductions with real captures, typical actions and runtime conditions. This is an introduction set, not a release-app count or universal online-playability claim. The older gallery has a different sample set; broader Web desktop and application experiences are planned for v0.5.1.',
    galleryHref: '/apps#system-apps',
    galleryLabel: 'Browse the application introductions',
    cards: [
      { icon: '🎵', title: 'Music Player', description: 'Web and desktop music-player forms, with playlists, progress and cover art; real media interactions continue to improve.' },
      { icon: '🎬', title: 'Video Player', description: 'Web and desktop video-player forms with real media services; desktop controls and hit testing continue to improve.' },
      { icon: '🗂️', title: 'File Manager', description: 'A file-browser demo with directory, preview and selection interfaces; data and supported operations are described individually.' },
      { icon: '🚀', title: 'Launcher', description: 'The application launcher and start-menu foundation; available entries depend on the desktop build.' },
      { icon: '🃏', title: 'FreeCell', description: 'A card-game demo; its introduction describes the supported controls and runtime path.' },
      { icon: '💣', title: 'Minesweeper', description: 'A Minesweeper demo showing the board, flags, timer and difficulty selection.' },
      { icon: '🧱', title: 'Tetris', description: 'Tetris: falling, rotation, line clears, scoring, smooth keyboard control.' },
      { icon: '🖥️', title: 'Sys Monitor', description: 'System-monitor charts and a process table; distinguish sample data from a connected system service.' },
    ] as ReleaseCard[],
  },

  playground: {
    navLabel: 'Playground',
    badge: 'Playground',
    title: 'The New Playground: An Auto Lab in Your Browser',
    desc: 'Read and edit corpus notes, golden samples and book snippets in the browser. Running, transpiling and debugging require a configured Playground backend.',
    narrative: 'The Playground brings corpus browsing to the website. Reading and editing need no backend; connect a local or co-deployed service to run supported examples. Debugging coverage depends on the execution path.',
    points: [
      'Corpus snapshot (2026-09-07) — 460 VM golden samples, 158 AAVM files, 634 book fences and 28 demo examples; the current browser lists the generated collection.',
      'Debug support — pair it with a local backend to watch the AutoVM execute, no longer just a black box.',
      'Run / transpile with a backend — inspect execution and generated Rust/Python for supported examples; one result is not itself a parity test.',
    ],
    stats: [
      { value: '460', label: 'VM Golden Samples', description: 'Corpus as tests.', color: '#6366f1' },
      { value: '634', label: 'Book Fences', description: 'Eight books with browsable snippets; execution needs a backend.', color: '#8b5cf6' },
      { value: 'Debug', label: 'Debug Support', description: 'Watch it execute.', color: '#14b8a6' },
    ],
  },

  language: {
    navLabel: 'Language',
    badge: 'Language',
    title: 'Language Progress: Full-Ecosystem Scripting + Curved Bootstrap',
    desc: 'v0.5 moves Auto into the Rust, Python, and Web ecosystems all at once, and Auto\'s toolchain starts to be written in Auto — and run by Auto.',
    narrative: 'Last but not least, the language itself. v0.5\'s answer is full-ecosystem scripting: Rust, Python, and Web advancing in parallel — and deep in the toolchain, the bootstrap loop closed, along a curved route.',
    points: [
      'Rust ecosystem — AutoVM calls Rust through natives and dep/use.rs shims, with a2r as the static release path. Layout probes, concrete generic instances and real-library parity provide evidence; compatibility depends on interfaces and signatures rather than an ecosystem-wide percentage.',
      'Python ecosystem — use.py calls CPython and a2py emits Python. The .as script mode adds bridge syntax, exceptions and context managers; PyTorch inference, training and Module/Dataset callbacks have three-way parity tests.',
      'Vue / TS / JS ecosystem — Vue is the main Web renderer, supporting state, events and full-stack APIs; component support follows concrete examples and verified coverage.',
      'Curved bootstrap — host side (avm, a2r) × bootstrap side (aavm, aa2r): since Auto doesn\'t yet have a compiler backend that emits binaries, bootstrapping takes a curved route — the VM and transpiler written in Auto are transpiled to Rust by aa2r and compiled into binaries, then run the VM and transpiler written in Auto.',
    ],
    matrix: {
      head: 'Curved bootstrap: [avm, a2r] × [aavm, aa2r] × aa2r',
      cells: [
        { name: 'avm', sub: 'Rust host interpreter', kind: 'host' },
        { name: 'a2r', sub: 'Transpiled to native Rust', kind: 'host' },
        { name: 'aavm', sub: 'VM written in Auto', kind: 'boot' },
        { name: 'aa2r', sub: 'Transpiler written in Auto', kind: 'boot' },
      ],
      foot: 'One .at corpus, four execution paths, identical output; the last mile of bootstrap borrows the Rust compiler — transpiled by aa2r, compiled to binary',
    },
  },

  toolchain: {
    navLabel: 'Toolchain',
    title: 'New: inspectable, maintainable applications',
    desc: 'Beyond the desktop, these toolchain changes are part of the v0.5 milestone.',
    cards: [
      { icon: '🔗', title: 'Real Rust libraries', description: 'dep/use.rs, layout probes and concrete generic instances, checked against real dependencies through three-way parity.' },
      { icon: '🐍', title: 'Python scripts and PyTorch', description: '.as lowering, exceptions and with-as, with inference, training and Module/Dataset callback corpus.' },
      { icon: '🔎', title: 'Structured UI inspection', description: 'Live MCP trees, layout and screenshots, plus Select Anything rectangle export to Auto/JSON/Atom.' },
      { icon: '📝', title: 'Shared editor foundation', description: 'Rope, COW snapshots, edit deltas and text/directory diff; full-document materialization remains a large-file limit.' },
      { icon: '⚙️', title: 'Backend and cache', description: 'Axum/Hyper HTTP transport, segmented UI-handler waiting and automatic UI-cache invalidation on generator updates.' },
    ] as ReleaseCard[],
    notesLabel: 'Read the full notes and known limits →',
    notesHref: '/docs/releases/v0.5',
  },

  philosophy: {
    navLabel: 'Philosophy & journey',
    title: 'The Ideas Behind v0.5',
    desc: 'Each philosophy in one line above; the full arguments are expandable below — no claim was trimmed in this reorganization.',
    journeyTitle: 'Five Months, Three Steps',
    journeyDesc: 'From v0.3 to v0.5 (Apr — Sep 2026), each release took on a new question to answer.',
    leads: [
      'v0.3 answered "is it a language?"; v0.4 answered "can it run real things?". With v0.5, the question is the hardest one yet: "can it become something you open every day?"',
      'To answer it, the repository reached 578K lines of Rust and 135K lines of Auto (September 7 snapshot, including tests and corpus) — but the line counts are just footnotes. The real change: Auto no longer lives only in compilers and test corpora. It has grown an interface, a desktop, and a whole app ecosystem. Here is the report, in order.',
    ],
    stats: [
      { value: '100 Billion', label: 'AI R&D Tokens', description: 'Platform-scale engineering written with deep AI involvement.', color: '#6366f1' },
      { value: '5,700+', label: 'Commits', description: 'September 7, 2026 snapshot: 5,711 commits since v0.3.', color: '#8b5cf6' },
      { value: '578K+', label: 'Lines of Rust', description: 'Compiler, AutoVM, the iced desktop shell, and system services.', color: '#14b8a6' },
      { value: '135K+', label: 'Lines of Auto', description: 'Bootstrap libraries, apps, and corpus — the platform\'s own language is growing.', color: '#ec4899' },
    ],
    statsNote: 'Scale figures are a September 7, 2026 repository snapshot including tests and corpus; token usage is author-provided. September 30 audit: 8,033 commits from v0.3 to 14e444f02; no frozen v0.5 tag yet.',
    timeline: [
      { version: 'v0.3', title: 'The language kernel stood up', text: 'Type system, ownership and borrowing, pattern matching — the foundation of a language was laid.' },
      { version: 'v0.4', title: 'Runtime and transpilers', text: 'AutoVM and a2r expanded their supported paths, while AI-agent infrastructure started supporting real application workflows.' },
      { version: 'v0.5', title: 'Desktop and app ecosystem', text: 'The AutoUI dual-backend architecture matured, the AutoOS virtual desktop became usable, and Auto apps and Rust services formed a desktop ecosystem.' },
    ],
  },

  roadmap: {
    navLabel: 'What\'s next',
    title: 'Looking Ahead to v0.6',
    desc: 'Near-term work and longer-term directions. Independent OS forms and knowledge collaboration have no fixed release date.',
    cards: [
      { icon: '🖥️', title: 'AutoOS', description: 'Improve the virtual desktop and host integration; explore independent systems using existing kernels. An AutoOS kernel remains longer-term research, without a delivery date.' },
      { icon: '📱', title: 'AutoUI', description: 'Build on Harmony and Jetpack Compose feasibility demos; complete Android and iOS delivery still needs further work and validation.' },
      { icon: '🤖', title: 'ROS2 Ecosystem', description: 'Robotics node development on Rust/Python, plugging into DORA-RS and beyond.' },
      { icon: '🎮', title: 'Godot Ecosystem', description: 'Fuller GDScript support and native Scene/Node capabilities, fitting into the Godot development workflow.' },
      { icon: '🔩', title: 'MCU Ecosystem', description: 'Embedded: Auto transpiles to C, and the AutoMan build system takes Auto into the microcontroller world.' },
      { icon: '🧠', title: 'AutoAI', description: 'Develop shared model services and Agent work tools. Cross-application knowledge collaboration is a longer-term direction, built on clearer data and interaction contracts.' },
    ] as ReleaseCard[],
  },

  cta: {
    title: 'Try Auto Today',
    desc: 'Read an example, then prepare the toolchain with the run and build guide. Browsing needs no backend; execution and debugging require a service.',
    primaryText: 'Get Started',
    primaryLink: '/docs/language/overview#running-and-building',
    secondaryText: 'Open Playground',
    secondaryLink: '/playground',
  },
}

const zh = {
  hero: {
    badge: 'v0.5 里程碑',
    titlePre: 'Auto',
    title: '：v0.5 里程碑',
    description: '动态开发、静态发布；前端解耦、语言即 OS —— 三个理念长成了一个平台。这一次，Auto 开始成为它自己。',
    primaryText: '阅读发布说明',
    primaryLink: '/zh/docs/releases/v0.5',
    secondaryText: '查看桌面实景 ↓',
    secondaryLink: '#desktop',
    status: '本页介绍 v0.5 里程碑。发行工件与下载入口待候选冻结后确认；已有截图按标注日期保留。',
    heroShot: {
      src: '/desktop-showcase/02-desktop-dark.png',
      label: '桌面',
      alt: 'AutoOS 深色桌面：壁纸、图标与小组件，没有展开的应用窗口',
      caption: '从桌面开始 —— 壁纸、应用图标与任务栏；小组件来自已启动并最小化的应用。实际截图，2026-10-01。',
      width: 2560,
      height: 1600,
    } as ReleaseShot,
  },

  highlightsLead: {
    title: '三个理念，一个平台',
    desc: '动态开发、层层解耦、语言即 OS —— 后面每一节都是证据。',
  },

  highlights: [
    {
      icon: '🌓',
      name: '动态开发，静态发布',
      nameSub: 'ONE SOURCE · EVERY ECOSYSTEM',
      tagline: '开发时是脚本语言，发布时是系统语言 —— 同一份源码，进入每一个生态。',
      example: 'auto run（解释执行）↔ a2r（原生 Rust）',
      detailsTitle: '完整论述：跨生态的动静配对',
      details: [
        '大多数语言让你二选一：脚本带来即时迭代，却把性能挡在门外；系统语言给出原生速度，却让每个想法都付一次编译税。Auto 拒绝选择。',
        '开发态，AutoVM 解释执行你的代码 —— 即时启动、热重载，REPL 与 LSP 随侍在侧。',
        '发布态，a2r 将受支持的源码转译为 Rust。C 生成器有独立覆盖范围，完整 MCU 交付仍属后续方向。',
        '不止 Rust —— 这一动一静的配对，是 Auto 版图上所有生态共同的设计原点。',
        '每个生态都有一动静配对 —— UI 开发（VM/iced 热重载预览 ↔ 转译发布）、MCU 与 Godot 的动静配对在分阶段探索、科学计算（use.py 直接调 PyTorch ↔ a2r 发布服务）。进入任何生态前都要回答两个问题：动态怎么跑，静态怎么发？',
        '热重载是动态侧的灵魂 —— 动态模式的价值不只是启动快：改了不用重启、运行状态还在、所见即所改。AutoUI 桌面开发支持热重载；状态能否保留取决于路径与改动。',
        '一致性由机器守护 —— parity 体系让每个测试都跑过 AutoVM、转译 Rust、原生 Rust 三条路并要求输出一致；20+ 复刻的第三方 Rust 库是常备回归语料。',
      ],
      proof: '跨生态动静配对 · 三后端 parity',
      color: '#6366f1',
      color2: '#38bdf8',
    },
    {
      icon: '🔌',
      name: '层层解耦',
      nameSub: 'FRONTEND EXPRESSES · BACKEND DELIVERS',
      tagline: '从 UI 直到 OS 内核的一条架构定律 —— 接缝两端始终可替换。',
      example: 'UI↔渲染器 / 应用↔合成器 / Agent↔Daemon / Shell↔内核',
      detailsTitle: '完整论述：每一层的接缝模式',
      details: [
        '解耦不是 UI 的花招 —— 它是 Auto 施加在每一层的元模式。',
        '最外层是 AutoUI：它的契约不依赖任何宿主框架，同一份 .at 可在 Vue、iced、ArkTS、Jetpack Compose 之间切换渲染臂。',
        '向内走，模式逐层重复 —— 直到操作系统本身：shell 与内核也不过是另一对前与后。',
        '当前虚拟桌面通过宿主窗口系统和服务使用内核与驱动。独立 Linux 或 OpenHarmony 集成仍需要平台工程，是未来运行形态。',
        '一个模式，逐层复现 —— AutoUI ↔ 渲染引擎（Vue / iced / ArkTS / Jetpack）；AutoOS 应用架构：auto-ui ↔ auto-compositor（RenderQueue，共享内存无锁）；AutoAI：Agent 与 AI 应用 ↔ ai-daemon（LLM 资源集中调度）。',
        '共享定义连接应用与服务。AutoMusk 结合 Auto UI 源码、生成代码与 Rust 基础设施；应用工作流与模型 Daemon 的职责仍然分开。',
      ],
      proof: '前端 · 接缝 · 后端',
      color: '#14b8a6',
      color2: '#6366f1',
    },
    {
      icon: '🖥️',
      name: '语言即 OS',
      nameSub: 'LAOS',
      tagline: '用 Auto 组织系统部件，适配不同生态，并逐步统一对外接口与体验。',
      example: '桌面 · 应用 · 配置 · Shell · 服务 · 平台适配',
      detailsTitle: '实现理念、当前形态与长期方向',
      details: [
        'Language as OS（LaOS）是实现理念：用 Auto 组织 UI、合成器、桌面、应用、配置、启动器、监控、Shell 与 AI 服务。',
        '当前语言运行时与底层设施主要建立在 Rust 和既有生态上；内核、驱动和宿主服务并不等同于语言运行时。',
        'OS over OS 是当前产品形态：虚拟桌面与宿主桌面并行，自带应用在其中运行，宿主互操作按平台推进。',
        '复用 Linux 内核或与 OpenHarmony 结合形成独立系统，是后续方向；自有内核和语言底座自实现 native 属于更长期研究，目前尚未实现。',
        '跨生态的命令、UI、服务、通知、启动与通信一致性是逐项推进的目标，不是所有平台已完成兼容的声明。',
        'AI + Lang + OS 是长期面向人的知识管理、工作生活环境：共享 AI 能力、语言底座与 OS/应用共同承载任务和成果。',
      ],
      proof: 'LaOS → OS over OS → AI + Lang + OS',
      color: '#ec4899',
      color2: '#a855f7',
    },
  ] as ReleaseHighlight[],

  desktop: {
    navLabel: '桌面',
    kicker: 'AutoOS · 头牌',
    title: 'AutoOS 虚拟桌面',
    desc: '单一 OS 窗口内的虚拟合成器，让 AutoUI 应用装进跨平台桌面 —— Windows 上今日可用，Linux 桌面环境与鸿蒙在路线图上。',
    narrative: '这一次，我们想给你看的第一样东西不是语言特性 —— 是一个能用的桌面。窗口管理器本身是一个 AutoUI 应用：虚拟窗口的行为用 Auto 实现在宿主窗口之上；拖拽、焦点与任务栏都是普通 Auto 代码。深浅两主题都已就位，应用在逐步接入，成熟度不一。',
    points: [
      'WM-as-App —— 虚拟窗口的边框、拖拽、焦点与任务栏都是普通 AutoUI 应用；共享契约支撑渲染器间的一致性。',
      '深浅主题对照 —— 前两张截图展示桌面主题，接着看 Launcher、小游戏与编辑器工作场景。',
      '应用集成 —— 下方截图展示 Launcher、三个小游戏、AutoEdit、Todo 与日期；各应用集成程度和成熟度不同。',
      '共享设置 —— auto-os-config 用 Auto 组织 UI，从支持的数据形状生成表单；模块可能仍需要额外规则和集成。',
      'AutoTerm 终端基建 —— PTY + alacritty 仿真内核，AutoOS 的终端底座。',
    ],
    galleryLabel: '桌面视图',
    shots: desktopShots(true),
  },

  autoui: {
    navLabel: 'AutoUI',
    badge: 'AutoUI · 头牌',
    title: 'AutoUI：一份契约，两种渲染',
    desc: 'Web + 桌面双端支持：UI 契约不依赖任何宿主框架，同一份 .at 源同时生成 Vue（Web）与 iced（桌面）两端。',
    narrative: '桌面能跑起来，源于一个更早的决定：把 UI 契约从渲染器里抽出来。这件事我们做了两年，v0.5 是终于能说做对了的时刻 —— 同一份 .at 真正同时长出 Web 端与桌面端，行为由语料测试约束。',
    points: [
      '契约层解耦 —— 组件声明、状态与事件协议不依赖渲染器；换渲染臂如同换后端，Vue / iced 为主路径，ArkTS / Jetpack Compose 为可行性演示成熟度。',
      '跨渲染器验证 —— Vue 与 iced 有组件、事件与布局检查；已验证语料保护一致性，已知差异在继续收敛。',
      '热重载是开发的灵魂 —— 桌面开发支持热重载；状态保留取决于路径与改动。',
      '令牌化深浅主题 —— 一份主题声明，两端同一观感；示例生态默认深色，一个 CLI 开关切到浅色。',
    ],
    compareTitle: '同一个例子，两端',
    compareDesc: '两端将使用同一看板数据与源码版本，分别拍摄 Vue/Web 与原生桌面视图；当前留下候选待拍位置。此示例与 Musk Kanban 分开，图片本身不证明逐项行为一致。',
    arch: {
      src: '一份 .at',
      srcSub: '组件 · 状态 · 事件契约',
      arms: [
        { emit: 'a2ts emit', name: 'Vue', sub: 'Web · 浏览器即画布' },
        { emit: 'a2r transpile', name: 'iced', sub: '桌面 · 原生窗口' },
      ],
      foot: '一份契约 · 一种行为 · 一个观感',
    },
    galleryLinks: [
      { href: '/ui/gallery/index.html', title: '🧩 组件画廊', sub: '46+ 组件，活的、可交互' },
      { href: '/zh/apps#system-apps', title: '🗂️ 系统应用与 Demo', sub: '28 项介绍、截图与各自运行条件' },
      { href: '/ui/charts/index.html', title: '📊 图表画廊', sub: '面积 · 柱 · 线 · 环' },
    ],
  },

  flagship: {
    navLabel: '旗舰应用',
    title: '四个主应用',
    desc: 'AutoEdit 用于代码与文本，AutoShell 用于命令与脚本，AutoMusk 用于 AI 辅助开发，JadeEdit 用于文档与知识。各应用分别说明功能、运行条件与当前进展。',
    items: flagshipItems(true),
  },

  systemApps: {
    navLabel: '系统应用',
    title: '28 项系统应用与 Demo 介绍',
    desc: '应用目录整理了 28 项介绍，提供实图、典型操作与运行条件。这是介绍集合，不是发布应用数量或全部在线可玩的承诺。旧交互画廊的示例集合不同；更完整的 Web 桌面和应用体验安排在 v0.5.1。',
    galleryHref: '/zh/apps#system-apps',
    galleryLabel: '浏览应用介绍',
    cards: [
      { icon: '🎵', title: '音乐播放器', description: 'Web 与桌面两种形态，带播放列表、进度与封面；真实媒体交互持续完善。' },
      { icon: '🎬', title: '视频播放器', description: 'Web 与桌面两种形态，接真实媒体服务；桌面端控件与命中测试持续完善。' },
      { icon: '🗂️', title: '文件管理器', description: '文件浏览示例，展示目录、预览与选择界面；数据来源和实际操作见应用介绍。' },
      { icon: '🚀', title: '启动器', description: '应用启动器与开始菜单基座；可用入口取决于实际桌面构建与接入应用。' },
      { icon: '🃏', title: 'FreeCell', description: '纸牌游戏示例；支持的操作与运行路径见应用介绍。' },
      { icon: '💣', title: '扫雷', description: '扫雷示例，展示棋盘、标记、计时与难度选择。' },
      { icon: '🧱', title: '俄罗斯方块', description: '方块下落、旋转、消行、计分，键盘操控顺滑。' },
      { icon: '🖥️', title: '系统监视器', description: '系统监视器的曲线与进程表界面；示例数据和接入系统服务后的数据分别说明。' },
    ] as ReleaseCard[],
  },

  playground: {
    navLabel: 'Playground',
    badge: 'Playground',
    title: '新 Playground：浏览器里的 Auto 实验室',
    desc: '在浏览器中阅读、编辑仓内语料、金样和书籍片段；运行、转译与调试需要连接已配置的 Playground 后端。',
    narrative: 'Playground 把语料浏览带进网站。阅读和编辑无需后端；连接本地或配套部署的服务后，再运行受支持的示例并查看结果。调试覆盖以具体路径为准。',
    points: [
      '语料快照（2026-09-07） —— 460 个 VM 金样、158 个 AAVM 文件、634 个书籍围栏、28 个 Demo 示例；当前集合以页面清单为准。',
      'Debug 支持 —— 配合本地后端观看 AutoVM 执行，不再是黑盒。',
      '连接后端运行 / 转译 —— 对受支持的示例查看执行与 Rust/Python 生成结果；一次运行结果本身不等同于 parity 验证。',
    ],
    stats: [
      { value: '460', label: 'VM 金样', description: '语料即测试。', color: '#6366f1' },
      { value: '634', label: '书籍围栏', description: '八本书的可浏览片段；执行需要后端。', color: '#8b5cf6' },
      { value: 'Debug', label: '调试支持', description: '看着它执行。', color: '#14b8a6' },
    ],
  },

  language: {
    navLabel: '语言',
    badge: '语言',
    title: '语言进展：全生态脚本 + 曲线自举',
    desc: 'v0.5 让 Auto 同时进入 Rust、Python 与 Web 生态，工具链开始用 Auto 编写 —— 并由 Auto 运行。',
    narrative: '最后是语言本身。v0.5 的答案是全生态脚本：Rust、Python、Web 并行推进 —— 工具链深处，自举环路沿一条曲线闭合。',
    points: [
      'Rust 生态 —— AutoVM 经 natives 与 dep/use.rs 垫片调用 Rust，a2r 是静态发布路径。布局探针、具体泛型实例与真实库对拍提供证据；兼容性取决于接口与签名，而非生态级百分比。',
      'Python 生态 —— use.py 调用 CPython，a2py 发射 Python。.as 脚本模式补桥接语法、异常与上下文管理器；PyTorch 推理、训练与 Module/Dataset 回调有三向 parity 测试。',
      'Vue / TS / JS 生态 —— Vue 是主力 Web 渲染器，支持状态、事件与全栈 API；组件支持跟随具体示例与已验证覆盖。',
      '曲线自举 —— 宿主侧（avm，a2r）× 自举侧（aavm，aa2r）：Auto 还没有能发射二进制的编译器后端，自举走曲线 —— Auto 写的 VM 与转译器经 aa2r 转译成 Rust、编译为二进制，再运行 Auto 写的 VM 与转译器。',
    ],
    matrix: {
      head: '曲线自举：[avm, a2r] × [aavm, aa2r] × aa2r',
      cells: [
        { name: 'avm', sub: 'Rust 宿主解释器', kind: 'host' },
        { name: 'a2r', sub: '转译为原生 Rust', kind: 'host' },
        { name: 'aavm', sub: 'Auto 写的 VM', kind: 'boot' },
        { name: 'aa2r', sub: 'Auto 写的转译器', kind: 'boot' },
      ],
      foot: '一份 .at 语料，四条执行路径，输出一致；自举最后一程借 Rust 编译器 —— aa2r 转译、编译为二进制',
    },
  },

  toolchain: {
    navLabel: '工具链',
    title: '新增：可检视、可维护的应用',
    desc: '桌面之外，这些工具链变化同属 v0.5 里程碑。',
    cards: [
      { icon: '🔗', title: '真实 Rust 库', description: 'dep/use.rs、布局探针与具体泛型实例，经三向 parity 与真实依赖对照检查。' },
      { icon: '🐍', title: 'Python 脚本与 PyTorch', description: '.as 降级、异常与 with-as，含推理、训练与 Module/Dataset 回调语料。' },
      { icon: '🔎', title: '结构化 UI 检视', description: '实时 MCP 树、布局与截图，外加 Select Anything 框选导出 Auto/JSON/Atom。' },
      { icon: '📝', title: '共享编辑器基座', description: 'Rope、COW 快照、编辑增量与文本/目录 diff；整文档物化仍是大文件边界。' },
      { icon: '⚙️', title: '后端与缓存', description: 'Axum/Hyper HTTP 传输、分段 UI 处理器等待与生成器更新时的 UI 缓存自动失效。' },
    ] as ReleaseCard[],
    notesLabel: '阅读完整发布说明与已知边界 →',
    notesHref: '/zh/docs/releases/v0.5',
  },

  philosophy: {
    navLabel: '理念与历程',
    title: 'v0.5 背后的理念',
    desc: '了解动态开发、层层解耦与 LaOS 的实现思路，并区分当前能力和长期方向。',
    journeyTitle: '五个月，三步',
    journeyDesc: '从 v0.3 到 v0.5（2026 年 4 月 — 9 月），每个版本都回答了一个新问题。',
    leads: [
      'v0.3 回答了"它是一门语言吗？"；v0.4 回答了"它能跑真东西吗？"。到了 v0.5，问题变成了最难的一个："它能成为你每天打开的东西吗？"',
      '为回答它，仓库达到 57.8 万行 Rust 与 13.5 万行 Auto（9 月 7 日快照，含测试与语料）—— 但行数只是脚注。真正的变化是：Auto 不再只活在编译器与测试语料里。它长出了界面、桌面与一整个应用生态。以下是按顺序的报告。',
    ],
    stats: [
      { value: '1000 亿', label: 'AI 研发 Token', description: '深度 AI 参与的平台级工程。', color: '#6366f1' },
      { value: '5,700+', label: '提交数', description: '2026-09-07 快照：v0.3 以来 5,711 次提交。', color: '#8b5cf6' },
      { value: '57.8 万+', label: 'Rust 行数', description: '编译器、AutoVM、iced 桌面 Shell 与系统服务。', color: '#14b8a6' },
      { value: '13.5 万+', label: 'Auto 行数', description: '自举库、应用与语料 —— 平台自己的语言在长大。', color: '#ec4899' },
    ],
    statsNote: '规模数字为 2026 年 9 月 7 日仓库快照，含测试与语料；Token 用量为作者提供。9 月 30 日审计：v0.3 至 14e444f02 共 8,033 次提交；尚无冻结的 v0.5 标签。',
    timeline: [
      { version: 'v0.3', title: '语言内核立住', text: '类型系统、所有权与借用、模式匹配 —— 一门语言的地基完成。' },
      { version: 'v0.4', title: '运行时与转译器', text: 'AutoVM 与 a2r 扩展已支持路径，AI Agent 基础设施开始支撑真实应用工作流。' },
      { version: 'v0.5', title: '桌面与应用生态', text: 'AutoUI 双后端架构成熟，AutoOS 虚拟桌面可用，Auto 应用与 Rust 服务形成桌面生态。' },
    ],
  },

  roadmap: {
    navLabel: '展望',
    title: '展望 v0.6',
    desc: '近期工作与长期方向。独立 OS 形态和知识协作没有固定交付日期。',
    cards: [
      { icon: '🖥️', title: 'AutoOS', description: '继续完善虚拟桌面与宿主集成，探索复用现有内核的独立系统。自有内核属更长期研究，不承诺交付时间。' },
      { icon: '📱', title: 'AutoUI', description: '在已有鸿蒙与 Jetpack Compose 可行性演示上推进移动端支持；Android / iOS 的完整交付仍需后续验证。' },
      { icon: '🤖', title: 'ROS2 生态', description: 'Rust/Python 上的机器人节点开发，接入 DORA-RS 等。' },
      { icon: '🎮', title: 'Godot 生态', description: '更完整的 GDScript 支持与原生 Scene/Node 能力，融入 Godot 开发工作流。' },
      { icon: '🔩', title: 'MCU 生态', description: '嵌入式：Auto 转译到 C，AutoMan 构建系统把 Auto 带进微控制器世界。' },
      { icon: '🧠', title: 'AutoAI', description: '完善共享模型服务与 Agent 工作工具；跨应用知识协作是长期方向，需要继续建立数据与交互契约。' },
    ] as ReleaseCard[],
  },

  cta: {
    title: '今天就试试 Auto',
    desc: '先阅读示例，再按运行与构建说明准备工具链。浏览 Playground 无需后端；执行与调试需要配套服务。',
    primaryText: '快速开始',
    primaryLink: '/zh/docs/language/overview#运行与构建',
    secondaryText: '打开 Playground',
    secondaryLink: '/zh/playground',
  },
}

export type ReleaseContent = typeof en

export const RELEASE_V05: Record<'en' | 'zh', ReleaseContent> = { en, zh }

/** SectionNav 数据（顺序 = 页面新顺序）。 */
export function releaseNavSections(zh: boolean) {
  const c = zh ? RELEASE_V05.zh : RELEASE_V05.en
  return [
    { id: 'highlights', label: zh ? '亮点' : 'Highlights' },
    { id: 'desktop', label: c.desktop.navLabel },
    { id: 'autoui', label: c.autoui.navLabel },
    { id: 'flagship', label: c.flagship.navLabel },
    { id: 'ecosystem', label: zh ? '系统应用 / 工具链 / Playground' : 'System / Toolchain / Playground' },
    { id: 'philosophy', label: c.philosophy.navLabel },
    { id: 'roadmap', label: c.roadmap.navLabel },
    { id: 'get-started', label: zh ? '开始使用' : 'Get started' },
  ]
}
