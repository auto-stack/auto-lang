---
layout: home
---

<script setup>
import FeatureCard from '../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="apps-hero">
  <div class="badge">用 Auto 构建</div>
  <h1 class="title">真实应用，<span class="accent">真实平台</span></h1>
  <p class="description">
    这些应用证明 Auto 不仅是一门语言 —— 它是构建 Shell、Agent、知识库和全栈系统的平台。
  </p>
  <div class="actions">
    <a href="/zh/docs/releases/v0.5" class="btn btn-primary">v0.5 发布说明</a>
    <a href="/zh/playground" class="btn btn-secondary">在线体验</a>
  </div>
</div>

<div class="stats-section">
  <h2 class="section-title">v0.5 应用矩阵</h2>
  <div class="stats-grid">
    <StatCard value="4" label="旗舰应用" description="AutoShell、AutoMusk、AutoDown 与 AutoUI Demos。" color="#6366f1" />
    <StatCard value="100%" label="Auto 编写" description="所有应用均使用 Auto 语言自身实现。" color="#8b5cf6" />
    <StatCard value="3" label="平台" description="CLI、TUI 与 GUI 三种模式覆盖应用套件。" color="#14b8a6" />
    <StatCard value="∞" label="可扩展" description="每个应用都是构建你自己的应用的参考。" color="#ec4899" />
  </div>
</div>

<div class="apps-list">
  <div class="app-section" id="autoshell">
    <div class="app-header">
      <div class="app-icon">🐚</div>
      <div>
        <h2>AutoShell</h2>
        <span class="app-status beta">Beta</span>
      </div>
    </div>
    <p class="app-desc">
      完整的跨平台 Shell，可替代 Bash、Fish 与 Zsh。
      支持 CLI、TUI、GUI 三种形态，具备类 Warp 的 AI 能力。
    </p>
    <div class="app-features">
      <div class="app-feature">
        <h4>结构化管道</h4>
        <p>告别字符串解析。命令通过 Atom 管道返回类型化对象。</p>
      </div>
      <div class="app-feature">
        <h4>AI 集成</h4>
        <p>F3 AI 模式由 aaid 驱动。自然语言直接转 Shell 命令。</p>
      </div>
      <div class="app-feature">
        <h4>跨平台</h4>
        <p>Windows、Linux、macOS。一个 Shell，处处行为一致。</p>
      </div>
      <div class="app-feature">
        <h4>双实现</h4>
        <p>Rust 版功能完整。Auto 版完全可用。</p>
      </div>
    </div>
  </div>

  <div class="app-section" id="automusk">
    <div class="app-header">
      <div class="app-icon">🤖</div>
      <div>
        <h2>AutoMusk</h2>
        <span class="app-status beta">Beta</span>
      </div>
    </div>
    <p class="app-desc">
      基于 AutoPlan 的通用 Coding Agent。用 Auto 编写，
      运行在 AutoVM 上，通过 auto-os-config 配置。
    </p>
    <div class="app-features">
      <div class="app-feature">
        <h4>AutoPlan 驱动</h4>
        <p>结构化规划与执行，应对复杂编码任务。</p>
      </div>
      <div class="app-feature">
        <h4>多提供商</h4>
        <p>兼容 aaid Daemon 服务的任意模型。</p>
      </div>
      <div class="app-feature">
        <h4>自托管</h4>
        <p>使用 Auto 自身实现。证明 Auto 能构建真实 Agent。</p>
      </div>
      <div class="app-feature">
        <h4>配置 UI</h4>
        <p>通过 auto-os-config 编辑 Roles、Skills 与 Modes。</p>
      </div>
    </div>
  </div>

  <div class="app-section" id="autodown">
    <div class="app-header">
      <div class="app-icon">📄</div>
      <div>
        <h2>AutoDown</h2>
        <span class="app-status alpha">Alpha</span>
      </div>
    </div>
    <p class="app-desc">
      Auto 语言的方言，融合 Markdown 与 YAML。
      用结构化、可解析的文档表达任意知识库。
    </p>
    <div class="app-features">
      <div class="app-feature">
        <h4>Markdown + YAML</h4>
        <p>熟悉的语法，兼具结构化数据能力。</p>
      </div>
      <div class="app-feature">
        <h4>可解析</h4>
        <p>文档即数据。可查询、转换与生成。</p>
      </div>
      <div class="app-feature">
        <h4>知识库</h4>
        <p>构建 Wiki、文档与结构化内容系统。</p>
      </div>
      <div class="app-feature">
        <h4>Auto 方言</h4>
        <p>Auto 生态的一部分。使用相同工具链编译。</p>
      </div>
    </div>
  </div>

  <div class="app-section" id="autoui-demos">
    <div class="app-header">
      <div class="app-icon">🎨</div>
      <div>
        <h2>AutoUI Demos</h2>
        <span class="app-status alpha">Alpha</span>
      </div>
    </div>
    <p class="app-desc">
      数十个 Demo 与完整的 Widgets Gallery，全部使用 Auto 实现。
      证明 AutoUI 能够构建真实界面。
    </p>
    <div class="app-features">
      <div class="app-feature">
        <h4>Widgets Gallery</h4>
        <p>46+ 组件、24 个区块、图表与布局。</p>
      </div>
      <div class="app-feature">
        <h4>全栈示例</h4>
        <p>015-notes：Vue 前端 + Rust 后端，由 Auto 生成。</p>
      </div>
      <div class="app-feature">
        <h4>桌面 Demo</h4>
        <p>Rust/iced 应用，支持热重载与 DevTools。</p>
      </div>
      <div class="app-feature">
        <h4>移动原型</h4>
        <p>Android 与鸿蒙 Demo 应用。</p>
      </div>
    </div>
  </div>
