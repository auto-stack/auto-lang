<!-- App component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref } from 'vue'
import { AreaChart } from '@/components/ui/chart-area'
import { BarChart } from '@/components/ui/chart-bar'
import { Card } from '@/components/ui/card'
import { DonutChart } from '@/components/ui/chart-donut'
import { LineChart } from '@/components/ui/chart-line'
import { Button } from '@/components/ui/button'

import { BarChart3, Search } from 'lucide-vue-next'
import ThemeToggle from '@/components/ThemeToggle.vue'

import { CurveType } from '@unovis/ts'

const monthlyRevenue = ref<any[]>([{mobile: 80, month: 'Jan', desktop: 186, tablet: 45, wearable: 25}, {month: 'Feb', mobile: 200, desktop: 305, tablet: 60, wearable: 35}, {month: 'Mar', mobile: 120, desktop: 237, tablet: 55, wearable: 30}, {desktop: 73, mobile: 190, month: 'Apr', tablet: 40, wearable: 20}, {month: 'May', desktop: 209, mobile: 130, tablet: 50, wearable: 28}, {mobile: 140, month: 'Jun', desktop: 214, tablet: 65, wearable: 42}])
const quarterlySales = ref<any[]>([{product: 4500, service: 3200, quarter: 'Q1'}, {product: 5200, quarter: 'Q2', service: 4100}, {product: 4800, service: 3800, quarter: 'Q3'}, {quarter: 'Q4', product: 6100, service: 4500}])
const trafficSource = ref<any[]>([{source: 'Desktop', value: 55}, {source: 'Mobile', value: 30}, {source: 'Tablet', value: 15}])

</script>

<template>
  <div class="flex flex-col h-screen">
    <header class="sticky top-0 z-40 w-full border-b bg-background/95 backdrop-blur">
      <div class="flex flex-row h-14 items-center px-4 md:px-6 w-full gap-3">
        <div class="flex flex-row items-center gap-2 mr-4 shrink-0">
          <BarChart3 class="w-5 h-5 text-primary" />
          <span class="font-bold text-lg hidden sm:inline">Auto Charts</span>
        </div>
        <div class="flex flex-row hidden md:flex items-center gap-1 text-sm">
          <a href="/" class="group block"><span class="px-3 py-1.5 rounded-md transition-colors">Home</span></a>
          <a href="/ui/index.html" class="group block"><span class="px-3 py-1.5 rounded-md transition-colors">UI</span></a>
          <a href="/ui/gallery/index.html" class="group block"><span class="px-3 py-1.5 rounded-md transition-colors">Gallery</span></a>
          <a href="/ui/blocks/index.html" class="group block"><span class="px-3 py-1.5 rounded-md transition-colors">Blocks</span></a>
          <a href="/ui/charts/index.html" class="group block"><span class="px-3 py-1.5 rounded-md transition-colors">Charts</span></a>
          <a href="/ui/a2ui/index.html" class="group block"><span class="px-3 py-1.5 rounded-md transition-colors">A2UI</span></a>
        </div>
        <div class="flex flex-row ml-auto items-center gap-1">
          <Button variant="ghost" class="hidden md:flex h-9 w-9">
            <Search class="h-4 w-4" />
          </Button>
          <ThemeToggle />
        </div>
      </div>
    </header>
    <main class="flex-1 overflow-auto bg-background">
      <div class="flex flex-col p-6 md:p-8 gap-6 max-w-7xl mx-auto">
        <div class="flex flex-col gap-1">
          <h1 class="text-3xl md:text-4xl font-bold tracking-tight text-foreground">
            <span>Charts Gallery</span>
          </h1>
          <span class="text-sm text-muted-foreground">
            <span>shadcn-vue chart components powered by Unovis</span>
          </span>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <span class="text-lg font-semibold text-foreground">
                <span>Area Chart — Revenue Overview</span>
              </span>
              <AreaChart :data="monthlyRevenue" :categories="['desktop', 'mobile', 'tablet', 'wearable']" index="month" class="h-[300px] w-full" />
            </div>
          </Card>
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <span class="text-lg font-semibold text-foreground">
                <span>Bar Chart — Quarterly Sales (Grouped)</span>
              </span>
              <BarChart :data="quarterlySales" :categories="['product', 'service']" index="quarter" type="grouped" class="h-[300px] w-full" />
            </div>
          </Card>
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <span class="text-lg font-semibold text-foreground">
                <span>Bar Chart — Quarterly Sales (Stacked)</span>
              </span>
              <BarChart :data="quarterlySales" :categories="['product', 'service']" index="quarter" type="stacked" class="h-[300px] w-full" />
            </div>
          </Card>
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <span class="text-lg font-semibold text-foreground">
                <span>Line Chart — 4-Category Trend</span>
              </span>
              <LineChart :data="monthlyRevenue" :categories="['desktop', 'mobile', 'tablet', 'wearable']" index="month" :curve-type="CurveType.MonotoneX" class="h-[300px] w-full" />
            </div>
          </Card>
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <span class="text-lg font-semibold text-foreground">
                <span>Donut Chart — Traffic Source</span>
              </span>
              <DonutChart :data="trafficSource" category="value" index="source" class="h-[300px] w-full" />
            </div>
          </Card>
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <span class="text-lg font-semibold text-foreground">
                <span>Sparkline — Minimal Trend</span>
              </span>
              <LineChart :data="monthlyRevenue" :categories="['desktop']" index="month" :show-x-axis="false" :show-y-axis="false" :show-tooltip="false" :show-legend="false" :show-grid-line="false" class="h-[120px] w-full" />
            </div>
          </Card>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
/* Component styles */
</style>
