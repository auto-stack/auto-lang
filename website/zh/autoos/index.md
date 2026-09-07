---
layout: home
---

<script setup>
import OSHero from '../../.vitepress/theme/components/OSHero.vue'
import FeatureCard from '../../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #14b8a6; --page-accent-2: #3b82f6">

<OSHero
  badge="AutoOS · v0.5"
  title="跑在窗口里的桌面"
  description="窗口管理器本身就是一个 AutoUI 应用。虚拟桌面把 AutoUI 应用装进统一的跨平台桌面 shell —— 同一套窗口语义，从 Windows 到 Linux 再到鸿蒙。"
  primary-text="阅读 OS 文档"
  primary-link="/zh/docs/os"
  secondary-text="虚拟桌面设计"
  secondary-link="/zh/docs/design/autoui/virtual-desktop"
/>

<div class="stats-section">
  <h2 class="section-title">虚拟桌面 · 核心事实</h2>
  <div class="stats-grid">
    <StatCard value="1" label="个 OS 窗口" description="Win/Mac 上宿主即单窗口虚拟合成器，应用窗口是它内部的虚拟窗口。" color="#14b8a6" />
    <StatCard value="100%" label="AutoUI 写成" description="chrome、拖拽、resize、焦点、任务栏 —— 窗口管理器本身是个 AutoUI 应用。" color="#3b82f6" />
    <StatCard value="4" label="条叶子接缝" description="同一套 WM 代码：iced 元素树、离屏纹理、Wayland surface、DOM 节点。" color="#8b5cf6" />
    <StatCard value="2" label="条生长路线" description="独立 AutoOS 发行版（Pop!_OS/COSMIC）或嵌入现有系统的虚拟桌面。" color="#ec4899" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="把应用装进同一个桌面"
    description="任务栏、开始菜单、多虚拟窗口、主题外观 —— 全部由 AutoUI 绘制。这不是套壳的窗口管理器,它自己就是 AutoUI 应用。"
    badge="桌面 shell"
  >
    <ul>
      <li><strong>开始菜单与任务栏</strong> —— 应用启动、焦点切换、窗口召唤,一套桌面交互齐备。</li>
      <li><strong>多虚拟窗口</strong> —— DualApp 同屏多窗口,拖拽/resize/焦点全可操作。</li>
      <li><strong>主题外观</strong> —— 壁纸、明暗主题、accent 档位实时可换。</li>
    </ul>
    <template #visual>
      <div class="shot-stack">
        <img src="/v05/desktop-hero.png" alt="AutoOS 虚拟桌面:任务栏、开始菜单与多窗口" class="shot-main" />
        <div class="shot-pair">
          <img src="/v05/desktop-multiwindow.png" alt="多虚拟窗口" />
          <img src="/v05/desktop-light.png" alt="浅色主题桌面" />
        </div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="统一设置中心 —— auto-os-config"
    description="一个 Daemon，一个通用编辑器，服务所有配置模块。URL 按约定映射 .at 文件，表单按数据形状自动渲染 —— 新模块零前端代码。"
    badge="设置中心"
  >
    <ul>
      <li><strong>模块注册表</strong> —— 在 modules.d/ 放入 .at 文件即自动注册。</li>
      <li><strong>AI Daemon 管理</strong> —— aaid、Roles、Skills、Musk 配置开箱即管。</li>
      <li><strong>主题系统</strong> —— accent 档位（indigo/coral/ocean/sage/amber）实时切换。</li>
    </ul>
    <template #visual>
      <div class="shot-pair">
        <img src="/v05/autoos-config-agents.png" alt="设置中心 Agents 模块" />
        <img src="/v05/autoos-config-skills.png" alt="设置中心 Skills 模块" />
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="首个真实应用 —— auto-kanban"
    description="配置驱动的通用只读看板，几乎 100% Auto 编写（13 个 .at 文件，零手写 Rust）。同一份源码，Web 与桌面双轨渲染。"
    badge="应用"
    reverse
  >
    <ul>
      <li><strong>配置驱动</strong> —— 看板长什么样由 .at 配置描述，不改代码换场景。</li>
      <li><strong>双轨 parity</strong> —— Vue 网页端与 iced 桌面端逐项对拍。</li>
      <li><strong>v0.6 首发系统应用</strong> —— 它是 AutoOS 内置应用矩阵的第一个成员。</li>
    </ul>
    <template #visual>
      <div class="shot-pair">
        <img src="/v05/kanban-web.png" alt="auto-kanban Web 形态" />
        <img src="/v05/kanban-desktop.png" alt="auto-kanban 桌面形态" />
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="通往 AutoOS 的两条路线"
    description="共享同一套核心架构，朝两个方向生长。"
    badge="路线图"
  >
    <ul>
      <li><strong>嵌入式虚拟桌面</strong> —— 运行在 Windows、Linux、macOS 与鸿蒙之内；v0.6 完善跨平台体验，并基于 AutoWeb 打通远程桌面。</li>
      <li><strong>独立 AutoOS 发行版</strong> —— 基于 Pop!_OS 与 COSMIC Desktop 制作 ISO 镜像，用 Auto 自制的系统应用替换原生应用。</li>
    </ul>
    <template #visual>
      <div class="arch-diagram">
        <div class="arch-box clients">应用层<br /><small>看板 · 设置中心 · 终端 · 系统应用矩阵</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box daemons">虚拟桌面 shell<br /><small>WM-as-App · 单窗口虚拟合成器</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box system">宿主<br /><small>Windows / Linux(Smithay) / 鸿蒙 / 发行版</small></div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">v0.6 系统应用矩阵（预告）</h2>
  <div class="features-grid">
    <FeatureCard icon="📝" title="文本编辑器" description="代码高亮 + AutoDown 支持。" color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🧮" title="计算器" description="科学模式与编程模式。" color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🔍" title="Launcher" description="类 Everything 的快速文件搜索。" color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="📊" title="任务管理器" description="类 HTOP 的系统监控。" color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="📁" title="文件浏览器" description="双面板、键盘驱动。" color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🔀" title="文件比较器" description="类 Beyond Compare 的差异对比。" color="rgba(139, 92, 246, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">探索 AutoOS</h2>
  <div class="cta-actions">
    <a href="/zh/docs/os" class="cta-btn cta-primary">阅读 OS 文档</a>
    <a href="/zh/v05/" class="cta-btn cta-secondary">返回 v0.5 发布专题</a>
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
