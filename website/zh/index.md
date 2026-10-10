---
layout: home
---

<script setup>
import { introCopy } from '../.vitepress/theme/data/os-ai-introduction'
const copy = introCopy(true)
import HomeHero from '../.vitepress/theme/components/HomeHero.vue'
import HomeDemo from '../.vitepress/theme/components/HomeDemo.vue'
import AutoShellPreview from '../.vitepress/theme/components/AutoShellPreview.vue'
import FeatureCard from '../.vitepress/theme/components/FeatureCard.vue'
import ScreenshotSlot from '../.vitepress/theme/components/ScreenshotSlot.vue'
import { applicationCopy } from '../.vitepress/theme/data/applications'
const applications = applicationCopy(true).apps
const captureIds = { autoedit: 'SHOT-01', automusk: 'SHOT-04', jadeedit: 'SHOT-07' }
const icons = ['🌐', '🦀', '🐍', '🎨', '🤖', '💻']
</script>

<div class="landing-page" style="--page-accent-1: #6366f1; --page-accent-2: #8b5cf6">

<HomeHero
  badge="v0.5 里程碑"
  badge-link="/zh/v05/"
  title=": AI × Lang × OS"
  description="Auto 是一门动静结合的跨生态语言。<br>在 AutoVM 上即时编写脚本 —— 同一份源码发布为地道的 Rust。"
  primary-text="快速开始"
  primary-link="/zh/docs/language/overview#运行与构建"
  secondary-text="在线体验"
  secondary-link="/zh/playground"
/>

<p class="section-desc release-status">里程碑介绍与开发资料现已公开；发行包和下载链接待候选冻结后确认。</p>

<HomeDemo />

<div class="pillars-section">
  <h2 class="section-title">一门语言，贯穿每一层</h2>
  <p class="section-desc">v0.5 让语言、跨端 UI、虚拟桌面与应用开始连接起来；各路径的成熟度分别说明。</p>
  <div class="pillars-grid">
    <FeatureCard icon="🌐" title="语言" description="Actor 并发、类 Rust 泛型、编译期元编程、内存安全。" color="rgba(99, 102, 241, 0.15)" link="/zh/docs/language" />
    <FeatureCard icon="🦀" title="Rust" description="AutoVM 可作为 Rust 脚本环境。a2r 将 Auto 转译为 Rust；接口支持以具体签名与已验证语料为准。" color="rgba(222, 165, 132, 0.15)" link="/zh/rust" />
    <FeatureCard icon="🐍" title="Python" description="AutoVM 可直接调用 Python 代码。a2py 将 Auto 转译为 Python。" color="rgba(59, 130, 246, 0.15)" link="/zh/python" />
    <FeatureCard icon="🎨" title="UI" description="Vue/Web 与 iced/桌面是主要路径；鸿蒙与 Jetpack Compose 处于可行性演示阶段。" color="rgba(168, 85, 247, 0.15)" link="/zh/ui" />
    <FeatureCard icon="🤖" title="AI" :description="copy.home.ai" color="rgba(236, 72, 153, 0.15)" link="/zh/ai" />
    <FeatureCard icon="💻" title="OS" :description="copy.home.os" color="rgba(20, 184, 166, 0.15)" link="/zh/os" />
  </div>
</div>

<div class="apps-section">
  <h2 class="section-title">用 Auto 构建</h2>
  <p class="section-desc">围绕代码、命令、开发任务与知识组织的应用；当前进展见各专题。</p>
  <div class="apps-grid">
    <div v-for="app in applications" :key="app.key" class="app-card">
      <div class="app-shot"><AutoShellPreview v-if="app.key === 'autoshell'" lang="zh" /><ScreenshotSlot v-else :capture-id="captureIds[app.key]" /></div>
      <h3>{{ app.name }}</h3>
      <p>{{ app.summary }}</p>
      <a :href="'/zh/apps/' + app.key + '/'">了解更多 →</a>
    </div>
  </div>
  <p class="section-desc">AutoDown 是文档与编辑器基础；Jade Garden 是已有知识库工程，与 JadeEdit 分别介绍。 <a href="/zh/apps/autodown/">查看相关资料 →</a></p>
</div>

<div class="apps-section">
  <h2 class="section-title">脚本即时跑，发布即 Rust —— 有实证</h2>
  <p class="section-desc">同一份 Auto 源码在 AutoVM 上秒级运行，也可转译为地道的 Rust。行为一致性由三向 parity 体系（AutoVM = a2r 转译 Rust = 原生 Rust）强制验证，而非口头承诺。</p>
  <div class="cta-actions">
    <a href="/zh/script-as-rust" class="cta-btn cta-primary">Auto 如何作为 Rust 脚本层</a>
    <a href="https://github.com/zhaopuming/auto-lang/blob/master/parity/docs/parity-dashboard.html" class="cta-btn cta-secondary">Parity 仪表盘（一致性证据）</a>
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">v0.5 新特性</h2>
  <p class="section-desc">Rust/Python 互操作、Vue/iced 双端 UI、AutoAI 与 AutoOS 应用生态；完整支持范围见发布说明。</p>
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
  min-width: 0;
}

.app-shot {
  margin: -0.5rem -0.5rem 1rem;
  min-width: 0;
  overflow: hidden;
  border-radius: calc(var(--radius) - 2px);
}

.app-shot img {
  display: block;
  width: 100%;
  height: auto;
  border: 1px solid hsl(var(--border));
  border-radius: calc(var(--radius) - 2px);
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
