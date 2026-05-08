<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Button } from '@/components/ui/button'
import { Palette, Check } from 'lucide-vue-next'

const themes = [
  { name: 'daylight', label: 'Daylight', color: 'bg-indigo-500' },
  { name: 'midnight', label: 'Midnight', color: 'bg-indigo-400' },
  { name: 'forest', label: 'Forest', color: 'bg-emerald-500' },
  { name: 'sunset', label: 'Sunset', color: 'bg-orange-500' },
  { name: 'ocean', label: 'Ocean', color: 'bg-cyan-500' },
]

const currentTheme = ref('daylight')

onMounted(() => {
  const saved = localStorage.getItem('aui-theme')
  if (saved && themes.find(t => t.name === saved)) {
    currentTheme.value = saved
    applyTheme(saved)
  }
})

function applyTheme(name: string) {
  document.documentElement.setAttribute('data-theme', name)
  localStorage.setItem('aui-theme', name)
  currentTheme.value = name
}
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button variant="ghost" size="icon" class="h-9 w-9">
        <slot>
          <Palette class="h-4 w-4" />
          <span class="sr-only">Toggle theme</span>
        </slot>
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="end" class="w-44">
      <DropdownMenuItem
        v-for="theme in themes"
        :key="theme.name"
        @click="applyTheme(theme.name)"
        class="flex items-center gap-2 cursor-pointer"
      >
        <span class="h-4 w-4 rounded-full border" :class="theme.color" />
        <span class="flex-1">{{ theme.label }}</span>
        <Check v-if="currentTheme === theme.name" class="h-3.5 w-3.5" />
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
