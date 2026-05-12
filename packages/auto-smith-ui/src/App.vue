<template>
  <div class="autoforge-app">
    <nav class="view-rail">
      <div class="rail-brand">
        <Flame :size="20" />
        <span class="brand-text">AutoForge</span>
      </div>
      <div class="rail-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="rail-tab"
          :class="{ active: currentView === tab.id }"
          @click="currentView = tab.id"
        >
          <component :is="tab.icon" :size="18" />
          <span class="tab-label">{{ tab.label }}</span>
        </button>
      </div>
      <div class="rail-footer">
        <button class="theme-toggle" @click="cycleTheme" title="Toggle theme">
          <Sun v-if="mode === 'light'" :size="16" />
          <Moon v-else-if="mode === 'dark'" :size="16" />
          <Monitor v-else :size="16" />
        </button>
        <span class="version">炼器房 v0.1.0</span>
      </div>
    </nav>
    <main class="view-main">
      <FurnaceView v-if="currentView === 'furnace'" />
      <JadesView v-else-if="currentView === 'jades'" />
      <OrderView v-else-if="currentView === 'order'" />
      <StreamingDemoView v-else-if="currentView === 'demo'" />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Flame, MessageSquare, Scroll, Orbit, Sun, Moon, Monitor, Sparkles } from 'lucide-vue-next'
import { useTheme } from '@/composables/useTheme'
import FurnaceView from './views/FurnaceView.vue'
import JadesView from './views/JadesView.vue'
import OrderView from './views/OrderView.vue'
import StreamingDemoView from './views/StreamingDemoView.vue'

const { mode, cycle: cycleTheme } = useTheme()

const tabs: { id: 'furnace' | 'jades' | 'order' | 'demo'; label: string; icon: unknown }[] = [
  { id: 'furnace', label: '丹炉', icon: MessageSquare },
  { id: 'jades', label: '玉简', icon: Scroll },
  { id: 'order', label: '法阵', icon: Orbit },
  { id: 'demo', label: '演示', icon: Sparkles },
]

const currentView = ref<'furnace' | 'jades' | 'order' | 'demo'>('furnace')
</script>

<style>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html, body, #app {
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
  background: var(--af-bg);
  color: var(--af-fg);
}

.autoforge-app {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.view-rail {
  width: 64px;
  display: flex;
  flex-direction: column;
  align-items: center;
  background: var(--af-card);
  border-right: 1px solid var(--af-border);
  padding: 0.75rem 0;
  flex-shrink: 0;
}

.rail-brand {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.35rem;
  color: var(--af-primary);
  margin-bottom: 1.5rem;
}

.brand-text {
  font-size: 0.6rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.rail-tabs {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  flex: 1;
}

.rail-tab {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
  width: 48px;
  padding: 0.5rem 0;
  background: transparent;
  border: none;
  border-radius: 8px;
  color: var(--af-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.rail-tab:hover {
  background: var(--af-secondary);
  color: var(--af-fg);
}

.rail-tab.active {
  background: var(--af-primary-soft);
  color: var(--af-primary);
}

.tab-label {
  font-size: 0.6rem;
  font-weight: 500;
}

.rail-footer {
  margin-top: auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  color: var(--af-muted);
}

.theme-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: transparent;
  border: 1px solid var(--af-border);
  border-radius: 6px;
  color: var(--af-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.theme-toggle:hover {
  background: var(--af-secondary);
  color: var(--af-fg);
}

.version {
  font-size: 0.6rem;
}

.view-main {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
