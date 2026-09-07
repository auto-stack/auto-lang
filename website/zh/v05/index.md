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
  description="Auto 从一门语言成长为一个平台：AutoOS 桌面、四大旗舰应用、双生态脚本化、自举² 达成 —— 这一次，Auto 开始成为它自己。"
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
