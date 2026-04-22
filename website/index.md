---
layout: home

hero:
  name: "Auto"
  text: "The Language for Systems & AI"
  tagline: Multi-target transpiler · Actor concurrency · Comptime metaprogramming
  actions:
    - theme: brand
      text: Get Started
      link: /docs/getting-started
    - theme: alt
      text: Try Online
      link: /playground
    - theme: alt
      text: View on GitHub
      link: https://github.com/autostack/auto-lang

features:
  - icon: 🎯
    title: Multi-Target Transpiler
    details: Write once, run anywhere. Auto transpiles to C, Rust, TypeScript, and Python with zero-cost abstractions.
  - icon: ⚡
    title: Actor Concurrency
    details: Built on the Actor model with async ~T types. Write concurrent code that is safe by design.
  - icon: 🔮
    title: Comptime Metaprogramming
    details: Execute code at compile time. Generate code, validate invariants, and optimize without macros.
  - icon: 🛡️
    title: Memory Safety
    details: Ownership system inspired by Rust, with smart casts and flow typing for ergonomic safe code.
  - icon: 📦
    title: Modern Tooling
    details: Built-in package manager, LSP support, formatter, and seamless FFI to existing ecosystems.
  - icon: 🤖
    title: AI-Native Design
    details: First-class support for AI workloads with node-based dataflow and embedded model inference.
---

<script setup>
import HomeHero from './.vitepress/theme/components/HomeHero.vue'
import FeatureCard from './.vitepress/theme/components/FeatureCard.vue'
import { Zap, Shield, Code2, Box, Cpu, Bot } from 'lucide-vue-next'
</script>

<HomeHero />

<div class="features-section">
  <h2 class="section-title">Why Auto?</h2>
  <div class="features-grid">
    <FeatureCard :icon="Code2" title="Multi-Target Transpiler" description="Write once, run anywhere. Auto transpiles to C, Rust, TypeScript, and Python with zero-cost abstractions." />
    <FeatureCard :icon="Zap" title="Actor Concurrency" description="Built on the Actor model with async ~T types. Write concurrent code that is safe by design." />
    <FeatureCard :icon="Cpu" title="Comptime Metaprogramming" description="Execute code at compile time. Generate code, validate invariants, and optimize without macros." />
    <FeatureCard :icon="Shield" title="Memory Safety" description="Ownership system inspired by Rust, with smart casts and flow typing for ergonomic safe code." />
    <FeatureCard :icon="Box" title="Modern Tooling" description="Built-in package manager, LSP support, formatter, and seamless FFI to existing ecosystems." />
    <FeatureCard :icon="Bot" title="AI-Native Design" description="First-class support for AI workloads with node-based dataflow and embedded model inference." />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Ready to try Auto?</h2>
  <p class="section-desc">Jump into the interactive playground or read the book to learn Auto from the ground up.</p>
  <div class="cta-actions">
    <a href="/playground" class="cta-btn cta-primary">Open Playground</a>
    <a href="/books/tapl/" class="cta-btn cta-secondary">Read the Book</a>
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
