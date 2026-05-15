import { ref, onMounted, onUnmounted } from 'vue'

const STORAGE_KEY = 'vitepress-theme-appearance'

const isDark = ref(false)

function readPreference(): boolean {
  const saved = localStorage.getItem(STORAGE_KEY)
  if (saved === 'dark') return true
  if (saved === 'light') return false
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

function applyDark(dark: boolean) {
  if (dark) {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
}

function toggle(): boolean {
  isDark.value = !isDark.value
  applyDark(isDark.value)
  localStorage.setItem(STORAGE_KEY, isDark.value ? 'dark' : 'light')
  return isDark.value
}

// Sync across tabs
function onStorageChange(e: StorageEvent) {
  if (e.key === STORAGE_KEY && e.newValue) {
    isDark.value = e.newValue === 'dark'
    applyDark(isDark.value)
  }
}

export function useDarkMode() {
  onMounted(() => {
    isDark.value = readPreference()
    applyDark(isDark.value)
    window.addEventListener('storage', onStorageChange)
  })

  onUnmounted(() => {
    window.removeEventListener('storage', onStorageChange)
  })

  return { isDark, toggle }
}
