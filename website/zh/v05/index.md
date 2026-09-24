---
layout: home
---

<script setup>
import HomeHero from '../../.vitepress/theme/components/HomeHero.vue'
import FeatureCard from '../../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #6366f1; --page-accent-2: #a855f7">

<HomeHero
  badge="有史以来最大的一次更新"
  title="：v0.5 正式发布"
  description="动静相宜、前后解耦、Language as OS —— 三个理念长成一个平台：AutoUI 双端架构成熟、AutoOS 虚拟桌面走向可用、四大旗舰应用与 28 个系统应用全部用 Auto 写成。这一次，Auto 开始成为它自己。"
  primary-text="阅读发布说明"
  primary-link="/zh/docs/releases/v0.5"
  secondary-text="打开 Playground"
  secondary-link="/zh/playground"
  hide-code
>
  <div class="hero-stats">
    <StatCard value="1000 亿" label="AI 研发 Token" description="由 AI 深度参与编写的平台级工程。" color="#6366f1" />
    <StatCard value="5,700+" label="Commits" description="v0.3 以来 5,711 次提交，全部经双测门禁合入。" color="#8b5cf6" />
    <StatCard value="57.8 万" label="行 Rust 代码" description="编译器、AutoVM、iced 桌面端与系统服务。" color="#14b8a6" />
    <StatCard value="13.5 万" label="行 Auto 代码" description="自举库、应用与语料 —— 平台自己的语言在生长。" color="#ec4899" />
  </div>
  <p class="hero-stats-note">从 v0.3 到 v0.5（2026.04 — 2026.09），Auto 平台的量化足迹。</p>
</HomeHero>

