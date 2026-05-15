<!-- App component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref } from 'vue'
import { AreaChart } from '@/components/ui/chart-area'
import { BarChart } from '@/components/ui/chart-bar'
import { Card } from '@/components/ui/card'
import { DonutChart } from '@/components/ui/chart-donut'
import { LineChart } from '@/components/ui/chart-line'
import ThemeToggle from '@/components/ThemeToggle.vue'
import UnifiedNavbar from '@/components/UnifiedNavbar.vue'

import { CurveType } from '@unovis/ts'

const monthlyRevenue = ref<any[]>([{desktop: 186, month: 'Jan', mobile: 80, tablet: 45, wearable: 25}, {desktop: 305, tablet: 60, month: 'Feb', wearable: 35, mobile: 200}, {wearable: 30, desktop: 237, month: 'Mar', mobile: 120, tablet: 55}, {mobile: 190, month: 'Apr', desktop: 73, tablet: 40, wearable: 20}, {mobile: 130, tablet: 50, desktop: 209, month: 'May', wearable: 28}, {wearable: 42, desktop: 214, mobile: 140, tablet: 65, month: 'Jun'}])
const quarterlySales = ref<any[]>([{product: 4500, support: 1800, license: 2100, quarter: 'Q1', service: 3200}, {service: 4100, support: 1950, product: 5200, license: 2300, quarter: 'Q2'}, {product: 4800, license: 2400, support: 2100, quarter: 'Q3', service: 3800}, {product: 6100, support: 2200, quarter: 'Q4', license: 2600, service: 4500}])
const trafficSource = ref<any[]>([{value: 55, source: 'Desktop'}, {value: 30, source: 'Mobile'}, {source: 'Tablet', value: 15}])


</script>

<template>
  <div class="flex flex-col h-screen">
    <UnifiedNavbar active-section="charts" />
    <div class="flex flex-row items-center justify-between px-6 py-2 border-b">
      <span class="text-sm text-muted-foreground">Color Theme:</span>
      <ThemeToggle />
    </div>
    <main class="flex-1 overflow-auto">
      <div class="flex flex-col p-6 md:p-8 gap-6 max-w-7xl mx-auto bg-background">
      <div class="flex flex-col gap-1">
        <h1 class="text-4xl font-bold tracking-tight text-3xl font-bold tracking-tight">
          <span>Charts Gallery</span>
        </h1>
        <span class="text-sm text-slate-500">
          <span>shadcn-vue chart components powered by Unovis</span>
        </span>
      </div>
      <Card>
        <div class="flex flex-col p-6 gap-4">
          <span class="text-lg font-semibold">
            <span>Area Chart — Revenue Overview</span>
          </span>
          <AreaChart :data="monthlyRevenue" :categories="['desktop', 'mobile', 'tablet', 'wearable']" index="month" class="h-[300px] w-full" />
        </div>
      </Card>
      <Card>
        <div class="flex flex-col p-6 gap-4">
          <span class="text-lg font-semibold">
            <span>Bar Chart — Quarterly Sales (Grouped)</span>
          </span>
          <BarChart :data="quarterlySales" :categories="['product', 'service', 'license', 'support']" index="quarter" type="grouped" class="h-[300px] w-full" />
        </div>
      </Card>
      <Card>
        <div class="flex flex-col p-6 gap-4">
          <span class="text-lg font-semibold">
            <span>Bar Chart — Quarterly Sales (Stacked)</span>
          </span>
          <BarChart :data="quarterlySales" :categories="['product', 'service']" index="quarter" type="stacked" class="h-[300px] w-full" />
        </div>
      </Card>
      <Card>
        <div class="flex flex-col p-6 gap-4">
          <span class="text-lg font-semibold">
            <span>Line Chart — Trend Over Time</span>
          </span>
          <LineChart :data="monthlyRevenue" :categories="['desktop', 'mobile']" index="month" :curve-type="CurveType.MonotoneX" class="h-[300px] w-full" />
        </div>
      </Card>
      <Card>
        <div class="flex flex-col p-6 gap-4">
          <span class="text-lg font-semibold">
            <span>Donut Chart — Traffic Source</span>
          </span>
          <DonutChart :data="trafficSource" category="source" index="source" class="h-[300px] w-full" />
        </div>
      </Card>
      <Card>
        <div class="flex flex-col p-6 gap-4">
          <span class="text-lg font-semibold">
            <span>Area Chart — Custom Colors</span>
          </span>
          <AreaChart :data="monthlyRevenue" :categories="['desktop', 'mobile']" index="month" :colors="['#3b82f6', '#10b981']" class="h-[300px] w-full" />
        </div>
      </Card>
      <Card>
        <div class="flex flex-col p-6 gap-4">
          <span class="text-lg font-semibold">
            <span>Sparkline — Minimal Trend</span>
          </span>
          <LineChart :data="monthlyRevenue" :categories="['desktop']" index="month" :show-x-axis="false" :show-y-axis="false" :show-tooltip="false" :show-legend="false" :show-grid-line="false" class="h-[120px] w-full" />
        </div>
      </Card>
      </div>
    </main>
  </div>

</template>

<style scoped>
/* Component styles */

</style>