</div>

<div class="features-section">
  <h2 class="section-title">这些应用为何重要</h2>
  <div class="features-grid">
    <FeatureCard icon="✅" title="平台证明" description="真实应用证明 Auto 已具备生产可用性，而非玩具语言。" color="rgba(99, 102, 241, 0.15)" />
    <FeatureCard icon="📚" title="参考实现" description="研究这些应用，学习如何构建你自己的 Auto 项目。" color="rgba(139, 92, 246, 0.15)" />
    <FeatureCard icon="🔄" title="自举验证" description="Auto 构建 Auto。编译器、虚拟机与应用同属一个生态。" color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🚀" title="v0.6 基础" description="这些应用将成为未来 AutoOS 的系统应用。" color="rgba(236, 72, 153, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">开始用 Auto 构建</h2>
  <p class="section-desc">以这些应用为参考，或深入文档构建你自己的项目。</p>
  <div class="cta-actions">
    <a href="/zh/docs/" class="cta-btn cta-primary">阅读文档</a>
    <a href="/zh/playground" class="cta-btn cta-secondary">打开 Playground</a>
  </div>
</div>

<style scoped>
.apps-hero {
  padding: 6rem 2rem 4rem;
  text-align: center;
  max-width: 900px;
  margin: 0 auto;
}

.badge {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border-radius: 9999px;
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.3);
  color: #6366f1;
  font-size: 0.875rem;
  font-weight: 500;
  margin-bottom: 1.5rem;
}

.title {
  font-size: clamp(2.5rem, 5vw, 4rem);
  font-weight: 800;
  line-height: 1.1;
  letter-spacing: -0.02em;
  color: hsl(var(--foreground));
  margin: 0 0 1.5rem;
}

.accent {
  background: linear-gradient(120deg, #6366f1 30%, #8b5cf6 70%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.description {
  font-size: 1.25rem;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
  max-width: 700px;
  margin: 0 auto 2.5rem;
}

.actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
  flex-wrap: wrap;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.5rem;
  border-radius: var(--radius);
  font-weight: 600;
  font-size: 0.95rem;
  transition: all 0.2s ease;
  text-decoration: none;
}

.btn-primary {
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  color: white;
  box-shadow: 0 4px 14px rgba(99, 102, 241, 0.3);
}

.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(99, 102, 241, 0.4);
}

.btn-secondary {
  background: hsl(var(--card));
  color: hsl(var(--foreground));
  border: 1px solid hsl(var(--border));
}

.btn-secondary:hover {
  background: hsl(var(--accent));
}

.stats-section {
  padding: 4rem 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.section-title {
  font-size: 2rem;
  font-weight: 700;
  text-align: center;
  margin-bottom: 2.5rem;
  color: hsl(var(--foreground));
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 1.5rem;
}

.apps-list {
  max-width: 1000px;
  margin: 0 auto;
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 3rem;
}

.app-section {
  padding: 2rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
}

.app-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
}

.app-icon {
  font-size: 2.5rem;
}

.app-header h2 {
  margin: 0;
  font-size: 1.75rem;
  color: hsl(var(--foreground));
}

.app-status {
  padding: 0.25rem 0.75rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  font-weight: 700;
}

.app-status.beta { background: rgba(59, 130, 246, 0.15); color: #3b82f6; }
.app-status.alpha { background: rgba(139, 92, 246, 0.15); color: #8b5cf6; }

.app-desc {
  font-size: 1.1rem;
  color: hsl(var(--muted-foreground));
  line-height: 1.7;
  margin: 0 0 1.5rem;
}

.app-features {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
}

.app-feature h4 {
  margin: 0 0 0.5rem;
  font-size: 0.95rem;
  color: hsl(var(--foreground));
}

.app-feature p {
  margin: 0;
  font-size: 0.875rem;
  color: hsl(var(--muted-foreground));
  line-height: 1.5;
}

.features-section {
  padding: 4rem 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1.5rem;
}

.cta-section {
  padding: 4rem 2rem;
  text-align: center;
  background: linear-gradient(180deg, transparent 0%, rgba(99, 102, 241, 0.05) 100%);
}

.section-desc {
  font-size: 1.1rem;
  color: hsl(var(--muted-foreground));
  max-width: 500px;
  margin: 0 auto 2rem;
}

.cta-actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
  flex-wrap: wrap;
}

.cta-btn {
  display: inline-flex;
  align-items: center;
  padding: 0.875rem 2rem;
  border-radius: var(--radius);
  font-weight: 600;
  text-decoration: none;
  transition: all 0.2s ease;
}

.cta-primary {
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  color: white;
  box-shadow: 0 4px 14px rgba(99, 102, 241, 0.3);
}

.cta-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(99, 102, 241, 0.4);
}

.cta-secondary {
  background: hsl(var(--card));
  color: hsl(var(--foreground));
  border: 1px solid hsl(var(--border));
}

.cta-secondary:hover {
  background: hsl(var(--accent));
}

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  }
}
</style>
