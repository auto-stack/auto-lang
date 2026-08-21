---
layout: home
---

<script setup>
import AIHero from '../.vitepress/theme/components/AIHero.vue'
import FeatureCard from '../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #ec4899; --page-accent-2: #a855f7">

<AIHero
  badge="AutoAI 架构"
  title="：Client / Daemon AI 基础设施"
  description="为所有 AutoOS 应用提供统一的 AI 基础设施。并发仲裁、API 密钥保管、模型路由与用量追踪 —— 全部通过单一 Daemon 完成。"
  primary-text="阅读文档"
  primary-link="/zh/docs/ai"
  secondary-text="在线体验"
  secondary-link="/zh/playground"
/>

<div class="stats-section">
  <h2 class="section-title">AutoAI 核心数据</h2>
  <div class="stats-grid">
    <StatCard value="4" label="核心 Crates" description="ai-config、auto-ai-daemon、auto-ai-client、auto-ai-agent。" color="#ec4899" />
    <StatCard value="2" label="Agent 应用" description="AutoAI-Cli 与 AutoMusk，均基于共享基础设施构建。" color="#a855f7" />
    <StatCard value="1" label="Daemon" description="aaid —— 所有 AutoOS 应用的唯一 LLM 网关。" color="#6366f1" />
    <StatCard value="∞" label="模型" description="跨 OpenAI、Anthropic、智谱等提供商的模型无关路由。" color="#14b8a6" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="Client / Daemon 架构"
    description="AutoAI 在应用与 LLM 提供商之间引入清晰的分层。应用从不直接调用 LLM —— 每个请求都经过 aaid Daemon。"
    badge="架构"
  >
    <ul>
      <li><strong>auto-ai-daemon (aaid)</strong> —— 唯一 LLM 网关。掌握所有提供商知识、并发池、密钥保管与用量追踪。</li>
      <li><strong>auto-ai-client</strong> —— 轻量 HTTP 客户端。发送规范化请求，接收规范化响应，不感知提供商。</li>
      <li><strong>ai-config</strong> —— 共享线协议类型与提供商配置。规范化 ContentBlock 模型。</li>
      <li><strong>auto-ai-agent</strong> —— Profession 库、ReAct 循环、Workflow 引擎。通过 ai-config 校验模型。</li>
    </ul>
    <template #visual>
      <div class="arch-diagram">
        <div class="arch-box apps">应用<br /><small>AutoAI-Cli · AutoMusk · Forge</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box client">auto-ai-client<br /><small>规范化 HTTP</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box daemon">aaid daemon<br /><small>并发 · 密钥 · 路由</small></div>
        <div class="arch-arrow">↓</div>
        <div class="arch-box providers">LLM 提供商<br /><small>OpenAI · Anthropic · 智谱</small></div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="AutoMusk —— 通用 Coding Agent"
    description="基于 AutoPlan 的通用 Coding Agent，使用 Auto 语言自身实现。"
    badge="Agent"
    reverse
  >
    <ul>
      <li><strong>AutoPlan 集成</strong> —— 结构化规划与执行</li>
      <li><strong>多提供商支持</strong> —— 兼容 aaid 服务的任意模型</li>
      <li><strong>自托管</strong> —— 用 Auto 编写，运行在 AutoVM 上</li>
      <li><strong>通过 auto-os-config 配置</strong> —— 统一设置管理</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">automusk.at</span>
        </div>
        <pre class="code-body"><code><span class="keyword">use</span> auto_ai_agent::Agent;
<span class="keyword">use</span> auto_ai_client::AiClient;
<span class="keyword">fn</span> <span class="function">main</span>() {
    <span class="keyword">let</span> client = AiClient::new();
    <span class="keyword">let</span> agent = Agent::from_profession(
        <span class="string">"coder"</span>, client
    );
    agent.run(<span class="string">"fix the parser bug"</span>);
}</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="AutoAI-Cli —— 终端 Coding Agent"
    description="轻量级终端 Coding Agent，用于快速任务与 Shell 集成。"
    badge="CLI"
  >
    <ul>
      <li><strong>终端原生</strong> —— 无需离开 Shell 即可获得编码帮助</li>
      <li><strong>相同基础设施</strong> —— 使用 auto-ai-client 与 aaid</li>
      <li><strong>AutoShell 集成</strong> —— AutoShell 内的 F3 AI 模式</li>
      <li><strong>低开销</strong> —— 快速启动，快速响应</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">terminal</span>
        </div>
        <pre class="code-body"><code><span class="prompt">$</span> aictl status
<span class="output">Daemon: running (pid 1234)
Providers: zhipu, anthropic
Active pools: 2/8</span>
<span class="prompt">$</span> auto-ai-cli "explain this error"
<span class="output">The error occurs because...</span></code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="统一配置"
    description="所有 AutoAI 应用通过 auto-os-config 与 .at 文件共享配置。"
    badge="配置"
    reverse
  >
    <ul>
      <li><strong>~/.config/autoos/ai-client.at</strong> —— 提供商与默认配置</li>
      <li><strong>~/.config/autoos/ai-daemon.at</strong> —— 监听地址、并发、真实 API 密钥</li>
      <li><strong>auto-os-config</strong> —— 用于编辑所有配置模块的 Web UI</li>
      <li><strong>环境变量回退</strong> —— ZHIPU_API_KEY、ANTHROPIC_API_KEY、OPENAI_API_KEY</li>
    </ul>
    <template #visual>
      <div class="config-tree">
        <div class="config-file">~/.config/autoos/</div>
        <div class="config-item">├── ai-client.at</div>
        <div class="config-item">├── ai-daemon.at</div>
        <div class="config-item">├── auto-musk.at</div>
        <div class="config-item">├── roles/</div>
        <div class="config-item">└── skills/</div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">AutoAI 优势</h2>
  <div class="features-grid">
    <FeatureCard icon="🎯" title="单一网关" description="一个 Daemon 掌握所有 LLM 通信。应用保持简单且提供商无关。" color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🔐" title="密钥保管" description="API 密钥保存在 Daemon 中，而非分散在每个应用里。集中式安全管理。" color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="⚡" title="并发仲裁" description="按提供商划分的信号量池，防止跨应用的限流风暴。" color="rgba(99, 102, 241, 0.15)" />
    <FeatureCard icon="📊" title="用量追踪" description="按应用统计 Token 与请求量。精确掌握每个 Agent 的成本。" color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🔄" title="模型路由" description="将请求路由至最佳可用模型。内置故障转移与回退。" color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🧩" title="Agent 框架" description="Profession、Workflow 与 ReAct 原语，用于构建复杂 Agent。" color="rgba(59, 130, 246, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">用 Auto 构建 AI 驱动的应用</h2>
  <p class="section-desc">启动 aaid Daemon，链接 auto-ai-client，几分钟内发布你的第一个 Agent。</p>
  <div class="cta-actions">
    <a href="/zh/docs/design/15-ai-daemon-infrastructure" class="cta-btn cta-primary">阅读设计文档</a>
    <a href="/zh/playground" class="cta-btn cta-secondary">打开 Playground</a>
  </div>
</div>

</div>

<style scoped>
.arch-box.apps { background: linear-gradient(135deg, #ec4899, #db2777); }
.arch-box.client { background: linear-gradient(135deg, #a855f7, #8b5cf6); }
.arch-box.daemon { background: linear-gradient(135deg, #6366f1, #4f46e5); }
.arch-box.providers { background: linear-gradient(135deg, #14b8a6, #0d9488); }
</style>
