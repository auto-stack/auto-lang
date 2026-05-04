---
layout: home
---

<script setup>
import AIHero from '../.vitepress/theme/components/AIHero.vue'
import FeatureCard from '../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<AIHero
  badge="Made BY AI · Made FOR AI · Made of AI"
  title="为 AI 而生的语言"
  description="一门由 AI 构建、为 AI 优化、由 AI 构成的编程语言。从极低解析熵的语法到原生 Agent 架构。"
  primary-text="阅读文档"
  primary-link="/zh/docs/ai"
  secondary-text="在线体验"
  secondary-link="/zh/playground"
/>

<div class="stats-section">
  <h2 class="section-title">以 AI 速度开发</h2>
  <div class="stats-grid">
    <StatCard value="100亿" label="Tokens 消耗" description="智谱 AI MAX 套餐驱动整个开发周期。" color="#6366f1" />
    <StatCard value="1300+" label="Commits" description="短短两个月的快速迭代。" color="#8b5cf6" />
    <StatCard value="20万" label="行 Rust 代码" description="生产级编译器、虚拟机和转译器基础设施。" color="#ec4899" />
    <StatCard value="1+6" label="编译器矩阵" description="1 个解释器 + 6 个转译器（a2c、a2rs、a2ts、a2py、a2kt、a2vue）。" color="#14b8a6" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="Made BY AI"
    description="Auto 诞生于一个激进实验：一门现代系统级语言能否几乎完全由 AI 构建？"
    badge="开发模式"
  >
    <ul>
      <li>编译器前端与后端均由大模型协作生成</li>
      <li>从概念到可工作转译器，仅用两个月冲刺</li>
      <li>人类把控架构，AI 执行实现</li>
      <li>证明 AI 能够攻克复杂系统编程任务</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span /><span /><span /></div>
          <span class="code-title">generated.at</span>
        </div>
        <pre class="code-body"><code><span class="keyword">fn</span> <span class="function">main</span>() {
    <span class="comment">// AI 生成的核心解析器</span>
    <span class="keyword">let</span> ast = parser::parse(src);
    <span class="function">println</span>(ast.dump());
}</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Made FOR AI"
    description="每一项语法决策都致力于降低解析熵和 Token 消耗，使 Auto 成为最适合 AI 生成的语言。"
    badge="语法设计"
    reverse
  >
    <ul>
      <li><strong>类型后置</strong>：<code>radius float</code> —— 先实体，后约束</li>
      <li><strong>语义确定性</strong>：显式优于隐式，零歧义</li>
      <li><strong>双模式错误信息</strong>：人类友好（miette）vs 结构化 JSON（供 AI 使用）</li>
      <li><strong>意图注解</strong>：通过注解实现契约编程</li>
      <li><strong>节省 80%~90% Tokens</strong>：同等逻辑下远低于传统语言消耗</li>
    </ul>
    <template #visual>
      <div class="comparison-table">
        <div class="comp-row">
          <span class="comp-lang">C</span>
          <code class="comp-code">float radius;</code>
        </div>
        <div class="comp-row">
          <span class="comp-lang">Auto</span>
          <code class="comp-code highlight">radius float</code>
        </div>
        <div class="comp-note">AI 先看到标识符，再确认类型约束</div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Made of AI"
    description="Auto 不仅是 AI 的工具，更成为 AI Agent 运行的底层基质。"
    badge="Agent 架构"
  >
    <ul>
      <li><strong>SafeClaw</strong>：转译型 Agent 集群（AutoAgent + AutoTool + AutoSkill + AutoKnowledge）</li>
      <li><strong>AI 编译加速</strong>：直接转译到 CUDA、CANN、ROCm、MUSA</li>
      <li><strong>Atom 协议</strong>：大模型输出结构化数据，由 Auto 安全执行</li>
      <li><strong>一次生成，到处运行</strong>：一套代码，覆盖所有 GPU 平台</li>
    </ul>
    <template #visual>
      <div class="gpu-grid">
        <div class="gpu-item nvidia">CUDA</div>
        <div class="gpu-item huawei">CANN</div>
        <div class="gpu-item amd">ROCm</div>
        <div class="gpu-item moore">MUSA</div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">核心 AI 优势</h2>
  <div class="features-grid">
    <FeatureCard icon="🧠" title="极低解析熵" description="简化的 AST 结构大幅降低大模型的幻觉率和上下文窗口压力。" color="rgba(99, 102, 241, 0.15)" />
    <FeatureCard icon="🔍" title="物理透明" description="内存布局可见且可预测。AI 能精确计算数据占用和缓存行为。" color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="⚡" title="Code for AI, Summary for Human" description="Auto 将机器优化表示与人类可读摘要分离，各取所需。" color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🛡️" title="SafeClaw Agents" description="转译型 Agent 框架，Auto 代码本身即为自治 AI 系统的运行时。" color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🎯" title="意图注解" description="注解意图而非实现。编译器与 AI 协作填充细节。" color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🔄" title="反向转译" description="通过将现有代码库反向转译为 Auto，生成大规模训练语料。" color="rgba(59, 130, 246, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">准备好探索 AI 原生编程了吗？</h2>
  <p class="section-desc">深入阅读详细文档，或在交互式 Playground 中体验 Auto。</p>
  <div class="cta-actions">
    <a href="/zh/docs/ai" class="cta-btn cta-primary">阅读 AI 文档</a>
    <a href="/zh/playground" class="cta-btn cta-secondary">打开 Playground</a>
  </div>
</div>

<style scoped>
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

.showcase-wrapper {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
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

.code-window {
  background: #1e1e2e;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  width: 100%;
  max-width: 420px;
  text-align: left;
}

.code-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  background: #181825;
  border-bottom: 1px solid #313244;
}

.code-dots {
  display: flex;
  gap: 0.4rem;
}

.code-dots span {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.code-dots span:nth-child(1) { background: #ff5f56; }
.code-dots span:nth-child(2) { background: #ffbd2e; }
.code-dots span:nth-child(3) { background: #27c93f; }

.code-title {
  font-size: 0.8rem;
  color: #6c7086;
  font-family: 'JetBrains Mono', monospace;
}

.code-body {
  padding: 1.25rem;
  margin: 0;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 0.85rem;
  line-height: 1.6;
  color: #cdd6f4;
  overflow-x: auto;
}

.keyword { color: #cba6f7; }
.function { color: #89b4fa; }
.string { color: #a6e3a1; }
.comment { color: #6c7086; font-style: italic; }

.comparison-table {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: var(--radius);
  padding: 1.5rem;
  width: 100%;
  max-width: 420px;
}

.comp-row {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 0;
  border-bottom: 1px solid hsl(var(--border));
}

.comp-row:last-child {
  border-bottom: none;
}

.comp-lang {
  font-weight: 600;
  width: 60px;
  color: hsl(var(--muted-foreground));
}

.comp-code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.9rem;
  color: hsl(var(--foreground));
}

.comp-code.highlight {
  color: #a855f7;
  font-weight: 600;
}

.comp-note {
  margin-top: 1rem;
  font-size: 0.8rem;
  color: hsl(var(--muted-foreground));
  text-align: center;
}

.gpu-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  width: 100%;
  max-width: 320px;
}

.gpu-item {
  padding: 1rem;
  border-radius: var(--radius);
  text-align: center;
  font-weight: 700;
  font-size: 0.9rem;
  color: white;
}

.gpu-item.nvidia { background: linear-gradient(135deg, #76b900, #558b00); }
.gpu-item.huawei { background: linear-gradient(135deg, #cf0a2c, #9a0721); }
.gpu-item.amd { background: linear-gradient(135deg, #ed1c24, #b71518); }
.gpu-item.moore { background: linear-gradient(135deg, #3b82f6, #1d4ed8); }

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  }
}
</style>
