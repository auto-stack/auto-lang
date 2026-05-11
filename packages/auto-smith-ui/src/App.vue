<template>
  <div class="autosmith-app">
    <nav class="view-rail">
      <div class="rail-brand">
        <Hammer :size="20" />
        <span class="brand-text">AutoSmith</span>
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
        <span class="version">v0.1.0</span>
      </div>
    </nav>
    <main class="view-main">
      <ForgeView v-if="currentView === 'forge'" />
      <LedgerView v-else-if="currentView === 'ledger'" />
      <RelayView v-else-if="currentView === 'relay'" />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Hammer, MessageSquare, BookOpen, GitBranch } from 'lucide-vue-next'
import ForgeView from './views/ForgeView.vue'
import LedgerView from './views/LedgerView.vue'
import RelayView from './views/RelayView.vue'

const tabs: { id: 'forge' | 'ledger' | 'relay'; label: string; icon: unknown }[] = [
  { id: 'forge', label: 'Forge', icon: MessageSquare },
  { id: 'ledger', label: 'Ledger', icon: BookOpen },
  { id: 'relay', label: 'Relay', icon: GitBranch },
]

const currentView = ref<'forge' | 'ledger' | 'relay'>('forge')
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

.autosmith-app {
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
  color: #fab387;
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
  background: #fab38722;
  color: #fab387;
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