<div class="features-section">
  <h2 class="section-title">三个设计理念</h2>
  <p class="section-desc">动静相宜、前后解耦、Language as OS —— 本页接下来的每一个段落，都是这三句话的证据。</p>
  <div class="philosophy-grid">
    <article class="philosophy-card" style="--ph: #6366f1; --ph2: #38bdf8">
      <div class="philosophy-icon">🌓</div>
      <h3 class="philosophy-name">动静相宜</h3>
      <p class="philosophy-en">Dynamic Dev · Static Ship</p>
      <p class="philosophy-tagline">开发时是脚本语言，发布时是系统语言 —— 同一份源码，全生态通用。</p>
      <ul class="philosophy-list">
        <li class="sub">多数语言逼你二选一：脚本语言秒级周转，却把性能留在门外；系统语言性能拉满，却让你用编译周期偿还每一次灵感。Auto 拒绝选择。</li>
        <li class="sub">开发态由 AutoVM 解释执行 —— 秒级启动、热重载、REPL 与 LSP 常伴左右。</li>
        <li class="sub">发布态，同一份源码经 a2r 转译为原生 Rust（或经 a2c 转译为 C）。</li>
        <li class="sub">不止 Rust 一家 —— 这套动静组合是整个版图里每个生态共用的设计原点。</li>
        <li><strong>全生态动静组合</strong> —— UI 开发（VM/iced 热重载预览 ↔ 转译发布）、MCU 嵌入式（VM 即在线模拟器 ↔ a2c 交叉编译烧录）、Godot（发射 GDScript 编辑器内热改 ↔ C/Rust 静态缝合引擎）、科学计算（use.py 直调 PyTorch ↔ a2r 发布服务）——进入任何生态，两问必答：动态态怎么跑，静态态怎么发。</li>
        <li><strong>热重载是动态态的灵魂</strong> —— 动态的含金量不止"启动快"：改代码不重启、运行状态不丢、所见即所改。AutoUI 桌面端 VM / a2r 双轨热重载，保存的瞬间就是看到的瞬间。</li>
        <li><strong>一致性由机器守护</strong> —— parity 对拍让同一测试跑遍 AutoVM、转译 Rust、原生 Rust 三条通路，输出必须全等；20+ 三方 Rust 库复刻语料常态回归。</li>
      </ul>
      <div class="philosophy-proof"><code>auto run</code>（VM 直跑）↔ <code>a2r</code>（原生 Rust）· 全生态动静矩阵 · 三后端对拍</div>
    </article>
    <article class="philosophy-card" style="--ph: #14b8a6; --ph2: #6366f1">
      <div class="philosophy-icon">🔌</div>
      <h3 class="philosophy-name">前后解耦</h3>
      <p class="philosophy-en">Decoupled at Every Layer</p>
      <p class="philosophy-tagline">前端负责表达，后端负责兑现 —— 接缝清晰，两端皆可替换。</p>
      <ul class="philosophy-list">
        <li class="sub">前后解耦不是 UI 的专利，而是 Auto 在每一层反复使用的架构元模式。</li>
        <li class="sub">最外层是 AutoUI：契约独立于宿主框架，同一份 .at 在 Vue、iced、ArkTS、Jetpack Compose 之间切换渲染臂。</li>
        <li class="sub">往里走，模式一路重现 —— 直到操作系统的顶层：外壳与内核，也是一对前后端。</li>
        <li><strong>OS 外壳 ↔ OS 内核，后端可切换</strong> —— Windows 上，前端是虚拟桌面，后端是 Windows 内核；未来的 AutoOS 发行版里，AutoOS 桌面与 Linux 内核直接耦合；后端还可以切换到 OpenHarmony。换内核，不换你的应用。</li>
        <li><strong>同一模式，层层重现</strong> —— AutoUI ↔ 渲染引擎（Vue / iced / ArkTS / Jetpack）；AutoOS 应用架构：auto-ui ↔ auto-compositor（RenderQueue 共享内存无锁通讯）；AutoAI：各类 Agent 与 AI-App ↔ ai-daemon（LLM 资源统一调度）。</li>
        <li><strong>解耦的红利：接缝两端可同栈</strong> —— AutoDown 知识库 411 个文件、3.2 万行，前端后端逻辑全 .at 单源，Web / 桌面双形态；AutoMusk 前端五视图同样单源生成，148 项对拍全等。</li>
      </ul>
      <div class="philosophy-proof">前端 · 接缝 · 后端 —— UI↔渲染器 / app↔compositor / Agent↔daemon / 外壳↔内核</div>
    </article>
    <article class="philosophy-card" style="--ph: #ec4899; --ph2: #a855f7">
      <div class="philosophy-icon">🖥️</div>
      <h3 class="philosophy-name">LAOS：语言即操作系统</h3>
      <p class="philosophy-en">Language as OS</p>
      <p class="philosophy-tagline">语言模拟 OS 架构，把每个 OS 模块做成语言化组件。</p>
      <ul class="philosophy-list">
        <li class="sub">操作系统的三大天职 —— 共享、抽象、服务 —— 正是一门语言必须回答的三问。</li>
        <li class="sub">Auto 把答案写进语言本体：Task/Msg Actor 并发模型是调度器，view/mut/move 内存三元组是内存管理，io/net/http/fs 标准库是系统调用，多端渲染臂是外设驱动。</li>
        <li class="sub">更进一步：每个 OS 模块都有两条供给路线 —— Auto 亲手实现，或经生态桥调用现成实现 —— 按目标平台自由"装机"。</li>
        <li><strong>模块语言化，双路供给</strong> —— 跑在 Windows 上：应用与 UI 转译为 Rust/iced，内核调用 Windows 现成能力；跑进 MCU：应用与 UI 可转译为 C/LVGL，内核换成 Auto 自写的 RTOS。同一门语言，按机器"装机"。</li>
        <li><strong>OS 的形状已经可见</strong> —— AutoOS 虚拟桌面正在运行，窗口管理器本身就是一个 AutoUI 应用（见下一节）；auto-os-config 让配置文件的形状自动长成设置中心。</li>
        <li><strong>AI 算力按 OS 设备调度</strong> —— Client/Daemon 架构已在 AutoAI 落地：向系统要算力，而不是每个应用自建 AI 栈。</li>
      </ul>
      <div class="philosophy-proof">调度 · 内存 · 系统调用 · 外设 —— 件件可自实现，件件可生态桥</div>
    </article>
  </div>
</div>

