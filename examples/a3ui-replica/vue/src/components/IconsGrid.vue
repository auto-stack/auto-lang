<template>
  <div class="grid grid-cols-6 md:grid-cols-8 lg:grid-cols-12 gap-4">
    <div
      v-for="iconName in iconNames"
      :key="iconName"
      class="flex flex-col items-center gap-1 p-3 rounded-lg border border-slate-200 hover:border-violet-300 cursor-pointer transition-colors"
      @click="copyIcon(iconName)"
    >
      <component :is="getIcon(iconName)" class="h-6 w-6 text-slate-600" />
      <span class="text-[10px] text-slate-500">{{ iconName }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import * as lucide from 'lucide-vue-next'

const iconNames = [
  'home', 'user', 'mail', 'phone', 'search', 'settings', 'heart', 'star',
  'calendar', 'clock', 'map', 'music', 'camera', 'image', 'video', 'file',
  'folder', 'trash', 'edit', 'copy', 'check', 'x', 'plus', 'minus',
  'arrowLeft', 'arrowRight', 'arrowUp', 'arrowDown', 'chevronLeft', 'chevronRight',
  'menu', 'bell', 'bookmark', 'flag', 'tag', 'link', 'lock', 'unlock',
  'eye', 'eyeOff', 'download', 'upload', 'share', 'send', 'messageCircle',
  'globe', 'shoppingCart', 'creditCard', 'dollarSign', 'trendingUp', 'trendingDown',
  'activity', 'barChart', 'pieChart', 'layers', 'grid', 'list', 'layout',
  'monitor', 'smartphone', 'tablet', 'wifi', 'bluetooth', 'battery', 'power',
  'sun', 'moon', 'cloud', 'umbrella', 'thermometer', 'wind', 'droplets',
]

function getIcon(name: string) {
  const pascalName = name.charAt(0).toUpperCase() + name.slice(1)
  return (lucide as any)[pascalName] || lucide.HelpCircle
}

function copyIcon(name: string) {
  navigator.clipboard.writeText(name)
}
</script>
