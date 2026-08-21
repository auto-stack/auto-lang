---
layout: home
---

<script setup>
import FeatureCard from '../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #dea584; --page-accent-2: #f59e0b">

<div class="landing-hero">
  <div class="badge">Rust 集成 <span class="beta">Beta</span></div>
  <h1 class="title">Auto <span class="accent">×</span> Rust</h1>
  <p class="description">
    将 Auto 作为 Rust 的脚本环境。AutoVM 可直接调用几乎任何 Rust 代码，
    也可通过 A2R 将 Auto 转译为生产级 Rust。
  </p>
  <div class="actions">
    <a href="/zh/docs/design/auto-as-rust-script-strategy" class="btn btn-primary">阅读设计文档</a>
    <a href="/zh/playground" class="btn btn-secondary">在线体验</a>
  </div>
</div>

<div class="stats-section">
  <h2 class="section-title">深度 Rust 集成</h2>
  <div class="stats-grid">
    <StatCard value="任意" label="Rust 代码" description="AutoVM 可直接调用几乎任何 Rust 代码。" color="#dea584" />
    <StatCard value="A2R" label="转译器" description="Auto → Rust，行为与 AutoVM 保持一致。" color="#f59e0b" />
    <StatCard value="2" label="标准库模式" description="Auto 独立标准库或 Rust 原生标准库。" color="#3b82f6" />
    <StatCard value="100%" label="后端自由" description="从 Auto 代码中调用任意 Rust crate。" color="#10b981" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="AutoVM 作为 Rust 脚本环境"
    description="AutoVM 不再只是语言虚拟机，而是完整的 Rust 脚本环境。"
    badge="脚本化"
  >
    <ul>
      <li><strong>调用任意 Rust 代码</strong> — 从 Auto 脚本直接调用</li>
      <li><strong>热重载</strong> — 无需重新编译宿主程序即可迭代 Rust 逻辑</li>
      <li><strong>零 FFI 样板代码</strong> — AutoVM 自动处理类型封送</li>
      <li><strong>嵌入 Rust 库</strong> — 作为一等 Auto 模块使用</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">script.at</span>
        </div>
        <pre class="code-body"><code><span class="keyword">use.rust</span> std::path::PathBuf
<span class="keyword">use.rust</span> std::time::Duration
<span class="keyword">let</span> p = PathBuf.from(<span class="string">"src/main.rs"</span>)
<span class="function">print</span>(p)
<span class="keyword">let</span> d = Duration.from_secs(5)
<span class="function">print</span>(d)  <span class="comment">// prints 5000ms</span></code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="A2R 转译器"
    description="将 Auto 编译为 Rust，行为与 AutoVM 保持一致。从 Auto 源码直接发布生产级二进制。"
    badge="转译"
    reverse
  >
    <ul>
      <li><strong>行为一致</strong> — A2R 输出与 AutoVM 语义保持一致</li>
      <li><strong>Axum HTTP 生成</strong> — <code>#[api]</code> → 完整 Axum 服务器</li>
      <li><strong>Tauri IPC</strong> — <code>#[tauri::command]</code> 与 Channel 支持</li>
      <li><strong>逃逸分析</strong> — 自动选择最优的 borrow / clone / Rc</li>
      <li><strong>合并模式</strong> — 多文件项目级转译</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">a2r 输出</span>
        </div>
        <pre class="code-body"><code><span class="comment">// Auto 源码</span>
<span class="keyword">fn</span> <span class="function">add</span>(a: int, b: int) -> int { a + b }
<span class="comment">// 生成的 Rust</span>
<span class="keyword">fn</span> <span class="function">add</span>(a: i64, b: i64) -> i64 { a + b }</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="双标准库模式"
    description="根据部署目标选择最合适的标准库。"
    badge="标准库"
  >
    <ul>
      <li><strong>Auto 独立标准库</strong> — io、net、http、fs 等常用功能。当前支持 Rust，未来支持 Python 等平台。</li>
      <li><strong>Rust 嵌入模式</strong> — 直接使用 Rust 标准库和任意 crates.io 第三方库，无需重新实现。</li>
    </ul>
    <template #visual>
      <div class="mode-grid">
        <div class="mode-card auto">
          <h4>Auto 标准库</h4>
          <p>跨平台可移植</p>
          <code>Json.parse(data)</code>
        </div>
        <div class="mode-card rust">
          <h4>Rust 标准库</h4>
          <p>完整 crates.io 访问</p>
          <code>use.rust std::net::TcpListener</code>
        </div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">为什么选择 Auto + Rust？</h2>
  <div class="features-grid">
    <FeatureCard icon="🚀" title="脚本化速度" description="无需为每个任务搭建 Cargo 项目，用 Auto 编写 Rust 驱动的脚本。" color="rgba(222, 165, 132, 0.15)" />
    <FeatureCard icon="📦" title="Crate 生态" description="无需离开 Auto 即可接入 crates.io。Axum、Tokio、Serde 全部可用。" color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🔄" title="迭代工作流" description="在 AutoVM 中快速原型，用 A2R 发布生产代码。同一套代码，两种执行模式。" color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🛡️" title="内存安全" description="Auto 的所有权系统与 Rust 借用检查器自然映射。" color="rgba(16, 185, 129, 0.15)" />
    <FeatureCard icon="⚡" title="零成本抽象" description="A2R 生成地道 Rust 代码，无运行时开销。" color="rgba(139, 92, 246, 0.15)" />
    <FeatureCard icon="🔧" title="逃逸分析" description="编译器根据实际使用模式自动选择 borrow、clone 或 Rc。" color="rgba(236, 72, 153, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">开始使用 Auto + Rust</h2>
  <p class="section-desc">阅读设计文档、运行 Cookbook 示例，或在 Playground 中体验 Rust 脚本模式。</p>
  <div class="cta-actions">
    <a href="/zh/docs/design/auto-as-rust-script-strategy" class="cta-btn cta-primary">阅读设计文档</a>
    <a href="/zh/docs/releases/v0.5" class="cta-btn cta-secondary">v0.5 发布说明</a>
  </div>
</div>

</div>

<style scoped>
.mode-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  width: 100%;
  max-width: 420px;
}

.mode-card {
  padding: 1.5rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  text-align: center;
}

.mode-card h4 {
  margin: 0 0 0.5rem;
  color: hsl(var(--foreground));
}

.mode-card p {
  margin: 0 0 1rem;
  font-size: 0.875rem;
  color: hsl(var(--muted-foreground));
}

.mode-card code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  color: var(--page-accent-1);
  background: color-mix(in srgb, var(--page-accent-1) 10%, transparent);
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
}

@media (max-width: 768px) {
  .mode-grid {
    grid-template-columns: 1fr;
  }
}
</style>
