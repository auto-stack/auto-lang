---
layout: home
---

<script setup>
import OSHero from '../.vitepress/theme/components/OSHero.vue'
import FeatureCard from '../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<OSHero
  badge="Language as OS"
  title="：代码中的操作系统"
  description="Auto 模糊了语言与操作系统的边界。编译期执行、多目标转译器、从 MCU 到云端的全平台统一运行时。"
  primary-text="阅读文档"
  primary-link="/zh/docs/os"
  secondary-text="探索 LaOS"
  secondary-link="/zh/docs/os#laos"
/>

<div class="stats-section">
  <h2 class="section-title">突破不可能三角</h2>
  <div class="stats-grid">
    <StatCard value="9/10" label="开发效率" description="脚本模式、简洁泛型、直观语法。" color="#f59e0b" />
    <StatCard value="9/10" label="内存安全" description="线性类型、借用检查、智能转换。" color="#3b82f6" />
    <StatCard value="9/10" label="运行性能" description="AOT 编译、零成本抽象、无 GC 停顿。" color="#ec4899" />
    <StatCard value="6" label="目标平台" description="Windows、Linux、macOS、Android、鸿蒙、MCU。" color="#14b8a6" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="Language as OS (LaOS)"
    description="传统技术栈将语言、运行时和操作系统割裂为互不相关的层。Auto 将它们融合为一个连贯的整体。"
    badge="架构理念"
  >
    <ul>
      <li><strong>微内核语言</strong>：核心特性（AutoVM、转译器、CTE）充当内核</li>
      <li><strong>内存子系统</strong>：所有权 + 借用检查 + AutoFree + RC + 逃逸分析</li>
      <li><strong>I/O 子系统</strong>：统一的 <code>io.at</code> 接口，vm/rs/c 多后端</li>
      <li><strong>Shell 子系统</strong>：ASH 以结构化数据管道取代 Bash</li>
      <li><strong>UI 子系统</strong>：Aura 覆盖 Web、桌面、移动端和 MCU（LVGL）</li>
    </ul>
    <template #visual>
      <div class="arch-diagram">
        <div class="arch-layer app">应用层</div>
        <div class="arch-arrow">↓</div>
        <div class="arch-layer lang">Auto 语言</div>
        <div class="arch-arrow">↓</div>
        <div class="arch-layer runtime">Auto 运行时</div>
        <div class="arch-arrow">↓</div>
        <div class="arch-layer hw">硬件层</div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="AutoVM & 转译器矩阵"
    description="一门语言，无数目标。AutoVM 运行脚本和嵌入式代码，转译器则为每个平台生成本地代码。"
    badge="执行引擎"
    reverse
  >
    <ul>
      <li><strong>AutoVM</strong>：独立模式、编译期模式、嵌入式模式</li>
      <li><strong>a2rs</strong>：Rust 后端，面向系统编程</li>
      <li><strong>a2c</strong>：C 后端，面向嵌入式和汽车电子（90% C 支持）</li>
      <li><strong>a2ts</strong>：TypeScript 后端，面向 Web 前端</li>
      <li><strong>a2py</strong>：Python 后端，面向 AI 和脚本</li>
      <li><strong>a2kt</strong>：Kotlin 后端，面向 Android / Jetpack Compose</li>
      <li><strong>a2vue / a2jet / a2ark</strong>：Web、桌面、鸿蒙的 UI 目标</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span /><span /><span /></div>
          <span class="code-title">hello.at</span>
        </div>
        <pre class="code-body"><code><span class="comment">// 可在所有目标上运行</span>
