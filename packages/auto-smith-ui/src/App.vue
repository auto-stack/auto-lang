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
        <span class="version">炼器房 v0.1.0</span>
      </div>
    </nav>
    <main class="view-main">
      <FurnaceView v-if="currentView === 'furnace'" />
      <JadeTabsView v-else-if="currentView === 'jade-tabs'" />
      <ArrayView v-else-if="currentView === 'array'" />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Flame, MessageSquare, Scroll, Orbit } from 'lucide-vue-next'
import FurnaceView from './views/FurnaceView.vue'
import JadeTabsView from './views/JadeTabsView.vue'
import ArrayView from './views/ArrayView.vue'

const tabs: { id: 'furnace' | 'jade-tabs' | 'array'; label: string; icon: unknown }[] = [
  { id: 'furnace', label: '熔炉', icon: MessageSquare },
  { id: 'jade-tabs', label: '玉简', icon: Scroll },
  { id: 'array', label: '法阵', icon: Orbit },
]

const currentView = ref<'furnace' | 'jade-tabs' | 'array'>('furnace')
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
  background: #0f0f14;
  color: #cdd6f4;
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
  background: #181825;
  border-right: 1px solid #313244;
  padding: 0.75rem 0;
  flex-shrink: 0;
}

.rail-brand {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.35rem;
  color: #f38ba8;
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
  color: #6c7086;
  cursor: pointer;
  transition: all 0.15s;
}

.rail-tab:hover {
  background: #313244;
  color: #cdd6f4;
}

.rail-tab.active {
  background: #f38ba822;
  color: #f38ba8;
}

.tab-label {
  font-size: 0.6rem;
  font-weight: 500;
}

.rail-footer {
  margin-top: auto;
  color: #45475a;
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
