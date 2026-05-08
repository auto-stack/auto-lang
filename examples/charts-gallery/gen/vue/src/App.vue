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
const trafficSource = ref<any[]>([{source: 'Desktop', value: 55, description: 'Windows, macOS & Linux browsers'}, {source: 'Mobile', value: 30, description: 'iOS & Android devices'}, {source: 'Tablet', value: 15, description: 'iPad & Android tablets'}])

const sparklineMetrics = ref([
  { label: 'Revenue', value: '$12.4K', change: '+12%', positive: true, data: [{m:'Jan',v:10},{m:'Feb',v:12},{m:'Mar',v:11},{m:'Apr',v:14},{m:'May',v:13},{m:'Jun',v:16}] },
  { label: 'Users', value: '8,320', change: '+5%', positive: true, data: [{m:'Jan',v:120},{m:'Feb',v:115},{m:'Mar',v:125},{m:'Apr',v:130},{m:'May',v:128},{m:'Jun',v:135}] },
  { label: 'Bounce Rate', value: '42%', change: '-3%', positive: true, data: [{m:'Jan',v:48},{m:'Feb',v:46},{m:'Mar',v:45},{m:'Apr',v:44},{m:'May',v:43},{m:'Jun',v:42}] },
  { label: 'Session', value: '4m 12s', change: '+8%', positive: true, data: [{m:'Jan',v:220},{m:'Feb',v:210},{m:'Mar',v:230},{m:'Apr',v:240},{m:'May',v:235},{m:'Jun',v:250}] },
  { label: 'Churn', value: '2.1%', change: '+0.4%', positive: false, data: [{m:'Jan',v:18},{m:'Feb',v:19},{m:'Mar',v:20},{m:'Apr',v:21},{m:'May',v:21},{m:'Jun',v:22}] },
  { label: 'CTR', value: '3.8%', change: '-0.5%', positive: false, data: [{m:'Jan',v:42},{m:'Feb',v:40},{m:'Mar',v:41},{m:'Apr',v:39},{m:'May',v:38},{m:'Jun',v:37}] },
])

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
          <Card class="md:col-span-2">
            <div class="flex flex-col p-6 gap-4">
              <span class="text-lg font-semibold text-foreground">
                <span>Sparkline Grid — 6 Metrics</span>
              </span>
              <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
                <div v-for="m in sparklineMetrics" :key="m.label" class="flex flex-col gap-2 p-3 rounded-lg border bg-card">
                  <div class="flex justify-between items-start">
                    <span class="text-xs text-muted-foreground">{{ m.label }}</span>
                    <span class="text-xs font-medium" :class="m.positive ? 'text-emerald-600' : 'text-rose-600'">{{ m.change }}</span>
                  </div>
                  <span class="text-lg font-bold">{{ m.value }}</span>
                  <LineChart :data="m.data" :categories="['v']" index="m" :show-x-axis="false" :show-y-axis="false" :show-tooltip="false" :show-legend="false" :show-grid-line="false" class="h-[40px] w-full" />
                </div>
              </div>
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
