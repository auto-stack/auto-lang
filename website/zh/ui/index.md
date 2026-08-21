---
layout: home
---

<script setup>
import FeatureCard from '../../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #8b5cf6; --page-accent-2: #ec4899">

<div class="landing-hero">
  <div class="badge">AutoUI <span class="alpha">Alpha</span></div>
  <h1 class="title">一套 UI，<span class="accent">全平台运行</span></h1>
  <p class="description">
    用 Auto 编写一次 UI。为 Web 生成 Vue，为桌面生成 Rust/iced，
    为移动平台生成原生代码 —— 全部来自同一份源码。
  </p>
  <div class="actions">
    <a href="/ui/gallery/index.html" target="_self" class="btn btn-primary">打开组件画廊</a>
    <a href="/zh/docs/ui" class="btn btn-secondary">阅读 UI 文档</a>
  </div>
</div>

<div class="stats-section">
  <h2 class="section-title">v0.5 中的 AutoUI</h2>
  <div class="stats-grid">
    <StatCard value="46+" label="Vue 组件" description="shadcn-vue 复刻，生产可用。" color="#8b5cf6" />
    <StatCard value="24" label="区块" description="预构建页面区块：登录、仪表盘、侧边栏等。" color="#3b82f6" />
    <StatCard value="4" label="平台" description="Web（Vue/Tauri）、桌面（iced）、Android、鸿蒙。" color="#14b8a6" />
    <StatCard value="2" label="DevTools" description="F12 检查器与 MCP，支持 AI 驱动的 UI 开发。" color="#ec4899" />
  </div>
</div>

<div class="platforms-section">
  <h2 class="section-title">支持平台</h2>
  <div class="platforms-grid">
    <a href="/ui/gallery/index.html" target="_self" class="platform-card web">
      <div class="platform-icon">🌐</div>
      <h3>Web <span class="status beta">Beta</span></h3>
      <p>Vue 3 + Tauri。可复刻大部分基于 Vue 的网站。生产可用。</p>
      <span class="platform-link">探索 →</span>
    </a>
    <div class="platform-card desktop">
      <div class="platform-icon">🖥️</div>
      <h3>桌面 <span class="status alpha">Alpha</span></h3>
      <p>Rust/iced 后端。VM 与 A2R 双模式支持 —— 相当于热重载。</p>
      <span class="platform-note">内置 DevTool</span>
    </div>
    <div class="platform-card android">
      <div class="platform-icon">🤖</div>
      <h3>Android <span class="status demo">Demo</span></h3>
      <p>Jetpack Compose 后端。可行性已通过 Demo 验证。</p>
      <span class="platform-note">v0.6 目标</span>
    </div>
    <div class="platform-card harmony">
      <div class="platform-icon">🌏</div>
      <h3>鸿蒙 <span class="status demo">Demo</span></h3>
      <p>ArkTS 后端。可行性已通过 Demo 验证。</p>
      <span class="platform-note">v0.6 目标</span>
    </div>
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="Web —— Vue 与 Tauri"
    description="最成熟的 AutoUI 后端。从 Auto view 块生成 Vue 3 应用，或通过 Tauri 包装为桌面级 Web 应用。"
    badge="Web"
  >
    <ul>
      <li><strong>46+ shadcn-vue 组件</strong> —— 按钮、表格、表单、对话框等</li>
      <li><strong>区块</strong> —— 预构建的登录、仪表盘、侧边栏与设置页面</li>
      <li><strong>图表</strong> —— 基于 Unovis 的面积图、柱状图、折线图与环形图</li>
      <li><strong>Tauri IPC</strong> —— <code>#[tauri::command]</code> 与 Channel 支持桌面集成</li>
    </ul>
    <template #visual>
      <div class="gallery-links">
        <a href="/ui/gallery/index.html" target="_self" class="gallery-link">
          <span class="gallery-icon">🧩</span>
          <span>组件画廊</span>
        </a>
        <a href="/ui/blocks/index.html" target="_self" class="gallery-link">
          <span class="gallery-icon">🧱</span>
          <span>区块画廊</span>
        </a>
        <a href="/ui/charts/index.html" target="_self" class="gallery-link">
          <span class="gallery-icon">📊</span>
          <span>图表画廊</span>
        </a>
        <a href="/ui/a2ui/index.html" target="_self" class="gallery-link">
          <span class="gallery-icon">⚡</span>
          <span>A2UI 演示</span>
        </a>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="桌面 —— Rust/iced"
    description="跨平台桌面应用，具备原生性能。支持 AutoVM 与 A2R 两种执行模式。"
    badge="桌面"
    reverse
  >
    <ul>
      <li><strong>VM 模式</strong> —— 在 AutoVM 中直接运行桌面 UI，支持热重载</li>
      <li><strong>A2R 模式</strong> —— 转译为 Rust/iced 生成生产级二进制</li>
      <li><strong>类 Chrome DevTool</strong> —— 检查组件树、属性与控制台</li>
      <li><strong>MCP 协议</strong> —— 让 AI Agent 查询和操作 AutoUI 界面</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">desktop.at</span>
        </div>
        <pre class="code-body"><code><span class="keyword">view</span> MainWindow {
    <span class="keyword">column</span>(padding: 20) {
        <span class="keyword">text</span>(<span class="string">"Hello, Desktop!"</span>)
        <span class="keyword">button</span>(<span class="string">"Click me"</span>, on_click: handle)
    }
}</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="移动端 —— Android 与鸿蒙"
    description="从同一份 Auto 源码生成原生移动 UI。当前处于可行性验证阶段，v0.6 目标为可用 Beta。"
    badge="移动端"
  >
    <ul>
      <li><strong>Android</strong> —— Jetpack Compose 后端。Demo 应用已验证。</li>
      <li><strong>鸿蒙</strong> —— ArkTS 后端。Demo 应用已验证。</li>
      <li><strong>共享 Widgets</strong> —— 同一套 AutoUI 组件定义跨所有平台复用</li>
      <li><strong>Widgets Gallery</strong> —— 数十个 Auto 实现的 Demo，验证概念可行性</li>
    </ul>
    <template #visual>
      <div class="mobile-grid">
        <div class="mobile-item android">Android</div>
        <div class="mobile-item harmony">鸿蒙</div>
        <div class="mobile-item compose">Compose</div>
        <div class="mobile-item arkts">ArkTS</div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">AutoUI 优势</h2>
  <div class="features-grid">
    <FeatureCard icon="📝" title="单一源码" description="一个 Auto view 块可编译为 Vue、iced、Compose 与 ArkTS。无需平台特定重写。" color="rgba(139, 92, 246, 0.15)" />
    <FeatureCard icon="🔥" title="热重载" description="VM 模式提供即时反馈。A2R 模式提供生产级性能。" color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🔍" title="内置 DevTools" description="F12 检查器，含组件树、属性编辑器与控制台。Chrome DevTools 的 AutoUI 版。" color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🤖" title="MCP for AI" description="AI Agent 可通过 MCP 协议查询和操作你的 UI。自动化 UI 测试与开发。" color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🎨" title="Tailwind 桥接" description="在 AutoUI 中使用 Tailwind 风格工具类。熟悉的样式写法，原生输出。" color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="📦" title="Widget 生态" description="数十个 Demo 与完整 Widgets Gallery，全部使用 Auto 实现。" color="rgba(99, 102, 241, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">构建你的第一个 AutoUI 应用</h2>
  <p class="section-desc">从组件画廊开始，或深入文档构建完整应用。</p>
  <div class="cta-actions">
    <a href="/ui/gallery/index.html" target="_self" class="cta-btn cta-primary">打开画廊</a>
    <a href="/zh/docs/ui" class="cta-btn cta-secondary">阅读 UI 文档</a>
  </div>
</div>

</div>

<style scoped>
.platforms-section {
  padding: 4rem 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.platforms-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 1.5rem;
}

.platform-card {
  padding: 1.5rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  text-decoration: none;
  color: inherit;
}

.platform-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.08);
}

