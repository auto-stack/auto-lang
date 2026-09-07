---
layout: home
---

<script setup>
import AIHero from '../../.vitepress/theme/components/AIHero.vue'
import FeatureCard from '../../.vitepress/theme/components/FeatureCard.vue'
import StatCard from '../../.vitepress/theme/components/StatCard.vue'
import ShowcaseSection from '../../.vitepress/theme/components/ShowcaseSection.vue'
</script>

<div class="landing-page" style="--page-accent-1: #ec4899; --page-accent-2: #a855f7">

<AIHero
  badge="AutoOS Flagship App · 100% Auto"
  title="AutoMusk Coding Agent"
  description="A general-purpose coding agent driven by the AutoPlan mode: planning, execution, and review — the whole workflow, fully structured. Implemented in the Auto language itself — the best proof that the platform can build agents."
  primary-text="Try the Playground"
  primary-link="/playground"
  secondary-text="AutoAI Architecture"
  secondary-link="/ai"
/>

<div class="stats-section">
  <h2 class="section-title">AutoMusk at a Glance</h2>
  <div class="stats-grid">
    <StatCard value="AutoPlan" label="Planning Mode" description="A structured coding loop: plan → execute → review." color="#ec4899" />
    <StatCard value=".at" label="Single Frontend Source" description="Five views generated from .at sources via auto build, with 148 parity checks all equal." color="#a855f7" />
    <StatCard value="Multi-Model" label="Provider-Agnostic" description="Routes to any model — OpenAI, Anthropic, Zhipu, and more — via the aaid daemon." color="#6366f1" />
    <StatCard value="2" label="Shapes" description="An iced desktop app and a Web workbench." color="#14b8a6" />
  </div>
</div>

<div class="showcase-wrapper">
  <ShowcaseSection
    title="The Desktop Workbench"
    description="Sessions, plans, specs, and the knowledge base in one rail; Block cards make every step of the agent's output structured and visible."
    badge="iced Desktop"
  >
    <ul>
      <li><strong>AutoPlan pipeline</strong> — plan → execute → review, with the spec as the single source of truth, saving tokens.</li>
      <li><strong>Block card rendering</strong> — agent output appears as blocks, not a wall of text.</li>
      <li><strong>Workspace awareness</strong> — pick a working directory and the agent follows the repo's conventions.</li>
    </ul>
    <template #visual>
      <div class="shot-stack">
        <img src="/v05/automusk-app.png" alt="AutoMusk desktop main interface" class="shot-main" />
      </div>
    </template>
  </ShowcaseSection>

  <ShowcaseSection
    title="The Web Workbench: Forge / Specs / Relay / Wiki"
    description="The five frontend views (Login/Chats/Plans/Specs/Wiki) come from a single .at source, built into a Vue project via auto build — .at is the only source of truth, with 148 parity checks all equal across both ends."
    badge="Web"
    reverse
  >
    <ul>
      <li><strong>Forge</strong> — the main arena for working with your agent, advancing plans item by item.</li>
      <li><strong>Specs</strong> — the spec ledger: the agent's long-term memory and acceptance criteria.</li>
      <li><strong>Relay</strong> — multi-agent orchestration with task handoffs.</li>
      <li><strong>Wiki</strong> — your project's accumulated knowledge base.</li>
    </ul>
    <template #visual>
      <div class="shot-pair">
        <img src="/v05/automusk-workspace.png" alt="AutoMusk web workspace" />
        <img src="/v05/automusk-plans.png" alt="AutoMusk plans view" />
      </div>
    </template>
  </ShowcaseSection>
</div>

<div class="features-section">
  <h2 class="section-title">Why AutoMusk Matters</h2>
  <div class="features-grid">
    <FeatureCard icon="🧩" title="The AutoPlan Methodology" description="A spec-driven serial agent: plan first, execute next, review last — structured progress on complex tasks." color="rgba(236, 72, 153, 0.15)" />
    <FeatureCard icon="🦾" title="Self-Hosted" description="Written in Auto, running on the AutoVM — agent development is Auto's own dogfooding ground." color="rgba(168, 85, 247, 0.15)" />
    <FeatureCard icon="🔐" title="Backed by aaid" description="Key custody, concurrency arbitration, model routing, and usage accounting — all handled by the AI daemon." color="rgba(99, 102, 241, 0.15)" />
    <FeatureCard icon="⚙️" title="Unified Config" description="Roles, Skills, and Modes edited in auto-os-config by .at shape, with zero frontend code." color="rgba(20, 184, 166, 0.15)" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Keep Exploring</h2>
  <div class="cta-actions">
    <a href="/v05/" class="cta-btn cta-primary">Back to v0.5 Release Highlights</a>
    <a href="/apps" class="cta-btn cta-secondary">View All Apps</a>
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
