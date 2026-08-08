<!--
  ThemeToggle.vue — handmade escape-hatch component (028-dom-escape pattern).
  The AutoUI vue generator emits <html class="dark"> by default (fixed dark).
  This component toggles the `dark` class on <html> at runtime for a real
  light/dark switch, and persists the choice to localStorage.

  Referenced from .at views via the `theme-toggle {}` tag (auto-mapped to
  <ThemeToggle>, see vue.rs:4412). Discovered/copied by prepare_vue_sources
  from vue/src/components/ThemeToggle.vue (vue.rs:2429-2438).
-->
<template>
  <button
    type="button"
    class="theme-toggle-btn"
    :aria-label="isDark ? 'Switch to light theme' : 'Switch to dark theme'"
    :title="isDark ? 'Light mode' : 'Dark mode'"
    @click="toggle"
  >
    <span class="theme-icon">{{ isDark ? '☀️' : '🌙' }}</span>
  </button>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

const STORAGE_KEY = 'br-theme'
const isDark = ref(true)

function applyTheme(dark: boolean) {
  const html = document.documentElement
  if (dark) {
    html.classList.add('dark')
  } else {
    html.classList.remove('dark')
  }
}

function toggle() {
  isDark.value = !isDark.value
  applyTheme(isDark.value)
  try {
    localStorage.setItem(STORAGE_KEY, isDark.value ? 'dark' : 'light')
  } catch {
    /* ignore storage errors */
  }
}

onMounted(() => {
  // Default to dark (matches the generator's <html class="dark">), but honour
  // any previously saved preference so the toggle feels sticky.
  let saved: string | null = null
  try {
    saved = localStorage.getItem(STORAGE_KEY)
  } catch {
    /* ignore */
  }
  if (saved === 'light') {
    isDark.value = false
  } else if (saved === 'dark') {
    isDark.value = true
  } else {
    isDark.value = document.documentElement.classList.contains('dark')
  }
  applyTheme(isDark.value)
})
</script>

<style scoped>
.theme-toggle-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--foreground);
  cursor: pointer;
  transition: background 0.15s ease, border-color 0.15s ease;
}
.theme-toggle-btn:hover {
  background: var(--accent);
}
.theme-icon {
  font-size: 16px;
  line-height: 1;
}
</style>