<div class="features-section">
  <h2 class="section-title">五个月，三个台阶</h2>
  <p class="section-desc">从 v0.3 到 v0.5（2026.04 — 2026.09），每到一个版本，Auto 就换一个问题来回答。</p>
  <div class="journey">
    <p class="journey-lead">v0.3 回答的是<em>"它是一门语言吗"</em>；v0.4 回答的是<em>"它能跑真的东西吗"</em>。到 v0.5，问题换成了最难的一个：<em>"它能不能成为你每天打开的东西？"</em></p>
    <p class="journey-lead">为了回答这个问题，过去的五个月里，我们写了 57.8 万行 Rust 和 13.5 万行 Auto —— 但行数只是脚注。真正的变化是：Auto 不再只活在编译器和测试语料里，它长出了界面、长出了桌面、长出了一整个应用生态。下面按顺序汇报。</p>
    <div class="timeline">
      <div class="timeline-item">
        <span class="timeline-version">v0.3</span>
        <div class="timeline-card">
          <h3>语言内核立住</h3>
          <p>类型系统、所有权与借用、模式匹配就位；一门语言的地基打完了。</p>
        </div>
      </div>
      <div class="timeline-item">
        <span class="timeline-version">v0.4</span>
        <div class="timeline-card">
          <h3>运行时与转译</h3>
          <p>AutoVM 全功能化，a2r 转译生产级可用，AI Agent 基础设施搭建 —— Auto 开始跑真实的应用。</p>
        </div>
      </div>
      <div class="timeline-item">
        <span class="timeline-version">v0.5</span>
        <div class="timeline-card">
          <h3>桌面与应用生态</h3>
          <p>AutoUI 双端架构成熟，AutoOS 虚拟桌面走向可用，四大旗舰与 28 个系统应用全部用 Auto 写成。</p>
        </div>
      </div>
    </div>
  </div>
</div>

