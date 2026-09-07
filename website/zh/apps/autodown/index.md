---
layout: home
---

<script setup>
import FeatureCard from '../../../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../../../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../../../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #6366f1; --page-accent-2: #14b8a6">

<div class="landing-hero">
  <div class="badge">AutoOS 旗舰应用 · Alpha</div>
  <h1 class="title">AutoDown：<span class="accent">会思考的知识库</span></h1>
  <p class="description">
    Auto 语言的 Markdown+YAML 方言,以及用它构建的类 Obsidian 知识库 Jade Garden。
    前后端逻辑全部是 .at 单源 —— 全 Auto 生态里"一份代码、两种形态"的最大示范工程。
  </p>
  <div class="actions">
    <a href="/zh/v05/" class="btn btn-primary">返回 v0.5 发布专题</a>
    <a href="/zh/apps" class="btn btn-secondary">查看全部应用</a>
  </div>
</div>

<div class="stats-section">
  <h2 class="section-title">AutoDown 速览</h2>
  <div class="stats-grid">
    <StatCard value="3.2 万" label="行 .at 单源" description="411 个 .at 文件 —— 全生态最大的 Auto 代码库。" color="#6366f1" />
    <StatCard value="2" label="种运行形态" description="同一份源码:Web(Vue)与桌面(iced)双形态。" color="#14b8a6" />
    <StatCard value="23" label="项 e2e 对拍" description="Playwright 双后端(vue/VM)全等验证。" color="#8b5cf6" />
    <StatCard value="9+" label="后端能力模块" description="parser/links/search/tasks/agenda/query/srs/linkgraph…" color="#ec4899" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="Jade Garden:一棵源码树,两种形态"
    description="29 个 widget 由 .at 经 a2ts 生成 Vue SFC;同一批 DSL 又由 AutoVM 解释、iced 原生渲染。Web 与桌面不是两次开发,是一次开发。"
    badge="双形态"
  >
    <ul>
      <li><strong>三窗格布局</strong> —— 源码树、编辑器、预览联动。</li>
      <li><strong>深浅主题</strong> —— 桌面端暗色/亮色完整支持。</li>
      <li><strong>流式演示</strong> —— edit/view/stream 三种 showcase 模式。</li>
    </ul>
    <template #visual>
      <div class="shot-stack">
        <img src="/v05/autodown-desktop.png" alt="Jade Garden 桌面形态(iced)" class="shot-main" />
        <div class="shot-pair">
          <img src="/v05/autodown-web.png" alt="Jade Garden Web 形态(Vue)" />
        </div>
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="文档即数据"
    description="AutoDown 文档可以被解析、查询、转换与生成 —— 知识库不只是给人读的。"
    badge="方言"
    reverse
  >
    <ul>
      <li><strong>Markdown + YAML</strong> —— 熟悉的写作语法,兼具结构化数据能力。</li>
      <li><strong>知识图谱</strong> —— links 与 linkgraph 模块构建文档间引用网络。</li>
      <li><strong>任务与日程</strong> —— tasks/agenda 模块让 TODO 成为可查询数据。</li>
      <li><strong>间隔重复</strong> —— srs 模块把笔记变成学习卡片。</li>
    </ul>
    <template #visual>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots"><span></span><span></span><span></span></div>
          <span class="code-title">note.down — Markdown + YAML</span>
        </div>
        <pre class="code-body"><code><span class="keyword">meta</span>:
  <span class="function">title</span>: <span class="string">"v0.5 发布复盘"</span>
  <span class="function">tags</span>: [release, v05]
<span class="keyword">#</span> 发布规模
<span class="function">commits</span>: 5711
<span class="function">highlights</span>:
  - AutoOS 虚拟桌面
  - 自举² 矩阵达成</code></pre>
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">为什么 AutoDown 重要</h2>
  <div class="features-grid">
    <FeatureCard icon="🌱" title="单源真值" description="前后端逻辑全 .at,Rust 只做壳 —— Auto 全栈的最高形态。" color="rgba(99, 102, 241, 0.15)" />
    <FeatureCard icon="🔁" title="双端对拍" description="23 项 e2e 在 vue 与 VM 双后端上全等,一致性被测试锁死。" color="rgba(20, 184, 166, 0.15)" />
    <FeatureCard icon="🧠" title="知识即数据" description="可解析、可查询、可生成 —— 知识库是 AutoOS 的记忆层。" color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🛠️" title="同栈工具链" description="用与语言相同的编译器、VM 与转译器构建,没有特殊通道。" color="rgba(245, 158, 11, 0.15)" />
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
