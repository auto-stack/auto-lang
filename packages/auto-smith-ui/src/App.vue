<template>
  <div class="autoforge-app">
    <nav class="view-rail">
      <div class="rail-brand">
        <Flame :size="18" />
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
          <component :is="tab.icon" :size="16" />
          <span class="tab-label">{{ tab.label }}</span>
        </button>
      </div>
      <div class="rail-footer">
        <button class="theme-toggle" @click="cycleTheme" title="Toggle theme">
          <Sun v-if="mode === 'light'" :size="14" />
          <Moon v-else-if="mode === 'dark'" :size="14" />
          <Monitor v-else :size="14" />
        </button>
        <span class="version">v0.1.0</span>
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
  { id: 'furnace', label: 'Furnace', icon: MessageSquare },
  { id: 'jades', label: 'Jades', icon: Scroll },
  { id: 'order', label: 'Order', icon: Orbit },
  { id: 'demo', label: 'Demo', icon: Sparkles },
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
  width: 180px;
  display: flex;
  flex-direction: column;
  background: hsl(var(--secondary));
  border-right: 1px solid var(--af-border);
  padding: 1rem 0;
  flex-shrink: 0;
}

.rail-brand {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--af-fg);
  padding: 0 1rem;
  margin-bottom: 1.5rem;
}

.brand-text {
  font-size: 0.85rem;
  font-weight: 600;
}

.rail-tabs {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  flex: 1;
  padding: 0 0.5rem;
}

.rail-tab {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  width: 100%;
  padding: 0.5rem 0.6rem;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: var(--af-muted);
  cursor: pointer;
  transition: all 0.15s;
  font-size: 0.8rem;
}

.rail-tab:hover {
  background: hsl(var(--muted-foreground) / 0.06);
  color: var(--af-fg);
}

.rail-tab.active {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
  font-weight: 500;
}

.tab-label {
  font-size: 0.8rem;
}

.rail-footer {
  margin-top: auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 1rem;
  color: var(--af-muted);
}

.theme-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: var(--af-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.theme-toggle:hover {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
}

.version {
  font-size: 0.7rem;
}

.view-main {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