.dark .platform-card:hover {
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.3);
}

.platform-card.web {
  border-color: rgba(139, 92, 246, 0.3);
  background: rgba(139, 92, 246, 0.03);
}

.platform-icon {
  font-size: 2rem;
  margin-bottom: 1rem;
}

.platform-card h3 {
  margin: 0 0 0.75rem;
  font-size: 1.25rem;
  color: hsl(var(--foreground));
}

.platform-card p {
  margin: 0 0 1rem;
  color: hsl(var(--muted-foreground));
  font-size: 0.95rem;
  line-height: 1.6;
}

.status {
  padding: 0.125rem 0.5rem;
  border-radius: 9999px;
  font-size: 0.7rem;
  font-weight: 700;
}

.status.beta { background: rgba(59, 130, 246, 0.15); color: #3b82f6; }
.status.alpha { background: rgba(139, 92, 246, 0.15); color: #8b5cf6; }
.status.demo { background: rgba(245, 158, 11, 0.15); color: #f59e0b; }

.platform-link {
  color: var(--page-accent-1);
  font-weight: 600;
  font-size: 0.95rem;
}

.platform-note {
  color: hsl(var(--muted-foreground));
  font-size: 0.875rem;
}

.gallery-links {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  width: 100%;
  max-width: 420px;
}

.gallery-link {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  text-decoration: none;
  color: hsl(var(--foreground));
  font-weight: 600;
  font-size: 0.9rem;
  transition: all 0.2s ease;
}

.gallery-link:hover {
  border-color: var(--page-accent-1);
  background: color-mix(in srgb, var(--page-accent-1) 5%, transparent);
}

.gallery-icon {
  font-size: 1.25rem;
}

.mobile-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  width: 100%;
  max-width: 320px;
}

.mobile-item {
  padding: 1rem;
  border-radius: var(--radius);
  text-align: center;
  font-weight: 700;
  font-size: 0.85rem;
  color: white;
}

.mobile-item.android { background: linear-gradient(135deg, #3ddc84, #2da766); }
.mobile-item.harmony { background: linear-gradient(135deg, #007dff, #005bb5); }
.mobile-item.compose { background: linear-gradient(135deg, #4285f4, #2b5cb8); }
.mobile-item.arkts { background: linear-gradient(135deg, #f4a460, #d97706); }

@media (max-width: 768px) {
  .gallery-links {
    grid-template-columns: 1fr;
  }
}
</style>
