---
layout: home

hero:
  name: "Auto"
  text: "系统与 AI 语言"
  tagline: 多目标转译器 · Actor 并发 · 编译期元编程
  actions:
    - theme: brand
      text: 快速开始
      link: /zh/docs/getting-started
    - theme: alt
      text: 在线体验
      link: /zh/playground
    - theme: alt
      text: GitHub
      link: https://github.com/autostack/auto-lang

features:
  - icon: 🎯
    title: 多目标转译器
    details: 一次编写，到处运行。Auto 可转译为 C、Rust、TypeScript 和 Python，实现零成本抽象。
  - icon: ⚡
    title: Actor 并发
    details: 基于 Actor 模型，配合 async ~T 类型。编写天生安全的并发代码。
  - icon: 🔮
    title: 编译期元编程
    details: 在编译期执行代码。生成代码、验证不变式、优化性能，无需宏。
  - icon: 🛡️
    title: 内存安全
    details: 受 Rust 启发的所有权系统，配合智能转换和流类型，提供符合人体工学的安全代码。
  - icon: 📦
    title: 现代工具链
    details: 内置包管理器、LSP 支持、代码格式化器，以及与现有生态系统的无缝 FFI。
  - icon: 🤖
    title: AI 原生设计
    details: 对 AI 工作负载的一流支持，包括基于节点的数据流和嵌入式模型推理。
---

<script setup>
import HomeHero from '../.vitepress/theme/components/HomeHero.vue'
import FeatureCard from '../.vitepress/theme/components/FeatureCard.vue'
import { Zap, Shield, Code2, Box, Cpu, Bot } from 'lucide-vue-next'
</script>

<HomeHero />

<div class="features-section">
  <h2 class="section-title">为什么选择 Auto？</h2>
  <div class="features-grid">
    <FeatureCard :icon="Code2" title="多目标转译器" description="一次编写，到处运行。Auto 可转译为 C、Rust、TypeScript 和 Python，实现零成本抽象。" />
    <FeatureCard :icon="Zap" title="Actor 并发" description="基于 Actor 模型，配合 async ~T 类型。编写天生安全的并发代码。" />
    <FeatureCard :icon="Cpu" title="编译期元编程" description="在编译期执行代码。生成代码、验证不变式、优化性能，无需宏。" />
    <FeatureCard :icon="Shield" title="内存安全" description="受 Rust 启发的所有权系统，配合智能转换和流类型，提供符合人体工学的安全代码。" />
    <FeatureCard :icon="Box" title="现代工具链" description="内置包管理器、LSP 支持、代码格式化器，以及与现有生态系统的无缝 FFI。" />
    <FeatureCard :icon="Bot" title="AI 原生设计" description="对 AI 工作负载的一流支持，包括基于节点的数据流和嵌入式模型推理。" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">准备好尝试 Auto 了吗？</h2>
  <p class="section-desc">进入交互式 Playground，或阅读书籍从零开始学习 Auto。</p>
  <div class="cta-actions">
    <a href="/zh/playground" class="cta-btn cta-primary">打开 Playground</a>
    <a href="/zh/books/tapl/" class="cta-btn cta-secondary">阅读书籍</a>
  </div>
</div>

<style scoped>
.features-section {
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
</style>
