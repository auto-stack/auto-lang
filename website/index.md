---
layout: home
---

<script setup>
import { onMounted } from 'vue'
import HomeHero from './.vitepress/theme/components/HomeHero.vue'
import FeatureCard from './.vitepress/theme/components/FeatureCard.vue'
const icons = ['🎯', '⚡', '🔮', '🛡️', '🖥️', '🤖']
onMounted(() => {
  if (sessionStorage.getItem('auto-lang-checked')) return;
  sessionStorage.setItem('auto-lang-checked', '1');
  if (navigator.language.toLowerCase().startsWith('zh')) {
    window.location.replace('/zh/');
  }
});
</script>

<HomeHero
  badge="v0.3 is now available"
  title=": Language for AI &amp; OS"
  description="A modern programming language that transpiles to C, Rust, TypeScript, and Python. Featuring actor concurrency, compile-time metaprogramming, and zero-cost abstractions."
  primary-text="Get Started"
  primary-link="/docs/"
  secondary-text="Try Online"
  secondary-link="/playground"
/>

<div class="features-section">
  <h2 class="section-title">Why Auto?</h2>
  <div class="features-grid">
    <FeatureCard icon="🎯" title="Multi-Target Transpiler" description="Write once, run anywhere. Auto transpiles to C, Rust, TypeScript, and Python with zero-cost abstractions." color="rgba(239, 68, 68, 0.15)" link="/docs/features/multi-target-transpiler" />
    <FeatureCard icon="🖥️" title="AutoVM Interpreter" description="Dedicated VM with AOT/JIT compilation, hot reloading, and cross-platform support from desktop to embedded." color="rgba(20, 184, 166, 0.15)" link="/docs/features/autovm-interpreter" />
    <FeatureCard icon="🔮" title="Comptime Metaprogramming" description="Execute code at compile time. Generate code, validate invariants, and optimize without macros." color="rgba(168, 85, 247, 0.15)" link="/docs/features/comptime-metaprogramming" />
    <FeatureCard icon="🛡️" title="Memory Safety" description="Ownership system inspired by Rust, with smart casts and flow typing for ergonomic safe code." color="rgba(59, 130, 246, 0.15)" link="/docs/features/memory-safety" />
    <FeatureCard icon="⚡" title="Actor Concurrency" description="Built on the Actor model with async ~T types. Write concurrent code that is safe by design." color="rgba(245, 158, 11, 0.15)" link="/docs/features/actor-concurrency" />
    <FeatureCard icon="🤖" title="AI-Native Design" description="First-class support for AI workloads with node-based dataflow and embedded model inference." color="rgba(6, 182, 212, 0.15)" link="/docs/features/ai-native-design" />
  </div>
</div>

<div class="cta-section">
  <h2 class="section-title">Ready to try Auto?</h2>
  <p class="section-desc">Jump into the interactive playground or read the tutorial to learn Auto from the ground up.</p>
  <div class="cta-actions">
    <a href="/playground" class="cta-btn cta-primary">Open Playground</a>
    <a href="/books/tapl/" class="cta-btn cta-secondary">Read the Tutorial</a>
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
