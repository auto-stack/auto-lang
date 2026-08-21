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
  badge="AutoOS 架构"
  title="：Client / Daemon OS 基础"
  description="AutoOS 正在演进为完整的操作系统层。Client/Daemon 架构、统一配置系统，以及两条未来路线：独立发行版与嵌入式虚拟桌面。"
  primary-text="阅读文档"
  primary-link="/zh/docs/os"
  secondary-text="探索 auto-os-config"
  secondary-link="/zh/docs/os#auto-os-config"
/>

<div class="stats-section">
  <h2 class="section-title">AutoOS 基础</h2>
  <div class="stats-grid">
    <StatCard value="1" label="配置 Daemon" description="auto-os-config —— 所有配置模块的统一 Daemon。" color="#14b8a6" />
    <StatCard value="2" label="未来路线" description="独立 AutoOS 发行版或嵌入式虚拟桌面。" color="#3b82f6" />
    <StatCard value="4+" label="配置模块" description="AI Daemon、Harness、Skills、Roles、Auto Musk 等。" color="#8b5cf6" />
    <StatCard value="0" label="前端代码" description="通用编辑器根据 .at 文件形状自动渲染表单。" color="#f59e0b" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="auto-os-config —— 统一设置中心"
    description="一个 Daemon，一个通用编辑器，服务所有配置模块。Vue 3 SPA + Rust 后端直接读写任意 .at 配置文件。"
    badge="配置"
  >
    <ul>
      <li><strong>统一 Daemon</strong> —— 唯一的配置读写服务。URL 按约定映射为文件路径。</li>
      <li><strong>通用编辑器</strong> —— 根据 .at 数据形状与键名约定自动渲染表单。新模块零前端代码。</li>
      <li><strong>模块注册表</strong> —— 在 <code>modules.d/</code> 中放入 .at 文件即可自动注册新模块。</li>
      <li><strong>自定义 UX</strong> —— 通用编辑器不够用时，可通过 <code>createComponent(Vue)</code> 工厂接入远程 Vue 组件。</li>
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
    title="Client / Daemon 架构"
    description="AutoOS 应用遵循一致模式：系统 Daemon 掌握共享状态与资源；轻量客户端连接它。"
    badge="架构"
    reverse
  >
    <ul>
      <li><strong>aaid</strong> —— AI Daemon，负责 LLM 路由、并发与用量追踪</li>
      <li><strong>auto-os-config-daemon</strong> —— 统一配置读写服务</li>
      <li><strong>AutoShell daemon</strong> —— Shell 会话与任务管理</li>
      <li><strong>未来</strong> —— 窗口管理器、文件系统与设备 Daemon</li>
    </ul>
    <template #visual>
      <div class="arch-diagram">
        <div class="arch-box clients">客户端<br /><small>AutoShell · AutoMusk · 配置 UI</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box daemons">Daemons<br /><small>aaid · config · shell</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box system">系统<br /><small>~/.config/autoos · .at 文件</small></div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="通往 AutoOS 的两条路线"
    description="AutoOS 设计为朝两个方向生长，共享同一套核心架构。"
    badge="路线图"
  >
    <ul>
      <li><strong>独立 AutoOS</strong> —— 基于 Pop!_OS 与 COSMIC Desktop。AutoOS ISO 镜像，搭载 Auto 原生系统应用。</li>
      <li><strong>嵌入式虚拟桌面</strong> —— 运行在 Windows、Linux、macOS 与鸿蒙系统内。基于 AutoUI 构建的虚拟桌面操作系统。</li>
    </ul>
    <template #visual>
      <div class="path-grid">
        <div class="path-card standalone">
          <h4>独立发行版</h4>
          <p>Pop!_OS + COSMIC</p>
          <span>完整系统</span>
        </div>
        <div class="path-card embedded">
          <h4>嵌入式</h4>
          <p>AutoUI 虚拟桌面</p>
          <span>Windows · Linux · macOS · 鸿蒙</span>
        </div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="系统应用"
    description="AutoOS 将搭载一整套原生应用，全部使用 Auto 编写。"
    badge="应用"
    reverse
  >
    <ul>
      <li><strong>文本编辑器</strong> —— 代码高亮与 AutoDown 支持</li>
      <li><strong>计算器</strong> —— 科学模式与编程模式</li>
      <li><strong>扫雷</strong> —— 经典游戏，AutoUI 实现</li>
      <li><strong>日历</strong> —— 日程管理</li>
      <li><strong>Launcher</strong> —— 类 Everything 的快速文件搜索</li>
      <li><strong>任务管理器</strong> —— 类 HTOP 系统监控</li>
      <li><strong>文件浏览器</strong> —— 双面板、键盘驱动</li>
      <li><strong>文件比较器</strong> —— 类 Beyond Compare 的差异对比</li>
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
  <h2 class="section-title">AutoOS 设计原则</h2>
  <div class="features-grid">
    <FeatureCard icon="⚙️" title="一种配置格式" description="所有系统设置使用 .at（auto-atom）文件。一致、可解析、可版本控制。" color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🔌" title="Daemon 优先" description="共享状态保存在 Daemon 中，而非应用里。应用是轻量、可替换的客户端。" color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🎨" title="AutoUI 原生" description="系统 UI 使用 AutoUI 构建。一套框架覆盖桌面、Web 与移动端。" color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="🤖" title="AI 集成" description="AI 能力是系统服务，而非应用插件。每个应用都可通过 aaid 使用 AI。" color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="📦" title="即插即用模块" description="新配置模块只需将 .at 文件放入 modules.d/ 即可注册。无需修改源码。" color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🌐" title="跨平台" description="可作为独立发行版运行，也可嵌入现有操作系统。" color="rgba(139, 92, 246, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">探索 AutoOS</h2>
  <p class="section-desc">阅读设计文档、本地体验 auto-os-config，或跟随路线图走向完整的操作系统。</p>
  <div class="cta-actions">
    <a href="/zh/docs/os" class="cta-btn cta-primary">阅读 OS 文档</a>
    <a href="/zh/docs/releases/v0.5" class="cta-btn cta-secondary">v0.5 发布说明</a>
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
