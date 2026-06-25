<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Search, Layers, ArrowRight,
  MousePointerClick, Type, Bell, Navigation,
  SquareStack, LayoutGrid, Command
} from 'lucide-vue-next'

const router = useRouter()
const searchQuery = ref('')

const componentCategories = [
  {
    name: 'Form',
    color: 'bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-200 dark:border-blue-800',
    dot: 'bg-blue-500',
    items: [
      { name: 'Button', path: '/button', desc: 'Clickable actions', icon: MousePointerClick },
      { name: 'Input', path: '/input', desc: 'Text fields', icon: Type },
      { name: 'Checkbox', path: '/checkbox', desc: 'Boolean toggles', icon: SquareStack },
      { name: 'Switch', path: '/switch', desc: 'Slide toggles', icon: SquareStack },
      { name: 'Select', path: '/select', desc: 'Dropdown choices', icon: LayoutGrid },
      { name: 'Slider', path: '/slider', desc: 'Range values', icon: LayoutGrid },
      { name: 'Form', path: '/form', desc: 'Form layouts', icon: LayoutGrid },
      { name: 'Calendar', path: '/calendar', desc: 'Date display', icon: LayoutGrid },
    ]
  },
  {
    name: 'Display',
    color: 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800',
    dot: 'bg-emerald-500',
    items: [
      { name: 'Card', path: '/card', desc: 'Content containers', icon: SquareStack },
      { name: 'Badge', path: '/badge', desc: 'Status labels', icon: Bell },
      { name: 'Avatar', path: '/avatar', desc: 'User pictures', icon: Layers },
      { name: 'Skeleton', path: '/skeleton', desc: 'Loading states', icon: LayoutGrid },
      { name: 'Carousel', path: '/carousel', desc: 'Slideshows', icon: LayoutGrid },
      { name: 'Table', path: '/table', desc: 'Data grids', icon: LayoutGrid },
    ]
  },
  {
    name: 'Feedback',
    color: 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-200 dark:border-amber-800',
    dot: 'bg-amber-500',
    items: [
      { name: 'Alert', path: '/alert', desc: 'Callout messages', icon: Bell },
      { name: 'Progress', path: '/progress', desc: 'Progress bars', icon: LayoutGrid },
      { name: 'Sonner', path: '/sonner', desc: 'Toast notifications', icon: Bell },
      { name: 'Toast', path: '/toast', desc: 'Brief alerts', icon: Bell },
      { name: 'Tooltip', path: '/tooltip', desc: 'Context hints', icon: Bell },
    ]
  },
  {
    name: 'Navigation',
    color: 'bg-purple-500/10 text-purple-600 dark:text-purple-400 border-purple-200 dark:border-purple-800',
    dot: 'bg-purple-500',
    items: [
      { name: 'Breadcrumb', path: '/breadcrumb', desc: 'Path navigation', icon: Navigation },
      { name: 'Tabs', path: '/tabs', desc: 'Tab panels', icon: LayoutGrid },
      { name: 'Menubar', path: '/menubar', desc: 'Menu bars', icon: LayoutGrid },
      { name: 'Pagination', path: '/pagination', desc: 'Page controls', icon: LayoutGrid },
      { name: 'Command', path: '/command', desc: 'Command palettes', icon: Command },
    ]
  },
  {
    name: 'Overlay',
    color: 'bg-rose-500/10 text-rose-600 dark:text-rose-400 border-rose-200 dark:border-rose-800',
    dot: 'bg-rose-500',
    items: [
      { name: 'Dialog', path: '/dialog', desc: 'Modal windows', icon: SquareStack },
      { name: 'Drawer', path: '/drawer', desc: 'Slide panels', icon: LayoutGrid },
      { name: 'Sheet', path: '/sheet', desc: 'Bottom sheets', icon: SquareStack },
      { name: 'Popover', path: '/popover', desc: 'Floating panels', icon: LayoutGrid },
      { name: 'DropdownMenu', path: '/dropdownmenu', desc: 'Context menus', icon: LayoutGrid },
      { name: 'Accordion', path: '/accordion', desc: 'Collapsible sections', icon: LayoutGrid },
    ]
  },
]