<div class="features-section vd-section">
  <span class="kicker">AutoOS · 第一重点</span>
  <h2 class="section-title">AutoOS 虚拟桌面</h2>
  <p class="section-desc">单 OS 窗口内的虚拟合成器，把 AutoUI 应用装进一个跨平台桌面 —— Windows 今天可用，Linux 桌面环境与鸿蒙在路线图上。</p>
  <div class="vd-intro">
    <p class="narrative">这一次，我们想先给你看的不是语言特性，而是一块能用的桌面。窗口管理器本身是一个 AutoUI 应用 —— 我们没有调用一行系统窗口 API，拖拽、焦点、任务栏全是普通 Auto 代码。深浅两套主题已经就位，四大旗舰与 28 个系统应用，就装在这张桌面里。</p>
    <ul class="vd-points">
      <li><strong>WM-as-App</strong> —— 虚拟窗口的 chrome、拖拽、焦点、任务栏全部用 AutoUI 写成普通应用，双端一致性是构造性保证。</li>
      <li><strong>深浅色双主题</strong> —— 桌面与主要应用已完成两套主题适配，运行中一键切换（下方案例同时给出两套主题）。</li>
      <li><strong>桌面即应用容器</strong> —— 开始菜单列出全部应用：四大旗舰与 20+ 系统应用直接在桌面上开窗口运行，见下方大图。</li>
      <li><strong>统一设置中心</strong> —— auto-os-config：一个 Daemon 按 .at 文件形状自动渲染配置表单，零前端代码接入新模块。</li>
      <li><strong>AutoTerm 终端设施</strong> —— PTY + alacritty 仿真核心，AutoOS 的终端底座。</li>
    </ul>
  </div>
  <figure class="big-shot">
    <!-- 已有：深色主视图 -->
    <img src="/v05/desktop-hero.png" alt="AutoOS 虚拟桌面（深色）：任务栏、开始菜单与多窗口" />
    <figcaption>深色主题主视图 —— 任务栏、开始菜单与多窗口</figcaption>
  </figure>
  <figure class="big-shot">
    <!-- 已有：浅色同款桌面 -->
    <img src="/v05/desktop-light.png" alt="AutoOS 虚拟桌面（浅色主题）" />
    <figcaption>浅色主题同款桌面</figcaption>
  </figure>
  <figure class="big-shot">
    <div class="shot-placeholder">待截图 · 桌面上同时打开多个系统应用（建议：music-player + file-manager + 设置中心，深色）</div>
    <figcaption>桌面巡礼 Ⅰ —— 系统应用以窗口形式跑在桌面上</figcaption>
  </figure>
  <figure class="big-shot">
    <div class="shot-placeholder">待截图 · 开始菜单 + 游戏应用窗口（建议：空档接龙 / 扫雷 / 俄罗斯方块任选，可浅色）</div>
    <figcaption>桌面巡礼 Ⅱ —— 开始菜单与游戏应用</figcaption>
  </figure>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="AutoUI：一份契约，双端渲染"
    description="Web 与桌面双支持的全新架构：UI 契约独立于宿主框架，同一份 .at 单源生成 Vue（Web）与 iced（桌面）两端。"
    badge="AutoUI · 第一重点"
    reverse
  >
    <p class="narrative">桌面能跑起来，靠的是一个更早的决定：把 UI 契约从渲染器里抽出来。这个架构做了两年，v0.5 终于可以说它是对的 —— 因为同一份 .at，今天真的同时长出了 Web 和桌面两端，而且行为一致。</p>
    <ul>
      <li><strong>契约层前后解耦</strong> —— 组件声明、状态与事件协议不依赖任何渲染器；换渲染臂如同换后端，Vue / iced / ArkTS / Jetpack Compose 自由切换。</li>
      <li><strong>双端对拍</strong> —— 同一测试跑遍 Vue 与 iced 两条渲染通路，输出全等；所见即所得不止于 Web。</li>
      <li><strong>热重载是开发态的灵魂</strong> —— 保存的瞬间就是看到的瞬间，运行状态不丢；桌面端 VM / a2r 双轨热重载。</li>
      <li><strong>深浅色主题 token 化</strong> —— 一份主题声明，双端同一外观；示例生态默认深色，CLI 一行切浅色。</li>
    </ul>
    <template #visual>
      <div class="arch-diagram">
        <div class="arch-node arch-src"><strong>一份 .at</strong><span>组件 · 状态 · 事件契约</span></div>
        <div class="arch-arms">
          <div class="arch-arm"><em>a2ts 发射</em><div class="arch-node arch-web"><strong>Vue</strong><span>Web · 浏览器即画布</span></div></div>
          <div class="arch-arm"><em>a2r 转译</em><div class="arch-node arch-desk"><strong>iced</strong><span>桌面 · 原生窗口</span></div></div>
        </div>
        <div class="arch-foot">同一契约 · 同一行为 · 同一外观</div>
      </div>
      <div class="gallery-links">
        <a href="/ui/gallery/index.html" target="_self" class="gallery-link-btn">🧩 Widgets Gallery<span>46+ 组件，活的可交互</span></a>
        <a href="/ui/demos/" target="_self" class="gallery-link-btn">🗂️ Demo Apps Gallery<span>28 个系统应用，在线试玩</span></a>
        <a href="/ui/charts/index.html" target="_self" class="gallery-link-btn">📊 Charts Gallery<span>面积 · 柱状 · 折线 · 环形</span></a>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">四大旗舰应用，100% Auto 构建</h2>
  <p class="section-desc">桌面回答的是"Auto 能跑什么"，这四个应用回答的是"Auto 能做什么"。Agent、Shell、知识库、编辑器 —— 四种完全不同的应用范式，全部由 Auto 自己写成，每个都有独立落地页。</p>
  <div class="flagship-list">
    <article class="flagship-item">
      <div class="flagship-text">
        <h3>🤖 AutoMusk</h3>
        <p>开发 Auto App 的 Agent，背后是 auto-ai 架构：Client/Daemon 统一调度 LLM 算力。AutoPlan 模式驱动，前端五个视图由 .at 单源生成。</p>
        <a class="flagship-link" href="/zh/apps/automusk/">查看落地页 →</a>
      </div>
      <!-- 已有：桌面形态截图 -->
      <img src="/v05/automusk-app.png" alt="AutoMusk 主界面" />
    </article>
    <article class="flagship-item reverse">
      <div class="flagship-text">
        <h3>🐚 AutoShell</h3>
        <p>AI 时代的结构化 Shell：AutoLang + NuShell + Fish + Warp 四家之长，命令交换类型化对象而非文本流；背后是 auto-term 终端设施，内置安全沙箱与 79 个 Agent 工具。</p>
        <a class="flagship-link" href="/zh/apps/autoshell/">查看落地页 →</a>
      </div>
      <div class="shot-placeholder">待截图 · AutoShell 主界面（深色主题）</div>
    </article>
    <article class="flagship-item">
      <div class="flagship-text">
        <h3>📄 AutoDown</h3>
        <p>Auto 语言的知识库：Markdown+YAML 方言与类 Obsidian 的 Jade Garden，内置 jade-edit 编辑器。前后端逻辑全 .at 单源，Web/桌面双形态。</p>
        <a class="flagship-link" href="/zh/apps/autodown/">查看落地页 →</a>
      </div>
      <!-- 已有：桌面形态截图 -->
      <img src="/v05/autodown-desktop.png" alt="AutoDown 桌面形态" />
    </article>
    <article class="flagship-item reverse">
      <div class="flagship-text">
        <h3>📝 AutoEdit</h3>
        <p>Auto 语言的文本编辑器，类 Zed 的极速体验 —— 用 Auto 写成的编辑器，编辑 Auto。Dogfooding 的终极形态。</p>
        <!-- 落地页待补：<a class="flagship-link" href="/zh/apps/autoedit/">查看落地页 →</a> -->
      </div>
      <div class="shot-placeholder">待截图 · AutoEdit 编辑器主界面（深色主题）</div>
    </article>
  </div>
