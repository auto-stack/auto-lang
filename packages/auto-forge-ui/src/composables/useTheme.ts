import { ref, readonly, onMounted } from 'vue'

/* ═════════════════════════════════════════════════════════════════════════════
   useTheme
   ────────────────────────────────────────────────────────────────────────────
   Syncs with the VitePress website's theme preference via localStorage
   key `vitepress-theme-appearance`.  Values: "auto" | "dark" | "light".
   Toggles the `.dark` class on <html> accordingly.
   ═════════════════════════════════════════════════════════════════════════════ */

const STORAGE_KEY = 'vitepress-theme-appearance'

type ThemeMode = 'auto' | 'dark' | 'light'

const _mode = ref<ThemeMode>('auto')

function systemPrefersDark(): boolean {
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

function applyMode(mode: ThemeMode) {
  const html = document.documentElement
  const isDark = mode === 'dark' || (mode === 'auto' && systemPrefersDark())
  if (isDark) {
    html.classList.add('dark')
  } else {
    html.classList.remove('dark')
  }
}

export function useTheme() {
  const mode = readonly(_mode)

  function setMode(next: ThemeMode) {
    _mode.value = next
    localStorage.setItem(STORAGE_KEY, next)
    applyMode(next)
  }

  function cycle() {
    const order: ThemeMode[] = ['light', 'dark', 'auto']
    const idx = order.indexOf(_mode.value)
    setMode(order[(idx + 1) % order.length])
  }

  function init() {
    const stored = localStorage.getItem(STORAGE_KEY) as ThemeMode | null
    const initial: ThemeMode = stored ?? 'auto'
    _mode.value = initial
    applyMode(initial)

    // React to system changes when in auto mode
    const mql = window.matchMedia('(prefers-color-scheme: dark)')
    mql.addEventListener?.('change', () => {
      if (_mode.value === 'auto') applyMode('auto')
    })
  }

  onMounted(init)

  return {
    mode,
    setMode,
    cycle,
  }
}