<span class="keyword">fn</span> <span class="function">main</span>() {
    <span class="keyword">let</span> msg = <span class="string">"Hello, OS!"</span>;
    <span class="function">println</span>(msg);
}</code></pre>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="内存与并发"
    description="安全而不妥协。Auto 将 Rust 级别的内存安全与多种自动回收策略及 Actor 模型并发融为一体。"
    badge="系统级"
  >
    <ul>
      <li><strong>线性类型 + 借用检查</strong>：编译期所有权追踪</li>
      <li><strong>AutoFree</strong>：确定性自动内存清理</li>
      <li><strong>RC + 逃逸分析</strong>：在安全场景下自动引用计数</li>
      <li><strong>Task / Msg</strong>：无共享状态的 Actor 模型并发</li>
      <li><strong>~T（异步）</strong>：原生异步类型</li>
      <li><strong>Atom 协议</strong>：跨任务边界的零拷贝序列化</li>
    </ul>
    <template #visual>
      <div class="memory-grid">
        <div class="mem-item safe">Linear</div>
        <div class="mem-item safe">Borrow</div>
        <div class="mem-item auto">AutoFree</div>
        <div class="mem-item auto">RC</div>
        <div class="mem-item auto">Escape</div>
        <div class="mem-item task">Actor</div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="跨平台适配器"
    description="从汽车 MCU 到云服务器，Auto 适配平台，而非让平台适配 Auto。"
    badge="可移植性"
    reverse
  >
    <ul>
      <li><strong>Windows / Linux / macOS</strong>：Rust 原生后端</li>
      <li><strong>SOC / 机器人</strong>：Rust / C++（ROS）后端</li>
      <li><strong>MCU / 嵌入式</strong>：基于 FreeRTOS 的 C 后端</li>
      <li><strong>移动 / Web</strong>：a2vue、a2jet、a2ark 转译器</li>
      <li><strong>设备间通信</strong>：通过 Auto Virtual Bus 实现 RPC</li>
      <li><strong>整车开发</strong>：从中央 HPC 到传感器 MCU 的全栈覆盖</li>
    </ul>
    <template #visual>
      <div class="platform-grid">
        <div class="plat-item desktop">桌面</div>
        <div class="plat-item mobile">移动</div>
        <div class="plat-item soc">SOC / ROS</div>
        <div class="plat-item mcu">MCU</div>
        <div class="plat-item web">Web</div>
        <div class="plat-item vehicle">整车</div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">LaOS 子系统</h2>
  <div class="features-grid">
    <FeatureCard icon="⚙️" title="编译期执行" description="在编译期执行 Auto 代码。生成代码、验证不变式、在程序运行前配置系统。" color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="🔌" title="Auto Virtual Bus" description="基于 Atom 协议的跨进程、跨设备 RPC。远程对象如同本地。" color="rgba(99, 102, 241, 0.15)" />
    <FeatureCard icon="🖥️" title="AutoShell (ASH)" description="结构化 Shell，基于 Atom 管道。告别字符串解析，命令返回类型化对象。" color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🎨" title="Aura UI" description="声明式 UI 框架，转译为 Vue、Jetpack Compose 和 ArkTS。一套代码，全屏覆盖。" color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="📦" title="AutoMan" description="通用项目管理器和包管理系统。跨所有目标的构建、测试和部署。" color="rgba(245, 158, 11, 0.15)" />
    <FeatureCard icon="🛡️" title="SafeClaw 安全" description="NanoClaw 转译为安全运行时。IronClaw 特性加上 Aura UI 构建安全 Agent 界面。" color="rgba(59, 130, 246, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">准备好构建下一代操作系统了吗？</h2>
  <p class="section-desc">了解 Auto 如何将语言、运行时和操作系统统一为单一工具链。</p>
  <div class="cta-actions">
    <a href="/zh/docs/os" class="cta-btn cta-primary">阅读 OS 文档</a>
    <a href="/zh/docs/" class="cta-btn cta-secondary">探索特性</a>
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

.arch-diagram {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  max-width: 320px;
}

.arch-layer {
  width: 100%;
  padding: 1rem;
  border-radius: var(--radius);
  text-align: center;
  font-weight: 700;
  color: white;
}

.arch-layer.app { background: linear-gradient(135deg, #f59e0b, #d97706); }
.arch-layer.lang { background: linear-gradient(135deg, #6366f1, #8b5cf6); }
.arch-layer.runtime { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.arch-layer.hw { background: linear-gradient(135deg, #14b8a6, #0d9488); }

.arch-arrow {
  color: hsl(var(--muted-foreground));
  font-size: 1.2rem;
}

.memory-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  width: 100%;
  max-width: 320px;
}

.mem-item {
  padding: 0.875rem;
  border-radius: var(--radius);
  text-align: center;
  font-weight: 600;
  font-size: 0.85rem;
  color: white;
}

.mem-item.safe { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.mem-item.auto { background: linear-gradient(135deg, #6366f1, #4f46e5); }
.mem-item.task { background: linear-gradient(135deg, #ec4899, #db2777); grid-column: span 2; }

.platform-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  width: 100%;
  max-width: 320px;
}

.plat-item {
  padding: 0.875rem;
  border-radius: var(--radius);
  text-align: center;
  font-weight: 600;
  font-size: 0.85rem;
  color: white;
}

.plat-item.desktop { background: linear-gradient(135deg, #6366f1, #4f46e5); }
.plat-item.mobile { background: linear-gradient(135deg, #ec4899, #db2777); }
.plat-item.soc { background: linear-gradient(135deg, #f59e0b, #d97706); }
.plat-item.mcu { background: linear-gradient(135deg, #14b8a6, #0d9488); }
.plat-item.web { background: linear-gradient(135deg, #3b82f6, #2563eb); }
.plat-item.vehicle { background: linear-gradient(135deg, #8b5cf6, #7c3aed); }

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  }
}
</style>
