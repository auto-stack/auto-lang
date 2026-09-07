<template>
  <section class="hero">
    <div class="hero-background">
      <div class="gradient-orb orb-1" />
      <div class="gradient-orb orb-2" />
      <div class="grid-pattern" />
    </div>
    <div class="hero-content">
      <div class="badge">
        <span class="badge-dot" />
        {{ badge }}
      </div>
      <h1 class="title">
        <span class="gradient-text">Auto</span>
        <span class="subtitle"
          ><template v-for="(part, i) in titleParts" :key="i"
            ><svg
              v-if="i > 0"
              class="title-x"
              viewBox="0 0 24 24"
              role="img"
              aria-label="×"
            >
              <defs>
                <linearGradient
                  :id="`auto-hero-x-grad-${i}`"
                  x1="0"
                  y1="0"
                  x2="24"
                  y2="24"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#6366f1" />
                  <stop offset="1" stop-color="#a855f7" />
                </linearGradient>
              </defs>
              <path
                d="M18 6 6 18M6 6l12 12"
                :stroke="`url(#auto-hero-x-grad-${i})`"
                stroke-width="4.4"
                stroke-linecap="round"
                fill="none"
              /> </svg
            >{{ part }}</template
          ></span
        >
      </h1>
      <p class="description" v-html="description" />
      <div class="actions">
        <a :href="primaryLink" class="btn btn-primary">
          {{ primaryText }}
          <ArrowRight class="icon" :size="16" />
        </a>
        <a :href="secondaryLink" class="btn btn-secondary">
          {{ secondaryText }}
          <Play class="icon" :size="16" />
        </a>
      </div>
      <div class="code-window">
        <div class="code-header">
          <div class="code-dots">
            <span />
            <span />
            <span />
          </div>
          <span class="code-title">hello.at</span>
        </div>
        <pre class="code-body"><code><span class="keyword">fn</span> <span class="function">main</span>() {
    <span class="keyword">let</span> msg = <span class="string">"Hello, Auto!"</span>
    <span class="function">println</span>(msg)
}</code></pre>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ArrowRight, Play } from 'lucide-vue-next'

interface Props {
  badge?: string
  title?: string
  description?: string
  primaryText?: string
  primaryLink?: string
  secondaryText?: string
  secondaryLink?: string
}

const props = withDefaults(defineProps<Props>(), {
  badge: 'v0.3 is now available',
  title: ': Language for AI & OS',
  description: 'A modern programming language that transpiles to C, Rust, TypeScript, and Python. Featuring actor concurrency, compile-time metaprogramming, and zero-cost abstractions.',
  primaryText: 'Get Started',
  primaryLink: '/docs/',
  secondaryText: 'Try Online',
  secondaryLink: '/playground',
})

// 标题里的 × 渲染为渐变描边 SVG 叉(参考 Python 页样式,主色调 Auto 紫)
const titleParts = computed(() => props.title.split('×'))
</script>

<style scoped>
.hero {
  position: relative;
  min-height: 60vh;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  padding: 2rem;
}

.hero-background {
  position: absolute;
  inset: 0;
  z-index: 0;
}

.gradient-orb {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.4;
}

.orb-1 {
  width: 600px;
  height: 600px;
  background: linear-gradient(135deg, #6366f1, #a855f7);
  top: -200px;
  right: -100px;
}

.orb-2 {
  width: 400px;
  height: 400px;
  background: linear-gradient(135deg, #3b82f6, #6366f1);
  bottom: -100px;
  left: -100px;
}

.grid-pattern {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(99, 102, 241, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(99, 102, 241, 0.03) 1px, transparent 1px);
  background-size: 40px 40px;
}

.hero-content {
  position: relative;
  z-index: 1;
  max-width: 800px;
  text-align: center;
}

.badge {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border-radius: 9999px;
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.2);
  color: #6366f1;
  font-size: 0.875rem;
  font-weight: 500;
  margin-bottom: 2rem;
}

.badge-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #6366f1;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.title {
  font-size: clamp(2.5rem, 5vw, 3.75rem);
  font-weight: 800;
  line-height: 1.1;
  letter-spacing: -0.02em;
  color: hsl(var(--foreground));
  margin-bottom: 1.5rem;
}

.gradient-text {
  background: linear-gradient(120deg, #6366f1 30%, #a855f7 70%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.title-x {
  width: 0.66em;
  height: 0.66em;
  display: inline-block;
  vertical-align: 0.05em;
  margin: 0 0.08em;
}

.description {
  font-size: 1.25rem;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
  max-width: 600px;
  margin: 0 auto 2.5rem;
}

.actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
  margin-bottom: 3rem;
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
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  color: white;
  box-shadow: 0 4px 14px rgba(99, 102, 241, 0.3);
}

.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(99, 102, 241, 0.4);
}

.btn-secondary {
  background: hsl(var(--card));
  color: hsl(var(--foreground));
  border: 1px solid hsl(var(--border));
}

.btn-secondary:hover {
  background: hsl(var(--accent));
}

.icon {
  transition: transform 0.2s ease;
}

.btn:hover .icon {
  transform: translateX(2px);
}

.code-window {
  background: #1e1e2e;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  max-width: 500px;
  margin: 0 auto;
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
  font-size: 0.9rem;
  line-height: 1.6;
  color: #cdd6f4;
  overflow-x: auto;
}

.keyword { color: #cba6f7; }
.function { color: #89b4fa; }
.string { color: #a6e3a1; }

@media (max-width: 640px) {
  .hero {
    padding: 1rem;
  }
  .title {
    font-size: 2.5rem;
  }
  .description {
    font-size: 1rem;
  }
}
</style>
