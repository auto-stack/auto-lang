---
layout: home
---

<script setup>
import FeatureCard from '../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="python-hero">
  <div class="badge">Python 集成 <span class="alpha">Alpha</span></div>
  <h1 class="title">Auto <span class="accent">×</span> Python</h1>
  <p class="description">
    AutoVM 现在可以直接调用 Python 代码。a2py 可将 Auto 转译为 Python。
    在系统编程与 Python 生态之间架起桥梁。
  </p>
  <div class="actions">
    <a href="/zh/docs/design/python-parity-roadmap" class="btn btn-primary">阅读路线图</a>
    <a href="/zh/playground" class="btn btn-secondary">在线体验</a>
  </div>
</div>

<div class="stats-section">
  <h2 class="section-title">接入 Python 生态</h2>
  <div class="stats-grid">
    <StatCard value="直接" label="VM 调用" description="AutoVM 可直接调用 Python 代码。" color="#3b82f6" />
    <StatCard value="a2py" label="转译器" description="Auto → Python，输出地道 Python 代码。" color="#10b981" />
    <StatCard value="PyO3" label="FFI" description="基于 PyO3 的 Rust-Python 互操作。" color="#f59e0b" />
    <StatCard value="AI/ML" label="目标" description="PyTorch、NumPy 与 Python AI 技术栈。" color="#8b5cf6" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="在 AutoVM 中调用 Python"
    description="AutoVM 现已具备一等 Python 互操作能力。像调用原生 Auto 代码一样导入并使用 Python 模块。"
    badge="互操作"
  >
    <ul>
      <li><strong>直接调用 Python</strong> — <code>use.py</code> 导入 Python 模块</li>
      <li><strong>自动类型封送</strong> — Auto 与 Python 类型自动转换</li>
      <li><strong>REPL Python FFI</strong> — 交互式实验</li>
      <li><strong>多类型支持</strong> — 标量、集合与自定义对象</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">ai_script.at</span>
        </div>
        <pre class="code-body"><code><span class="keyword">use</span>.py numpy <span class="keyword">as</span> np;

<span class="keyword">fn</span> <span class="function">main</span>() {
    <span class="keyword">let</span> arr = np.array([1, 2, 3]);
    <span class="function">println</span>(arr.mean());
}</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="a2py 转译器"
    description="将 Auto 编译为 Python。从 Auto 源码生成地道的 Python 代码。"
    badge="转译"
    reverse
  >
    <ul>
      <li><strong>地道输出</strong> — Pythonic 方法映射与静态方法</li>
      <li><strong><code>use</code> → <code>import</code></strong> — 模块系统转译</li>
      <li><strong><code>@dataclass</code></strong> — Auto 结构体变为 Python dataclass</li>
      <li><strong>结构体解构</strong> — 模式匹配支持</li>
      <li><strong>两阶段发射</strong> — 干净、可读的 Python 输出</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">a2py 输出</span>
        </div>
        <pre class="code-body"><code><span class="comment"># Auto 源码</span>
<span class="keyword">struct</span> Point { x: float, y: float }

<span class="comment"># 生成的 Python</span>
<span class="keyword">@dataclass</span>
<span class="keyword">class</span> <span class="function">Point</span>:
    x: float
    y: float</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="走向完全对齐"
    description="Python 支持当前为 Alpha。路线图目标是完整生态接入。"
    badge="路线图"
  >
    <ul>
      <li><strong>v0.5（当前）</strong> — AutoVM 可调用大部分 Python 代码；a2py 可转译基础程序</li>
      <li><strong>v0.6</strong> — 可调用任意 Python 代码；支持 PyTorch 等常见 AI 开发环境</li>
      <li><strong>未来</strong> — 跳过 Python 胶水层，直接调用底层 C/C++ 代码以获得性能</li>
    </ul>
    <template #visual>
      <div class="roadmap-grid">
        <div class="roadmap-item current">
          <span class="version">v0.5</span>
          <span class="status">Alpha</span>
        </div>
        <div class="roadmap-item">
          <span class="version">v0.6</span>
          <span class="status">Beta</span>
        </div>
        <div class="roadmap-item">
          <span class="version">未来</span>
          <span class="status">稳定</span>
        </div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">为什么选择 Auto + Python？</h2>
  <div class="features-grid">
    <FeatureCard icon="🧠" title="AI/ML 生态" description="从 Auto 访问 PyTorch、NumPy、Pandas 及整个 Python AI 技术栈。" color="rgba(59, 130, 246, 0.15)" />
    <FeatureCard icon="🔄" title="双工作流" description="在 AutoVM 中用 Python 互操作进行脚本开发，再转译为 Python 部署。" color="rgba(16, 185, 129, 0.15)" />
    <FeatureCard icon="⚡" title="性能路径" description="未来：绕过 Python 胶水层，直接调用 C/C++ 扩展。" color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="📊" title="数据科学" description="结合 Auto 的类型安全与 Python 的数据生态。" color="rgba(139, 92, 246, 0.15)" />
    <FeatureCard icon="🔧" title="PyO3 驱动" description="为 Python 互操作提供 Rust 级 FFI 性能。" color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🌉" title="桥梁" description="在系统编程与脚本开发之间自由切换，无需更换语言。" color="rgba(20, 184, 166, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">探索 Auto + Python</h2>
  <p class="section-desc">阅读对齐路线图、运行一致性测试，或在 Playground 中体验 Python 互操作。</p>
  <div class="cta-actions">
    <a href="/zh/docs/design/python-parity-roadmap" class="cta-btn cta-primary">阅读路线图</a>
    <a href="/zh/docs/releases/v0.5" class="cta-btn cta-secondary">v0.5 发布说明</a>
  </div>
</div>

<style scoped>
.python-hero {
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
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: #3b82f6;
  font-size: 0.875rem;
  font-weight: 500;
  margin-bottom: 1.5rem;
}

.badge .alpha {
  padding: 0.125rem 0.5rem;
  border-radius: 9999px;
  background: rgba(59, 130, 246, 0.2);
  font-size: 0.75rem;
  font-weight: 700;
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
  background: linear-gradient(120deg, #3b82f6 30%, #10b981 70%);
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
  background: linear-gradient(135deg, #3b82f6 0%, #10b981 100%);
  color: white;
  box-shadow: 0 4px 14px rgba(59, 130, 246, 0.3);
}

.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
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
  background: linear-gradient(180deg, transparent 0%, rgba(59, 130, 246, 0.05) 100%);
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
  background: linear-gradient(135deg, #3b82f6 0%, #10b981 100%);
  color: white;
  box-shadow: 0 4px 14px rgba(59, 130, 246, 0.3);
}

.cta-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
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

.roadmap-grid {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 1rem;
  width: 100%;
  max-width: 420px;
}

.roadmap-item {
  padding: 1.5rem 1rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.roadmap-item.current {
  border-color: #3b82f6;
  background: rgba(59, 130, 246, 0.05);
}

.roadmap-item .version {
  font-weight: 700;
  color: hsl(var(--foreground));
}

.roadmap-item .status {
  font-size: 0.8rem;
  color: #3b82f6;
  font-weight: 600;
}

@media (max-width: 768px) {
  .roadmap-grid {
    grid-template-columns: 1fr;
  }
}
</style>