</div>

<div class="features-section">
  <h2 class="section-title">28 个系统应用 Demo</h2>
  <p class="section-desc">一个桌面值不值得每天打开，要看它自带什么应用。AutoOS 的答案是 28 个 —— 其中明星应用完整可用，其余 20+（计算器、时钟、待办、天气、记事本、聊天、读书器、看板、图库……）均有不同进度的可运行 demo，全部收录在 <a href="/ui/demos/" target="_self">Demo Apps Gallery</a>。</p>
  <div class="features-grid">
    <FeatureCard icon="🎵" title="Music Player" description="完整的本地音乐播放器，Web / 桌面双端可用：播放列表、进度、封面，一个不缺。" color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🎬" title="Video Player" description="本地视频播放器，Web / 桌面双端可跑，进度条、音量、全屏俱全。" color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🗂️" title="File Manager" description="完整的文件浏览器：目录树、预览、多选操作，跑在虚拟桌面上的一个真窗口。" color="rgba(14, 165, 233, 0.15)" />
    <FeatureCard icon="🚀" title="Launcher" description="应用启动器 —— 28 个系统应用的入口，也是 AutoOS 开始菜单的底座。" color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🃏" title="空档接龙" description="经典纸牌游戏完整复刻，AutoUI 双端运行。" color="rgba(34, 197, 94, 0.15)" />
    <FeatureCard icon="💣" title="扫雷" description="扫雷完整可玩 —— 逻辑、计时、难度分级。" color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🧱" title="俄罗斯方块" description="俄罗斯方块：下落、旋转、消行、计分，键盘操作流畅。" color="rgba(139, 92, 246, 0.15)" />
    <FeatureCard icon="🖥️" title="Sys Monitor" description="系统监视器：KPI 曲线 + 进程表，真实系统数据。" color="rgba(99, 102, 241, 0.15)" />
  </div>
  <!-- 待截图（每个明星应用一张主界面截图，深色主题即可，不必双主题）：
  <div class="apps-shot-grid">
    <img src="/v05/app-music-player.png" alt="Music Player：播放界面与播放列表" />
    <img src="/v05/app-video-player.png" alt="Video Player：本地视频播放中" />
    <img src="/v05/app-file-manager.png" alt="File Manager：目录浏览与预览" />
    <img src="/v05/app-launcher.png" alt="Launcher：应用启动器" />
    <img src="/v05/app-freecell.png" alt="空档接龙" />
    <img src="/v05/app-minesweeper.png" alt="扫雷" />
    <img src="/v05/app-tetris.png" alt="俄罗斯方块" />
    <img src="/v05/app-sys-monitor.png" alt="Sys Monitor：KPI 与进程" />
  </div>
  -->
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="新版 Playground：浏览器里的 Auto 实验台"
    description="几乎全部 Auto 示例代码开箱即览，新增 Debug 支持 —— 语料、金样、书页代码在浏览器里就近试炼。"
    badge="Playground"
    reverse
  >
    <p class="narrative">语言和桌面之外，我们把实验台也搬进了浏览器。以前想看一个示例得先 clone 仓库；现在 1280+ 段语料在网页里即点即跑，还能边执行边看 AutoVM 的内部状态。</p>
    <ul>
      <li><strong>1280+ 语料笔记</strong> —— VM 金样 460、AAVM 自举语料 158、书页围栏 634、Demo 示例 28。</li>
      <li><strong>Debug 支持</strong> —— 配合本地后端，观察 AutoVM 的执行过程，不再只是黑盒运行。</li>
      <li><strong>在线运行 / 转译</strong> —— 同一份源码，解释执行或转译为 Rust/Python 对拍。</li>
    </ul>
    <template #visual>
      <div class="stats-grid playground-stats">
        <StatCard value="460" label="VM 金样" description="语料即测试。" color="#6366f1" />
        <StatCard value="634" label="书页围栏" description="八本书边读边跑。" color="#8b5cf6" />
        <StatCard value="Debug" label="调试支持" description="执行过程可见。" color="#14b8a6" />
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="语言进展：全生态脚本化 + 曲线自举"
    description="v0.5 让 Auto 同时住进 Rust、Python 与 Web 三大生态，也让 Auto 的工具链开始由 Auto 自己编写、自己运行。"
    badge="语言"
  >
    <p class="narrative">最后汇报语言本身的进展。v0.5 的答案是全生态脚本化：Rust、Python、Web 三条生态线同时推进；而在 toolchain 深处，自举闭环了 —— 走的是一条曲线。</p>
    <ul>
      <li><strong>Rust 生态（90%）</strong> —— 当作 Rust 的脚本来跑：AutoVM 可调用 90% 以上的 Rust 代码；同一份源码经 a2r 转译为原生 Rust —— 动态开发、静态发布，完美替代现有 Rust 开发流程。</li>
      <li><strong>Python 生态（66%）</strong> —— 直接调用 Python 脚本（如 PyTorch，AI 开发环境即刻可用），也可以把 Auto 翻译成 Python。</li>
      <li><strong>Vue / TS / JS 生态（80%）</strong> —— 可以复刻大部分 Vue 做出来的网站；前端开发的另一种写法。</li>
      <li><strong>曲线自举</strong> —— 宿主侧 (avm, a2r) × 自举侧 (aavm, aa2r)：Auto 暂未实现编译器后端的二进制生成，自举因此走了一条曲线 —— Auto 写的虚拟机与转译器，经 <strong>aa2r</strong> 转译为 Rust、再编译成二进制，跑起 Auto 写的虚拟机与转译器。</li>
    </ul>
    <template #visual>
      <div class="matrix">
        <div class="matrix-head">曲线自举：[avm, a2r] × [aavm, aa2r] × aa2r</div>
        <div class="matrix-grid">
          <div class="matrix-cell host"><strong>avm</strong><span>Rust 宿主解释器</span></div>
          <div class="matrix-cell host"><strong>a2r</strong><span>转译为原生 Rust</span></div>
          <div class="matrix-cell boot"><strong>aavm</strong><span>Auto 写的虚拟机</span></div>
          <div class="matrix-cell boot"><strong>aa2r</strong><span>Auto 写的转译器</span></div>
        </div>
        <div class="matrix-foot">同一份 .at 语料，四条执行通路，全等输出；自举的最后一站借道 Rust 编译器 —— aa2r 转译，编译成二进制</div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">v0.6 展望</h2>
  <p class="section-desc">下一站：从平台到操作系统。</p>
  <div class="features-grid">
    <FeatureCard icon="🖥️" title="AutoOS" description="独立 Linux 发行版（基于 Pop!_OS/COSMIC）；完善的跨平台虚拟桌面；基于 AutoWeb 的远程桌面。" color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="📱" title="AutoUI" description="支持鸿蒙生态；通过 Jetpack Compose 初步支持 Android / iOS。" color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="🤖" title="ROS2 生态" description="基于 Rust/Python 支持机器人节点开发，接入 DORA-RS 等生态。" color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🎮" title="Godot 生态" description="完善 GDScript 支持与 Scene/Node 原生能力，融入 Godot 开发工作流。" color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🔩" title="MCU 生态" description="嵌入式方向：Auto 转译到 C，配合 AutoMan 构建系统走进单片机世界。" color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🧠" title="AutoAI" description="AutoMusk 日常可用（Beta）；AI App 之间的通讯架构与资源调度。" color="rgba(99, 102, 241, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">现在就试 Auto</h2>
  <p class="section-desc">脚本即时跑，发布即 Rust —— 从浏览器到本机，五分钟上手。</p>
  <div class="cta-actions">
    <a href="/zh/docs/" class="cta-btn cta-primary">快速开始</a>
    <a href="/zh/playground" class="cta-btn cta-secondary">打开 Playground</a>
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