const filteredCategories = computed(() => {
  const q = searchQuery.value.toLowerCase().trim()
  if (!q) return componentCategories
  return componentCategories.map(cat => ({
    ...cat,
    items: cat.items.filter(i =>
      i.name.toLowerCase().includes(q) ||
      i.desc.toLowerCase().includes(q)
    )
  })).filter(cat => cat.items.length > 0)
})

function goToComponent(path: string) {
  router.push(path)
}
</script>

<template>
  <div class="flex flex-col gap-10">
    <!-- Hero Section -->
    <section class="relative overflow-hidden rounded-2xl md:rounded-3xl">
      <div class="absolute inset-0 bg-gradient-to-br from-primary/10 via-background to-accent/10" />
      <div class="absolute -top-24 -right-24 h-64 w-64 rounded-full bg-primary/5 blur-3xl" />
      <div class="absolute -bottom-24 -left-24 h-64 w-64 rounded-full bg-accent/5 blur-3xl" />

      <div class="relative px-4 py-12 md:py-20 lg:py-24 text-center">
        <Badge variant="outline" class="mb-6 px-3 py-1 text-xs font-medium tracking-wide">
          <Layers class="h-3 w-3 mr-1.5" />
          v1.0 — 46 Components
        </Badge>

        <h1 class="text-4xl md:text-5xl lg:text-7xl font-bold tracking-tight mb-4">
          <span class="bg-gradient-to-r from-foreground via-primary to-accent bg-clip-text text-transparent">
            Auto UI
          </span>
        </h1>

        <p class="text-base md:text-lg text-muted-foreground max-w-2xl mx-auto mb-2">
          A collection of beautifully designed, fully responsive components.
        </p>
        <p class="text-sm md:text-base text-muted-foreground/80 max-w-xl mx-auto mb-8">
          Built with Auto language, styled with Tailwind CSS, powered by Reka UI.
        </p>

        <div class="flex flex-wrap items-center justify-center gap-3">
          <Button size="lg" class="shadow-lg shadow-primary/20 gap-2" @click="router.push('/button')">
            Get Started
            <ArrowRight class="h-4 w-4" />
          </Button>
          <Button size="lg" variant="outline" @click="router.push('/button')">
            Browse Components
          </Button>
        </div>
      </div>
    </section>

    <!-- Search -->
    <div class="max-w-xl mx-auto w-full">
      <div class="relative">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          v-model="searchQuery"
          placeholder="Search components..."
          class="pl-10 h-11 rounded-xl"
        />
        <kbd class="absolute right-3 top-1/2 -translate-y-1/2 hidden sm:inline-flex h-6 items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground">
          CMD+K
        </kbd>
      </div>
    </div>

    <!-- Component Grid -->
    <div class="space-y-10">
      <div v-for="category in filteredCategories" :key="category.name">
        <div class="flex items-center gap-2 mb-4">
          <span class="h-2.5 w-2.5 rounded-full" :class="category.dot" />
          <h2 class="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
            {{ category.name }}
          </h2>
          <span class="text-xs text-muted-foreground/60">({{ category.items.length }})</span>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
          <button
            v-for="item in category.items"
            :key="item.path"
            @click="goToComponent(item.path)"
            class="group flex items-start gap-3 rounded-xl border p-4 text-left transition-all duration-200 hover:shadow-md hover:-translate-y-0.5 hover:border-primary/30 bg-card"
          >
            <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border" :class="category.color">
              <component :is="item.icon" class="h-5 w-5" />
            </div>
            <div class="min-w-0">
              <div class="font-medium text-sm truncate">{{ item.name }}</div>
              <div class="text-xs text-muted-foreground truncate">{{ item.desc }}</div>
            </div>
            <ArrowRight class="h-4 w-4 ml-auto shrink-0 text-muted-foreground opacity-0 -translate-x-2 transition-all group-hover:opacity-100 group-hover:translate-x-0" />
          </button>
        </div>
      </div>

      <div v-if="filteredCategories.length === 0" class="text-center py-12 text-muted-foreground">
        No components match "{{ searchQuery }}"
      </div>
    </div>

    <!-- Footer -->
    <footer class="border-t pt-8 pb-4 text-center text-sm text-muted-foreground">
      <p>Auto UI Component Gallery — Built with Auto Language</p>
    </footer>
  </div>
</template>
