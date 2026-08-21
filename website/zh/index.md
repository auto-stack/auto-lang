---
layout: home
---

<script setup>
import HomeHero from '../.vitepress/theme/components/HomeHero.vue'
import FeatureCard from '../.vitepress/theme/components/FeatureCard.vue'
const icons = ['🌐', '🦀', '🐍', '🎨', '🤖', '💻']
</script>

<div class="landing-page" style="--page-accent-1: #6366f1; --page-accent-2: #8b5cf6">

<HomeHero
  badge="v0.5 现已发布"
  title="：语言 + 运行时 + AI + OS"
  description="Auto 是一个全栈应用平台。用同一门语言编写脚本、后端、UI、AI Agent 和操作系统组件 —— 运行在 AutoVM 上，或转译为 Rust、Python 和 TypeScript。"
  primary-text="快速开始"
  primary-link="/zh/docs/"
  secondary-text="在线体验"
  secondary-link="/zh/playground"
/>

<div class="pillars-section">
  <h2 class="section-title">一门语言，贯穿每一层</h2>
  <p class="section-desc">v0.5 将 Auto 从一门语言进化为构建现代应用的完整平台。</p>
  <div class="pillars-grid">
    <FeatureCard icon="🌐" title="语言" description="Actor 并发、类 Rust 泛型、编译期元编程、内存安全。" color="rgba(99, 102, 241, 0.15)" link="/zh/docs/language" />
    <FeatureCard icon="🦀" title="Rust" description="AutoVM 可作为 Rust 脚本环境。A2R 将 Auto 转译为生产级 Rust。支持双标准库模式。" color="rgba(222, 165, 132, 0.15)" link="/zh/rust" />
    <FeatureCard icon="🐍" title="Python" description="AutoVM 可直接调用 Python 代码。a2py 将 Auto 转译为 Python。" color="rgba(59, 130, 246, 0.15)" link="/zh/python" />
    <FeatureCard icon="🎨" title="UI" description="Vue 和 Tauri 版本已成熟。桌面版（Rust/iced）基本可用。鸿蒙与 Android 通过可行性验证。" color="rgba(168, 85, 247, 0.15)" link="/zh/ui" />
    <FeatureCard icon="🤖" title="AI" description="Client/Daemon 架构。AutoAI-Cli 终端 Coding Agent。AutoMusk 通用 Coding Agent。" color="rgba(236, 72, 153, 0.15)" link="/zh/ai" />
    <FeatureCard icon="💻" title="OS" description="Client/Daemon 架构、统一配置系统。未来双路线：独立 AutoOS 发行版与嵌入式虚拟桌面。" color="rgba(20, 184, 166, 0.15)" link="/zh/os" />
  </div>
</div>

<div class="apps-section">
  <h2 class="section-title">用 Auto 构建</h2>
  <p class="section-desc">证明平台可用的真实应用。</p>
  <div class="apps-grid">
    <div class="app-card">
      <h3>AutoShell</h3>
      <p>跨平台 Shell，支持 CLI/TUI/GUI 三种形态，具备类 Warp 的 AI 能力。</p>
      <a href="/zh/apps#autoshell">了解更多 →</a>
    </div>
    <div class="app-card">
      <h3>AutoMusk</h3>
      <p>基于 AutoPlan 的通用 Coding Agent，用 Auto 语言自身实现。</p>
      <a href="/zh/apps#automusk">了解更多 →</a>
    </div>
    <div class="app-card">
      <h3>AutoDown</h3>
      <p>Auto 语言的方言，融合 Markdown 与 YAML，可用于表达任意知识库。</p>
      <a href="/zh/apps#autodown">了解更多 →</a>
    </div>
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">v0.5 新特性</h2>
  <p class="section-desc">迄今最大里程碑：Rust 集成、Python 支持、双标准库模式、成熟 AutoUI、AutoAI 架构与 AutoOS 基础。</p>
  <div class="cta-actions">
    <a href="/zh/docs/releases/v0.5" class="cta-btn cta-primary">阅读发布说明</a>
    <a href="/zh/playground" class="cta-btn cta-secondary">打开 Playground</a>
  </div>
</div>

<div class="icp-footer">
  <a href="https://beian.miit.gov.cn/" target="_blank">粤ICP备2026054131号-1</a>
</div>

</div>

<style scoped>
.pillars-section {
  padding: 4rem 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.pillars-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1.5rem;
}

.apps-section {
  padding: 4rem 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.apps-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 1.5rem;
}

.app-card {
  padding: 1.5rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.app-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.08);
}

.dark .app-card:hover {
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.3);
}

.app-card h3 {
  margin: 0 0 0.5rem;
  font-size: 1.25rem;
  color: hsl(var(--foreground));
}

.app-card p {
  margin: 0 0 1rem;
  color: hsl(var(--muted-foreground));
  font-size: 0.95rem;
  line-height: 1.6;
}

.app-card a {
  color: #6366f1;
  text-decoration: none;
  font-weight: 600;
  font-size: 0.95rem;
}

.app-card a:hover {
  text-decoration: underline;
}

.icp-footer {
  padding: 2rem;
  text-align: center;
  font-size: 0.875rem;
  color: hsl(var(--muted-foreground));
}

.icp-footer a {
  color: hsl(var(--muted-foreground));
  text-decoration: none;
}

.icp-footer a:hover {
  text-decoration: underline;
}
</style>
