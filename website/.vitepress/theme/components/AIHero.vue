<template>
  <section class="ai-hero">
    <div class="ai-hero-bg">
      <div class="gradient-orb orb-core" />
      <div class="gradient-orb orb-ring" />
      <div class="grid-pattern" />
    </div>
    <div class="ai-hero-content">
      <div class="ai-hero-left">
        <div class="badge">
          <span class="badge-dot" />
          {{ badge }}
        </div>
        <h1 class="title">
          <span class="gradient-text">Auto</span>
          <span class="subtitle">{{ title }}</span>
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
      </div>
      <div class="ai-hero-right">
        <div class="ai-core">
          <div class="core-glow" />
          <div class="core-ring r1" />
          <div class="core-ring r2" />
          <div class="core-ring r3" />
          <div class="core-label">AI</div>
        </div>
        <div class="orbit-tags">
          <div class="orbit-tag tag-by">
            <Zap :size="14" />
            <span>Made BY AI</span>
          </div>
          <div class="orbit-tag tag-for">
            <Brain :size="14" />
            <span>Made FOR AI</span>
          </div>
          <div class="orbit-tag tag-of">
            <Bot :size="14" />
            <span>Made of AI</span>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ArrowRight, Play, Zap, Brain, Bot } from 'lucide-vue-next'

interface Props {
  badge?: string
  title?: string
  description?: string
  primaryText?: string
  primaryLink?: string
  secondaryText?: string
  secondaryLink?: string
}

withDefaults(defineProps<Props>(), {
  badge: 'AI-Native Architecture',
  title: ': Designed for Intelligence',
  description: 'A programming language built with AI, optimized for AI, and composed of AI. From token-efficient syntax to agent-native architecture.',
  primaryText: 'Read the Docs',
  primaryLink: '/docs/ai',
  secondaryText: 'Try Playground',
  secondaryLink: '/playground',
})
</script>

<style scoped>
.ai-hero {
  position: relative;
  min-height: 60vh;
  display: flex;
  align-items: center;
  overflow: hidden;
  padding: 2rem;
}

.ai-hero-bg {
  position: absolute;
  inset: 0;
  z-index: 0;
}

.gradient-orb {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.35;
}

.orb-core {
  width: 500px;
  height: 500px;
  background: linear-gradient(135deg, #ec4899, #a855f7);
  top: -150px;
  right: 5%;
}

.orb-ring {
  width: 350px;
  height: 350px;
  background: linear-gradient(135deg, #6366f1, #3b82f6);
  bottom: -100px;
  left: -80px;
}

.grid-pattern {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(168, 85, 247, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(168, 85, 247, 0.03) 1px, transparent 1px);
  background-size: 40px 40px;
}

.ai-hero-content {
  position: relative;
  z-index: 1;
  max-width: 1200px;
  margin: 0 auto;
  width: 100%;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 4rem;
  align-items: center;
}

.ai-hero-left {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.badge {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border-radius: 9999px;
  background: rgba(236, 72, 153, 0.1);
  border: 1px solid rgba(236, 72, 153, 0.2);
  color: #ec4899;
  font-size: 0.875rem;
  font-weight: 500;
  width: fit-content;
}

.badge-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #ec4899;
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
  margin: 0;
}

.gradient-text {
  background: linear-gradient(120deg, #ec4899 30%, #a855f7 70%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.subtitle {
  display: block;
  margin-top: 0.25rem;
}

.description {
  font-size: 1.15rem;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
  margin: 0;
  max-width: 520px;
}

.actions {
  display: flex;
  gap: 1rem;
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
  background: linear-gradient(135deg, #ec4899 0%, #a855f7 100%);
  color: white;
  box-shadow: 0 4px 14px rgba(236, 72, 153, 0.3);
}

.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(236, 72, 153, 0.4);
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

/* Right side visual */
.ai-hero-right {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2rem;
  position: relative;
  min-height: 320px;
}

.ai-core {
  position: relative;
  width: 200px;
  height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.core-glow {
  position: absolute;
  width: 120px;
  height: 120px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(236, 72, 153, 0.6) 0%, rgba(168, 85, 247, 0.2) 60%, transparent 100%);
  animation: breathe 3s ease-in-out infinite;
}

@keyframes breathe {
  0%, 100% { transform: scale(1); opacity: 0.8; }
  50% { transform: scale(1.15); opacity: 1; }
}

.core-ring {
  position: absolute;
  border-radius: 50%;
  border: 1.5px solid;
}

.core-ring.r1 {
  width: 160px;
  height: 160px;
  border-color: rgba(236, 72, 153, 0.25);
  animation: spin 12s linear infinite;
}

.core-ring.r2 {
  width: 200px;
  height: 200px;
  border-color: rgba(168, 85, 247, 0.2);
  animation: spin 18s linear infinite reverse;
}

.core-ring.r3 {
  width: 240px;
  height: 240px;
  border-color: rgba(99, 102, 241, 0.15);
  animation: spin 24s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.core-label {
  position: relative;
  font-size: 2rem;
  font-weight: 800;
  background: linear-gradient(120deg, #ec4899, #a855f7);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  z-index: 2;
}

.orbit-tags {
  display: flex;
  gap: 0.75rem;
  flex-wrap: wrap;
  justify-content: center;
}

.orbit-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.5rem 1rem;
  border-radius: 9999px;
  font-size: 0.8rem;
  font-weight: 600;
  border: 1px solid;
  backdrop-filter: blur(8px);
}

.orbit-tag.tag-by {
  background: rgba(245, 158, 11, 0.08);
  border-color: rgba(245, 158, 11, 0.25);
  color: #f59e0b;
}

.orbit-tag.tag-for {
  background: rgba(236, 72, 153, 0.08);
  border-color: rgba(236, 72, 153, 0.25);
  color: #ec4899;
}

.orbit-tag.tag-of {
  background: rgba(168, 85, 247, 0.08);
  border-color: rgba(168, 85, 247, 0.25);
  color: #a855f7;
}

@media (max-width: 960px) {
  .ai-hero-content {
    grid-template-columns: 1fr;
    gap: 3rem;
    text-align: center;
  }
  .ai-hero-left {
    align-items: center;
  }
  .description {
    max-width: 100%;
  }
  .actions {
    justify-content: center;
  }
}

@media (max-width: 640px) {
  .ai-hero {
    padding: 1rem;
  }
  .title {
    font-size: 2.5rem;
  }
}
</style>
