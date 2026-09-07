---
layout: home
---

<script setup>
import FeatureCard from '../../../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../../../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../../../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #8b5cf6; --page-accent-2: #ec4899">

<div class="landing-hero">
  <div class="badge">AutoOS 旗舰应用 · Alpha</div>
  <h1 class="title">AutoUI Apps：<span class="accent">界面即代码</span></h1>
  <p class="description">
    数十个 Demo 与完整的 Widgets Gallery,全部用 Auto 语言实现。
    一份 .at 声明,同时生成 Web(Vue)与桌面(iced)两端 —— 下面就是活的画廊,不是截图。
  </p>
  <div class="actions">
    <a href="/ui/gallery/" class="btn btn-primary">全屏打开画廊</a>
    <a href="/zh/ui" class="btn btn-secondary">AutoUI 能力总览</a>
  </div>
</div>

<div class="stats-section">
  <h2 class="section-title">AutoUI Apps 速览</h2>
  <div class="stats-grid">
    <StatCard value="46+" label="基础组件" description="按钮到日历、图表到布局,语义化组件目录。" color="#8b5cf6" />
    <StatCard value="24" label="区块 Blocks" description="导航、页脚、定价表等成套页面区段。" color="#ec4899" />
    <StatCard value="30+" label="Demo 应用" description="Hero 区、定价表、聊天、扫雷、画板、视频…" color="#6366f1" />
    <StatCard value="2" label="渲染后端" description="Vue(Web)与 iced(桌面)逐项 parity 对拍。" color="#14b8a6" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="活的 Widgets Gallery"
    description="下面的画廊是真实运行的应用 —— Vue 实现直接跑在你的浏览器里。同一份 .at 源码,在桌面上由 iced 原生渲染。"
    badge="活体演示"
  >
    <ul>
      <li><strong>组件目录</strong> —— 按钮、表单、对话框、图表、标签页……点开即试。</li>
      <li><strong>主题与 accent</strong> —— 暗色/亮色与五档主题色实时切换。</li>
      <li><strong>移动端适配</strong> —— gallery_mobile 模式一键预览。</li>
    </ul>
    <template #visual>
      <div class="live-frame">
        <div class="live-bar"><span></span><span></span><span></span><em>/ui/gallery/ — 实时运行</em></div>
        <iframe src="/ui/gallery/" title="Widgets Gallery 实时演示" loading="lazy"></iframe>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Demo 矩阵与双端一致性"
    description="每个 Demo 都是完整的 .at 工程;VM 实拍与 Vue 实拍逐项对拍,一致性由体系保证而非口头承诺。"
    badge="双端一致"
    reverse
  >
    <ul>
      <li><strong>全栈示例</strong> —— 015-notes:Vue 前端 + Rust 后端,由 Auto 生成。</li>
      <li><strong>桌面 Demo</strong> —— iced 应用,支持热重载与类 Chrome 的 DevTools。</li>
      <li><strong>MCP 协议</strong> —— Agent 可对 AutoUI 界面进行任意操作与查询。</li>
    </ul>
    <template #visual>
      <div class="shot-pair">
        <img src="/v05/gallery-home.png" alt="Widgets Gallery 桌面形态(VM)" />
        <div class="demo-list">
          <div>006 Hero Section</div>
          <div>008 Pricing Table</div>
          <div>009 Article Feed</div>
          <div>015 Notes(全栈)</div>
          <div>017 Chat</div>
          <div>038 Minesweeper</div>
        </div>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">AutoUI 的三层能力</h2>
  <div class="features-grid">
    <FeatureCard icon="🌐" title="Web(Beta)" description="能复刻大部分基于 Vue.js 的网站。" color="rgba(139, 92, 246, 0.15)" />
    <FeatureCard icon="🖥️" title="Desktop(Alpha)" description="Rust/iced 原生渲染,VM 与 a2r 双轨可用,相当于自带热重载。" color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="📱" title="移动端(POC)" description="鸿蒙与 Android 通过可行性验证,可运行 Demo。" color="rgba(99, 102, 241, 0.15)" />
    <FeatureCard icon="🤖" title="Agent 友好" description="MCP 协议让 AI 直接操作与验证界面 —— AI 时代的 UI 框架。" color="rgba(20, 184, 166, 0.15)" />
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

<style scoped>
.live-frame {
  width: 100%;
  max-width: 520px;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  overflow: hidden;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
  background: hsl(var(--card));
}

.live-bar {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid hsl(var(--border));
}

.live-bar span {
  width: 9px;
  height: 9px;
  border-radius: 50%;
}

.live-bar span:nth-child(1) { background: #ff5f56; }
.live-bar span:nth-child(2) { background: #ffbd2e; }
.live-bar span:nth-child(3) { background: #27c93f; }

.live-bar em {
  margin-left: 0.5rem;
  font-style: normal;
  font-size: 0.75rem;
  font-family: 'JetBrains Mono', monospace;
  color: hsl(var(--muted-foreground));
}

.live-frame iframe {
  display: block;
  width: 100%;
  height: 480px;
  border: 0;
}

.shot-pair {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  width: 100%;
  max-width: 480px;
}

.shot-pair img {
  width: 100%;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
}

.demo-list {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  justify-content: center;
}

.demo-list div {
  padding: 0.45rem 0.75rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  font-size: 0.8rem;
  font-family: 'JetBrains Mono', monospace;
  color: hsl(var(--muted-foreground));
}
</style>
