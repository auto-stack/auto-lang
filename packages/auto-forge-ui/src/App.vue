<template>
  <div class="autoforge-app">
    <nav class="view-rail">
      <div class="rail-brand">
        <Flame :size="18" />
        <span class="brand-text">AutoForge</span>
        <span class="version">v0.1.0</span>
      </div>
      <div class="rail-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="rail-tab"
          :class="{ active: currentView === tab.id }"
          @click="currentView = tab.id"
        >
          <component :is="tab.icon" :size="16" class="tab-icon" />
          <span class="tab-label">{{ tab.label }}</span>
        </button>
      </div>
      <div class="rail-footer">
        <!-- Accent color picker -->
        <div ref="accentPickerRef" class="accent-picker">
          <button
            class="accent-toggle"
            :class="{ open: accentOpen }"
            :style="{ color: accentDotColor }"
            @click="accentOpen = !accentOpen"
            title="Accent color"
          >
            <Palette :size="14" />
          </button>
          <transition name="fade">
            <div v-if="accentOpen" class="accent-menu">
              <div class="accent-menu-title">Accent</div>
              <div class="accent-swatches">
                <button
                  v-for="opt in accentOptions"
                  :key="opt.name"
                  class="accent-swatch"
                  :class="{ active: accentCurrent === opt.name }"
                  :style="{ background: opt.brand1 }"
                  :title="opt.label"
                  @click="setAccent(opt.name); accentOpen = false"
                >
                  <Check v-if="accentCurrent === opt.name" :size="12" />
                </button>
              </div>
            </div>
          </transition>
        </div>

        <!-- Theme mode picker -->
        <div ref="themePickerRef" class="theme-picker">
          <button
            class="theme-toggle"
            :class="{ open: themeOpen }"
            @click="themeOpen = !themeOpen"
            title="Theme"
          >
            <Sun v-if="mode === 'light'" :size="14" />
            <Moon v-else-if="mode === 'dark'" :size="14" />
            <Monitor v-else :size="14" />
          </button>
          <transition name="fade">
            <div v-if="themeOpen" class="theme-menu">
              <button
                v-for="opt in themeOptions"
                :key="opt.value"
                class="theme-option"
                :class="{ active: mode === opt.value }"
                @click="setMode(opt.value); themeOpen = false"
              >
                <component :is="opt.icon" :size="14" />
                <span>{{ opt.label }}</span>
                <Check v-if="mode === opt.value" :size="12" class="check" />
              </button>
            </div>
          </transition>
        </div>

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
import { ref, onMounted, onUnmounted, computed } from 'vue'
import {
  Flame, MessageSquare, Scroll, Orbit,
  Sun, Moon, Monitor, Sparkles, Check, Palette,
} from 'lucide-vue-next'
import { useTheme } from '@/composables/useTheme'
import { useAccentColor, ACCENT_OPTIONS } from '@/composables/useAccentColor'
import FurnaceView from './views/FurnaceView.vue'
import JadesView from './views/JadesView.vue'
import OrderView from './views/OrderView.vue'
import StreamingDemoView from './views/StreamingDemoView.vue'

const { mode, setMode } = useTheme()
const { current: accentCurrent, setAccent, options: accentOptions } = useAccentColor()

const themeOpen = ref(false)
const accentOpen = ref(false)
const themePickerRef = ref<HTMLDivElement>()
const accentPickerRef = ref<HTMLDivElement>()

const accentDotColor = computed(() => {
  const opt = accentOptions.find((o) => o.name === accentCurrent.value)
  return opt?.brand1 ?? '#5558d6'
})

function onDocClick(e: MouseEvent) {
  const target = e.target as Node
  if (themeOpen.value && themePickerRef.value && !themePickerRef.value.contains(target)) {
    themeOpen.value = false
  }
  if (accentOpen.value && accentPickerRef.value && !accentPickerRef.value.contains(target)) {
    accentOpen.value = false
  }
}

onMounted(() => document.addEventListener('click', onDocClick))
onUnmounted(() => document.removeEventListener('click', onDocClick))

const themeOptions = [
  { value: 'light' as const, label: 'Light', icon: Sun },
  { value: 'dark' as const, label: 'Dark', icon: Moon },
  { value: 'auto' as const, label: 'System', icon: Monitor },
]

const tabs: { id: 'furnace' | 'jades' | 'order' | 'demo'; label: string; icon: unknown }[] = [
  { id: 'furnace', label: 'Chat', icon: MessageSquare },
  { id: 'jades', label: 'Specs', icon: Scroll },
  { id: 'order', label: 'Agents', icon: Orbit },
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
  color: var(--af-primary);
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
  background: hsl(var(--primary) / 0.08);
  color: var(--af-primary);
  font-weight: 500;
}

.rail-tab.active .tab-icon {
  color: var(--af-primary);
  stroke: var(--af-primary);
}

.tab-label {
  font-size: 0.8rem;
}

.rail-footer {
  margin-top: auto;
  display: flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0 1rem;
  color: var(--af-muted);
}

/* ─── Accent Color Picker ─────────────────────────────────────────────────── */

.accent-picker {
  position: relative;
}

.accent-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.accent-toggle:hover,
.accent-toggle.open {
  background: hsl(var(--muted-foreground) / 0.08);
}

.accent-menu {
  position: absolute;
  bottom: calc(100% + 6px);
  left: 0;
  min-width: 140px;
  background: var(--af-card);
  border: 1px solid var(--af-border);
  border-radius: 8px;
  padding: 0.5rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
  z-index: 100;
}

.accent-menu-title {
  font-size: 0.7rem;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--af-muted);
  margin-bottom: 0.4rem;
  padding: 0 0.1rem;
}

.accent-swatches {
  display: flex;
  gap: 0.4rem;
  flex-wrap: wrap;
}

.accent-swatch {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  transition: transform 0.1s, box-shadow 0.15s;
  padding: 0;
}

.accent-swatch:hover {
  transform: scale(1.1);
}

.accent-swatch.active {
  box-shadow: 0 0 0 2px var(--af-bg), 0 0 0 4px var(--af-primary);
}

/* ─── Theme Mode Picker ───────────────────────────────────────────────────── */

.theme-picker {
  position: relative;
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

.theme-toggle:hover,
.theme-toggle.open {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
}

.theme-menu {
  position: absolute;
  bottom: calc(100% + 6px);
  left: 0;
  min-width: 130px;
  background: var(--af-card);
  border: 1px solid var(--af-border);
  border-radius: 8px;
  padding: 0.35rem;
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
  z-index: 100;
}

.theme-option {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.35rem 0.5rem;
  border: none;
  border-radius: 5px;
  background: transparent;
  color: var(--af-fg);
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.1s;
  text-align: left;
}

.theme-option:hover {
  background: hsl(var(--muted-foreground) / 0.06);
}

.theme-option.active {
  color: var(--af-primary);
  font-weight: 500;
}

.theme-option .check {
  margin-left: auto;
  color: var(--af-primary);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(4px);
}

.version {
  font-size: 0.65rem;
  color: var(--af-muted);
  font-weight: 400;
  margin-left: 0.35rem;
}

.view-main {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
