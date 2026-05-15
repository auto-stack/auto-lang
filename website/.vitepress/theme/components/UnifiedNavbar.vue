<script setup lang="ts">
import { ref } from 'vue'
import { useDarkMode } from '../composables/useDarkMode'

const props = withDefaults(defineProps<{
  showSearch?: boolean
  activeSection?: 'home' | 'gallery' | 'blocks' | 'charts' | 'a2ui'
}>(), {
  showSearch: false,
  activeSection: 'home',
})

const emit = defineEmits<{
  searchClick: []
}>()

const { isDark, toggle } = useDarkMode()
const uiDropdownOpen = ref(false)
const mobileMenuOpen = ref(false)
let closeTimeout: ReturnType<typeof setTimeout> | undefined

const NAV_ITEMS = [
  { text: 'Home', href: '/' },
  { text: 'AI', href: '/ai' },
  { text: 'OS', href: '/os' },
  { text: 'Docs', href: '/docs/' },
  { text: 'Tutorials', href: '/books/' },
  { text: 'Playground', href: '/playground' },
]

const UI_DROPDOWN = [
  { text: 'Overview', href: '/ui/', active: 'home' },
  { text: 'A2UI Demo', href: '/ui/a2ui/index.html', active: 'a2ui' },
  { text: 'Components', href: '/ui/gallery/index.html', active: 'gallery' },
  { text: 'Blocks', href: '/ui/blocks/index.html', active: 'blocks' },
  { text: 'Charts', href: '/ui/charts/index.html', active: 'charts' },
  { text: 'Desktop', href: '/ui-desktop' },
  { text: 'Android', href: '/ui-android' },
  { text: 'Harmony', href: '/ui-harmony' },
]

function toggleUiDropdown() {
  cancelClose()
  uiDropdownOpen.value = !uiDropdownOpen.value
}

function scheduleClose() {
  closeTimeout = setTimeout(() => {
    uiDropdownOpen.value = false
  }, 150)
}

function cancelClose() {
  if (closeTimeout) {
    clearTimeout(closeTimeout)
    closeTimeout = undefined
  }
}

function closeDropdowns() {
  cancelClose()
  uiDropdownOpen.value = false
  mobileMenuOpen.value = false
}
</script>

<template>
  <header class="sticky top-0 z-50 w-full border-b bg-background/80 backdrop-blur-xl">
    <div class="flex h-14 items-center px-4 md:px-6 w-full gap-3">
      <!-- Logo -->
      <a href="/" class="flex items-center gap-2 mr-4 shrink-0 no-underline text-foreground">
        <img src="/auto.svg" alt="Auto" class="h-8 w-auto" />
        <span class="font-bold text-lg hidden sm:inline">Auto Language</span>
      </a>

      <!-- Desktop nav links -->
      <nav class="hidden md:flex items-center gap-1 text-sm">
        <a v-for="item in NAV_ITEMS" :key="item.href" :href="item.href"
          class="px-3 py-1.5 rounded-md transition-colors text-foreground/80 hover:text-foreground hover:bg-accent no-underline">
          {{ item.text }}
        </a>

        <!-- UI Dropdown -->
        <div class="relative" @mouseleave="scheduleClose" @mouseenter="cancelClose">
          <button @click="toggleUiDropdown"
            class="px-3 py-1.5 rounded-md transition-colors text-foreground/80 hover:text-foreground hover:bg-accent flex items-center gap-1 text-sm bg-transparent border-none cursor-pointer">
            UI
            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="transition-transform" :class="{ 'rotate-180': uiDropdownOpen }">
              <path d="m6 9 6 6 6-6" />
            </svg>
          </button>
          <div v-if="uiDropdownOpen" class="absolute right-0 top-full pt-1" @mouseenter="cancelClose" @mouseleave="scheduleClose">
            <div class="min-w-[11rem] py-1 bg-popover border border-border rounded-lg shadow-lg z-50">
            <a v-for="item in UI_DROPDOWN" :key="item.href" :href="item.href"
              class="block px-3 py-1.5 text-sm rounded no-underline transition-colors"
              :class="activeSection === item.active ? 'bg-accent text-accent-foreground font-medium' : 'text-popover-foreground hover:bg-accent'">
              {{ item.text }}
            </a>
          </div>
          </div>
        </div>
      </nav>

      <!-- Right side actions -->
      <div class="flex items-center gap-1 ml-auto">
        <!-- Search button -->
        <button v-if="showSearch" @click="emit('searchClick')"
          class="hidden md:inline-flex items-center justify-center h-9 w-9 rounded-md border-none bg-transparent cursor-pointer text-foreground hover:bg-accent transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" />
          </svg>
        </button>

        <!-- Dark mode toggle -->
        <button @click="toggle"
          class="inline-flex items-center justify-center h-9 w-9 rounded-md border-none bg-transparent cursor-pointer text-foreground hover:bg-accent transition-colors">
          <!-- Sun icon (shown in dark mode, click to go light) -->
          <svg v-if="isDark" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
          </svg>
          <!-- Moon icon (shown in light mode, click to go dark) -->
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
          </svg>
        </button>

        <!-- Mobile menu toggle -->
        <button @click="mobileMenuOpen = !mobileMenuOpen"
          class="md:hidden inline-flex items-center justify-center h-9 w-9 rounded-md border-none bg-transparent cursor-pointer text-foreground hover:bg-accent transition-colors">
          <svg v-if="!mobileMenuOpen" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="4" x2="20" y1="12" y2="12" /><line x1="4" x2="20" y1="6" y2="6" /><line x1="4" x2="20" y1="18" y2="18" />
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 6 6 18M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Mobile menu -->
    <div v-if="mobileMenuOpen" class="md:hidden border-t bg-background px-4 py-2">
      <a v-for="item in NAV_ITEMS" :key="item.href" :href="item.href"
        class="block py-2 text-sm text-foreground no-underline hover:text-primary">
        {{ item.text }}
      </a>
      <div class="py-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider mt-2">UI</div>
      <a v-for="item in UI_DROPDOWN" :key="item.href" :href="item.href"
        class="block py-1.5 pl-4 text-sm text-foreground no-underline hover:text-primary"
        :class="activeSection === item.active ? 'text-primary font-medium' : ''">
        {{ item.text }}
      </a>
    </div>
  </header>
</template>
