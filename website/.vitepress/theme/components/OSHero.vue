<template>
  <section class="os-hero">
    <div class="os-hero-bg">
      <div class="gradient-orb orb-kernel" />
      <div class="gradient-orb orb-io" />
      <div class="grid-pattern" />
    </div>
    <div class="os-hero-content">
      <div class="os-hero-left">
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
            <Layers class="icon" :size="16" />
          </a>
        </div>
      </div>
      <div class="os-hero-right">
        <div class="layer-stack">
          <div class="layer layer-app">
            <div class="layer-icon"><Monitor :size="18" /></div>
            <div class="layer-info">
              <span class="layer-name">Applications</span>
              <span class="layer-desc">AutoUI / AutoShell</span>
            </div>
          </div>
          <div class="layer-connector" />
          <div class="layer layer-lang">
            <div class="layer-icon"><Code :size="18" /></div>
            <div class="layer-info">
              <span class="layer-name">Auto Language</span>
              <span class="layer-desc">Syntax / Types / Spec</span>
            </div>
          </div>
          <div class="layer-connector" />
          <div class="layer layer-runtime">
            <div class="layer-icon"><Cpu :size="18" /></div>
            <div class="layer-info">
              <span class="layer-name">Auto Runtime</span>
              <span class="layer-desc">AutoVM / Transpilers / Atom</span>
            </div>
          </div>
          <div class="layer-connector" />
          <div class="layer layer-hw">
            <div class="layer-icon"><HardDrive :size="18" /></div>
            <div class="layer-info">
              <span class="layer-name">Hardware</span>
              <span class="layer-desc">MCU / SOC / Desktop / Cloud</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ArrowRight, Layers, Monitor, Code, Cpu, HardDrive } from 'lucide-vue-next'

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
  badge: 'Language as OS',
  title: ': The System in Syntax',
  description: 'Auto blurs the line between language and operating system. Compile-time execution, multi-target transpilers, and a unified runtime from MCU to cloud.',
  primaryText: 'Read the Docs',
  primaryLink: '/docs/os',
  secondaryText: 'Explore LaOS',
  secondaryLink: '/docs/os#laos',
})
</script>

<style scoped>
.os-hero {
  position: relative;
  min-height: 60vh;
  display: flex;
  align-items: center;
  overflow: hidden;
  padding: 2rem;
}
.os-hero-bg {
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
.orb-kernel {
  width: 500px;
  height: 500px;
  background: linear-gradient(135deg, #14b8a6, #3b82f6);
  top: -180px;
  right: 0;
}
.orb-io {
  width: 350px;
  height: 350px;
  background: linear-gradient(135deg, #6366f1, #14b8a6);
  bottom: -120px;
  left: -60px;
}
.grid-pattern {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(20, 184, 166, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(20, 184, 166, 0.03) 1px, transparent 1px);
  background-size: 40px 40px;
}
.os-hero-content {
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
.os-hero-left {
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
  background: rgba(20, 184, 166, 0.1);
  border: 1px solid rgba(20, 184, 166, 0.2);
  color: #14b8a6;
  font-size: 0.875rem;
  font-weight: 500;
  width: fit-content;
}
.badge-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #14b8a6;
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
  background: linear-gradient(120deg, #14b8a6 30%, #3b82f6 70%);
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
  background: linear-gradient(135deg, #14b8a6 0%, #3b82f6 100%);
  color: white;
  box-shadow: 0 4px 14px rgba(20, 184, 166, 0.3);
}
.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(20, 184, 166, 0.4);
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
.os-hero-right {
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  min-height: 320px;
}
.layer-stack {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0;
  width: 100%;
  max-width: 380px;
}
.layer {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem 1.25rem;
  border-radius: var(--radius);
  border: 1px solid;
  background: hsl(var(--card) / 0.6);
  backdrop-filter: blur(8px);
  transition: transform 0.3s ease;
}
.layer:hover {
  transform: translateX(6px);
}
.layer-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.layer-info {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}
.layer-name {
  font-weight: 700;
  font-size: 0.95rem;
  color: hsl(var(--foreground));
}
.layer-desc {
  font-size: 0.8rem;
  color: hsl(var(--muted-foreground));
}
.layer-app {
  border-color: rgba(245, 158, 11, 0.3);
}
.layer-app .layer-icon {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
}
.layer-lang {
  border-color: rgba(236, 72, 153, 0.3);
}
.layer-lang .layer-icon {
  background: rgba(236, 72, 153, 0.15);
  color: #ec4899;
}
.layer-runtime {
  border-color: rgba(59, 130, 246, 0.3);
}
.layer-runtime .layer-icon {
  background: rgba(59, 130, 246, 0.15);
  color: #3b82f6;
}
.layer-hw {
  border-color: rgba(20, 184, 166, 0.3);
}
.layer-hw .layer-icon {
  background: rgba(20, 184, 166, 0.15);
  color: #14b8a6;
}
.layer-connector {
  width: 2px;
  height: 20px;
  background: linear-gradient(180deg, rgba(99,102,241,0.2), rgba(20,184,166,0.2));
}
@media (max-width: 960px) {
  .os-hero-content {
    grid-template-columns: 1fr;
    gap: 3rem;
    text-align: center;
  }
  .os-hero-left {
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
  .os-hero {
    padding: 1rem;
  }
  .title {
    font-size: 2.5rem;
  }
}
</style>
