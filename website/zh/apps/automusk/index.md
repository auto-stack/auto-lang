---
layout: home
---

<script setup>
import AIHero from '../../../.vitepress/theme/components/AIHero.vue'
import FeatureCard from '../../../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../../../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../../../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #ec4899; --page-accent-2: #a855f7">

<AIHero
  badge="AutoOS 旗舰应用 · 100% Auto"
  title="AutoMusk Coding Agent"
  description="通用 Coding Agent,以 AutoPlan 模式驱动:规划、执行、复审全流程结构化。用 Auto 语言自身实现 —— 平台能造 Agent 的最好证明。"
  primary-text="在线体验 Playground"
  primary-link="/zh/playground"
  secondary-text="AutoAI 架构"
  secondary-link="/zh/ai"
/>

<div class="stats-section">
  <h2 class="section-title">AutoMusk 速览</h2>
  <div class="stats-grid">
    <StatCard value="AutoPlan" label="规划模式" description="计划→执行→复审的结构化编码闭环。" color="#ec4899" />
    <StatCard value=".at" label="前端单源" description="五个视图由 .at 源经 auto build 生成,148 项对拍全等。" color="#a855f7" />
    <StatCard value="多模型" label="提供商无关" description="经 aaid Daemon 路由 OpenAI/Anthropic/智谱等任意模型。" color="#6366f1" />
    <StatCard value="2" label="形态" description="iced 桌面应用与 Web 工作台双形态。" color="#14b8a6" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="桌面工作台"
    description="会话、计划、规范、知识库一栏贯通;Block 卡片让 Agent 的每一步产出结构化可见。"
    badge="iced 桌面"
  >
    <ul>
      <li><strong>AutoPlan 流水线</strong> —— 规划→执行→复审,spec 单一真源,token 更省。</li>
      <li><strong>Block 卡片渲染</strong> —— Agent 产出以 Block 呈现,而非一坨文本。</li>
      <li><strong>工作目录感知</strong> —— 选择工作目录后按仓内约定干活。</li>
    </ul>
    <template #visual>
      <div class="shot-stack">
        <img src="/v05/automusk-app.png" alt="AutoMusk 桌面主界面" class="shot-main" />
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="Web 工作台:Forge / Specs / Relay / Wiki"
    description="前端五个视图(Login/Chats/Plans/Specs/Wiki)的 .at 单源,经 auto build 生成 Vue 工程 —— .at 是唯一真源,双端对拍 148 项全等。"
    badge="Web"
    reverse
  >
    <ul>
      <li><strong>Forge</strong> —— 与 Agent 的对话主战场,计划逐项推进。</li>
      <li><strong>Specs</strong> —— 规范台账,Agent 的长期记忆与验收标准。</li>
      <li><strong>Relay</strong> —— 多 Agent 编排,任务接力。</li>
      <li><strong>Wiki</strong> —— 项目知识库沉淀。</li>
    </ul>
    <template #visual>
      <div class="shot-pair">
        <img src="/v05/automusk-workspace.png" alt="AutoMusk Web 工作区" />
        <img src="/v05/automusk-plans.png" alt="AutoMusk 计划视图" />
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">为什么 AutoMusk 重要</h2>
  <div class="features-grid">
    <FeatureCard icon="🧩" title="AutoPlan 方法论" description="spec-driven 的串行 Agent:先规划、再执行、后复审,结构化推进复杂任务。" color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🦾" title="自托管" description="用 Auto 写、跑在 AutoVM 上 —— Agent 开发本身就是 Auto 的吃狗粮现场。" color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="🔐" title="aaid 加持" description="密钥托管、并发仲裁、模型路由与用量统计,全部由 AI Daemon 承担。" color="rgba(99, 102, 241, 0.15)" />
    <FeatureCard icon="⚙️" title="统一配置" description="Roles、Skills、Modes 在 auto-os-config 中按 .at 形状编辑,零前端代码。" color="rgba(20, 184, 166, 0.15)" />
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
.shot-stack {
  width: 100%;
  max-width: 480px;
}

.shot-main {
  width: 100%;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
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
</style>
