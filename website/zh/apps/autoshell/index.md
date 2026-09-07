---
layout: home
---

<script setup>
import FeatureCard from '../../../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../../../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../../../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #f59e0b; --page-accent-2: #3b82f6">

<div class="landing-hero">
  <div class="badge">AutoOS 旗舰应用 · Beta</div>
  <h1 class="title">AutoShell：<span class="accent">结构化 Shell</span></h1>
  <p class="description">
    AI 时代的跨平台 Shell。命令之间流动的是类型化对象而非字符串；
    内置安全沙箱与 79 个 Agent 工具 —— 它既是你的日常 Shell，也是 AI Agent 的安全执行层。
  </p>
  <div class="actions">
    <a href="/zh/v05/" class="btn btn-primary">返回 v0.5 发布专题</a>
    <a href="/zh/apps" class="btn btn-secondary">查看全部应用</a>
  </div>
</div>

<div class="stats-section">
  <h2 class="section-title">ASH 速览</h2>
  <div class="stats-grid">
    <StatCard value="18" label="种语义类型" description="管道里流动的是 Atom 对象,不是字符串。" color="#f59e0b" />
    <StatCard value="80+" label="内置命令" description="Windows/Linux/macOS 三平台行为一致。" color="#3b82f6" />
    <StatCard value="79" label="Agent 工具" description="JSON Schema 完整描述,供 AI Agent 安全调用。" color="#8b5cf6" />
    <StatCard value="F1-F4" label="四种模式" description="Shell / AutoScript / AI 翻译 / AI 对话,一键切换。" color="#14b8a6" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="结构化管道"
    description="告别 grep/awk/sed 的字符串手艺。命令返回类型化对象,过滤排序按字段直取。"
    badge="管道"
  >
    <ul>
      <li><strong>类型化数据流</strong> —— 18 种语义类型在命令间原样流动。</li>
      <li><strong>原生格式转换</strong> —— from_json / to_csv / from_yaml / from_toml / from_xml,不再需要 jq。</li>
      <li><strong>AutoLang 内置</strong> —— 闭包、try/catch、类型系统,脚本能力来自 Auto。</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">ash — 结构化管道</span>
        </div>
        <pre class="code-body"><code><span class="prompt">❯</span> ls | <span class="function">filter</span> .size &gt; 10.mb | <span class="function">sort</span> .name | <span class="function">to_csv</span>
<span class="output">name,size,modified
logs,1.2.gb,2026-09-05
target,3.8.gb,2026-09-07</span>
<span class="prompt">❯</span> from_json package.json | <span class="function">get</span> .dependencies | <span class="function">length</span>
<span class="output">42</span></code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="AI 时代的双重身份"
    description="人用它是高效 Shell;Agent 用它是带安全边界的执行层。"
    badge="AI 就绪"
    reverse
  >
    <ul>
      <li><strong>安全沙箱</strong> —— --sandbox / --read-only / --no-network / --audit,给 Agent 的每一步上锁。</li>
      <li><strong>Agent CLI</strong> —— ash agent describe-tools 输出 79 个工具的 JSON Schema;check/run 提供校验执行闭环。</li>
      <li><strong>F3/F4 AI 模式</strong> —— 自然语言直译 Shell 命令、直接对话,由 aaid Daemon 驱动。</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">ash — agent 闭环</span>
        </div>
        <pre class="code-body"><code><span class="prompt">❯</span> ash agent describe-tools | <span class="function">length</span>
<span class="output">79 tools with JSON Schema</span>
<span class="prompt">❯</span> ash --sandbox --read-only agent run plan.at
<span class="output">[check] 12 steps validated
[run]   sandboxed execution, 0 writes</span></code></pre>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">三种形态,一个 Shell</h2>
  <div class="features-grid">
    <FeatureCard icon="⌨️" title="CLI" description="管道与脚本的最纯形态,替代 Bash/Fish/Zsh。" color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🖥️" title="TUI" description="全屏交互界面,键位驱动。" color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🪟" title="GUI" description="AutoUI 描述的图形外壳,与桌面生态同栈。" color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="🌍" title="跨平台" description="Windows、Linux、macOS 处处行为一致。" color="rgba(20, 184, 166, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">继续探索</h2>
  <div class="cta-actions">
    <a href="/zh/v05/" class="cta-btn cta-primary">返回 v0.5 发布专题</a>
    <a href="/zh/apps" class="cta-btn cta-secondary">查看全部应用</a>
  </div>
</div>

</div>
