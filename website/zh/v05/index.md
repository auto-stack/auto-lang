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
  description="动静相宜、前后解耦、Language as OS —— 三个理念长成一个平台：AutoOS 桌面、四大旗舰应用、双生态脚本化、自举² 达成。这一次，Auto 开始成为它自己。"
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
      <p class="philosophy-text">多数语言逼你二选一：脚本语言秒级周转，却把性能留在门外；系统语言性能拉满，却让你用编译周期偿还每一次灵感。Auto 拒绝选择 —— 开发态由 AutoVM 解释执行，秒级启动、热重载、REPL 与 LSP 常伴左右；发布态，同一份源码经 a2r 转译为原生 Rust（或经 a2c 转译为 C）。而在 Auto 的整个版图里，这套动静组合是每个生态共用的设计原点 —— 不止 Rust 一家。</p>
      <ul class="philosophy-list">
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
      <p class="philosophy-text">前后解耦不是 UI 的专利，而是 Auto 在每一层反复使用的架构元模式。最外层是 AutoUI：契约独立于宿主框架，同一份 .at 在 Vue、iced、ArkTS、Jetpack Compose 之间切换渲染臂。往里走，模式一路重现 —— 直到操作系统的顶层：外壳与内核，也是一对前后端。</p>
      <ul class="philosophy-list">
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
      <p class="philosophy-text">操作系统的三大天职 —— 共享、抽象、服务 —— 正是一门语言必须回答的三问。Auto 把答案写进语言本体：Task/Msg Actor 并发模型是调度器，view/mut/move 内存三元组是内存管理，io/net/http/fs 标准库是系统调用，多端渲染臂是外设驱动。更进一步：每个 OS 模块都有两条供给路线 —— Auto 亲手实现，或经生态桥调用现成实现 —— 按目标平台自由"装机"。</p>
      <ul class="philosophy-list">
        <li><strong>模块语言化，双路供给</strong> —— 跑在 Windows 上：应用与 UI 转译为 Rust/iced，内核调用 Windows 现成能力；跑进 MCU：应用与 UI 可转译为 C/LVGL，内核换成 Auto 自写的 RTOS。同一门语言，按机器"装机"。</li>
        <li><strong>OS 的形状已经可见</strong> —— AutoOS 虚拟桌面正在运行，窗口管理器本身就是一个 AutoUI 应用（见下一节）；auto-os-config 让配置文件的形状自动长成设置中心。</li>
        <li><strong>AI 算力按 OS 设备调度</strong> —— Client/Daemon 架构已在 AutoAI 落地：向系统要算力，而不是每个应用自建 AI 栈。</li>
      </ul>
      <div class="philosophy-proof">调度 · 内存 · 系统调用 · 外设 —— 件件可自实现，件件可生态桥</div>
    </article>
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="AutoOS 虚拟桌面"
    description="窗口管理器本身是一个 AutoUI 应用。单 OS 窗口内的虚拟合成器，把 AutoUI 应用装进一个跨平台桌面 —— Windows 今天可用，Linux 桌面环境与鸿蒙在路线图上。"
    badge="AutoOS"
  >
    <ul>
      <li><strong>WM-as-App</strong> —— 虚拟窗口的 chrome、拖拽、焦点、任务栏全部用 AutoUI 写成普通应用，双端一致性是构造性保证。</li>
      <li><strong>统一设置中心</strong> —— auto-os-config：一个 Daemon 按 .at 文件形状自动渲染配置表单，零前端代码接入新模块。</li>
      <li><strong>首个真实应用</strong> —— auto-kanban 看板，几乎 100% Auto 编写，Web 与桌面双轨渲染。</li>
      <li><strong>AutoTerm 终端设施</strong> —— PTY + alacritty 仿真核心，AutoOS 的终端底座。</li>
    </ul>
    <template #visual>
      <div class="shot-stack">
        <img src="/v05/desktop-hero.png" alt="AutoOS 虚拟桌面:任务栏、开始菜单与多窗口" class="shot-main" />
        <div class="shot-pair">
          <img src="/v05/kanban-desktop.png" alt="auto-kanban 桌面形态" />
          <img src="/v05/autoos-config-agents.png" alt="AutoOS 统一设置中心 Agents 模块" />
        </div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">四大旗舰应用，100% Auto 构建</h2>
  <p class="section-desc">每个应用都有独立落地页 —— 它们是平台可用性的证明，也是你构建自己应用的参考。</p>
  <div class="features-grid">
    <FeatureCard icon="🤖" title="AutoMusk" description="通用 Coding Agent。AutoPlan 模式驱动，前端五个视图由 .at 单源生成 —— 用 Auto 写的 Agent。" color="rgba(236, 72, 153, 0.15)" link="/zh/apps/automusk/" />
    <FeatureCard icon="🐚" title="AutoShell" description="AI 时代的结构化 Shell。命令交换类型化对象而非文本流，内置安全沙箱与 79 个 Agent 工具。" color="rgba(245, 158, 11, 0.15)" link="/zh/apps/autoshell/" />
    <FeatureCard icon="📄" title="AutoDown" description="Markdown+YAML 方言与类 Obsidian 知识库 Jade Garden。前后端逻辑全 .at 单源，Web/桌面双形态。" color="rgba(99, 102, 241, 0.15)" link="/zh/apps/autodown/" />
    <FeatureCard icon="🎨" title="AutoUI Apps" description="46+ 组件画廊、数十个 Demo、全栈示例 —— 全部 Auto 实现。本页直接嵌着活的画廊。" color="rgba(139, 92, 246, 0.15)" link="/zh/apps/autoui/" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="新版 Playground：浏览器里的 Auto 实验台"
    description="几乎全部 Auto 示例代码开箱即览，新增 Debug 支持 —— 语料、金样、书页代码在浏览器里就近试炼。"
    badge="Playground"
    reverse
  >
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
    title="语言进展：脚本化双生态 + 自举²"
    description="v0.5 让 Auto 同时住进 Rust 和 Python 的生态，也让 Auto 的工具链开始由 Auto 自己编写。"
    badge="语言"
  >
    <ul>
      <li><strong>Rust 生态（Alpha）</strong> —— Auto 作为 Rust 的脚本语言：AutoVM 可调用 90% 以上的 Rust 代码；a2r 转译出的 Rust 与 AutoVM 行为基本一致。</li>
      <li><strong>Python 生态（PreAlpha）</strong> —— 普通Python 脚本可直接调用，PyTorch 支持打通 AI 开发环境。</li>
      <li><strong>自举²（2×2 矩阵）</strong> —— 宿主侧 (avm, a2r) × 自举侧 (aavm, aa2r)：Auto 写的虚拟机与转译器已经达成，工具链开始"自己跑自己"。</li>
    </ul>
    <template #visual>
      <div class="matrix">
        <div class="matrix-head">同一份 .at 语料 → 四条执行通路</div>
        <div class="matrix-grid">
          <div class="matrix-cell host"><strong>avm</strong><span>Rust 宿主解释器</span></div>
          <div class="matrix-cell host"><strong>a2r</strong><span>转译为原生 Rust</span></div>
          <div class="matrix-cell boot"><strong>aavm</strong><span>Auto 写的虚拟机</span></div>
          <div class="matrix-cell boot"><strong>aa2r</strong><span>Auto 写的转译器</span></div>
        </div>
        <div class="matrix-foot">自举回路中不再有离不开 Rust 的那颗芯</div>
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
