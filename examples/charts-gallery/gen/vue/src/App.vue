<!-- App component - Charts Gallery -->
<script setup lang="ts">
import { ref } from 'vue'
import { AreaChart } from '@/components/ui/chart-area'
import { BarChart } from '@/components/ui/chart-bar'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { DonutChart } from '@/components/ui/chart-donut'
import { LineChart } from '@/components/ui/chart-line'
import ThemeToggle from '@/components/ThemeToggle.vue'
import UnifiedNavbar from '@/components/UnifiedNavbar.vue'

import { CurveType } from '@unovis/ts'

const activeTab = ref<string>('area')
const tabs = [
  { id: 'area', label: 'Area Charts' },
  { id: 'bar', label: 'Bar Charts' },
  { id: 'line', label: 'Line Charts' },
  { id: 'pie', label: 'Pie Charts' },
]

// Data sets
const monthlyRevenue = ref<any[]>([
  { desktop: 186, month: 'Jan', mobile: 80, tablet: 45, wearable: 25 },
  { desktop: 305, tablet: 60, month: 'Feb', wearable: 35, mobile: 200 },
  { wearable: 30, desktop: 237, month: 'Mar', mobile: 120, tablet: 55 },
  { mobile: 190, month: 'Apr', desktop: 73, tablet: 40, wearable: 20 },
  { mobile: 130, tablet: 50, desktop: 209, month: 'May', wearable: 28 },
  { wearable: 42, desktop: 214, mobile: 140, tablet: 65, month: 'Jun' },
])

const quarterlySales = ref<any[]>([
  { product: 4500, support: 1800, license: 2100, quarter: 'Q1', service: 3200 },
  { service: 4100, support: 1950, product: 5200, license: 2300, quarter: 'Q2' },
  { product: 4800, license: 2400, support: 2100, quarter: 'Q3', service: 3800 },
  { product: 6100, support: 2200, quarter: 'Q4', license: 2600, service: 4500 },
])

const trafficSource = ref<any[]>([
  { value: 45, source: 'Desktop' },
  { value: 30, source: 'Mobile' },
  { value: 12, source: 'Tablet' },
  { value: 8, source: 'Wearable' },
  { value: 5, source: 'Other' },
])

const visitorData = ref<any[]>([
  { visitors: 276, month: 'Jan' },
  { visitors: 330, month: 'Feb' },
  { visitors: 205, month: 'Mar' },
  { visitors: 310, month: 'Apr' },
  { visitors: 420, month: 'May' },
  { visitors: 380, month: 'Jun' },
])
</script>

<template>
  <div class="flex flex-col h-screen">
    <UnifiedNavbar active-section="charts" />
    <div class="flex flex-row items-center justify-between px-6 py-2 border-b">
      <span class="text-sm text-muted-foreground">Color Theme:</span>
      <ThemeToggle />
    </div>
    <main class="flex-1 overflow-auto">
      <div class="flex flex-col p-6 md:p-8 gap-6 max-w-5xl mx-auto">
        <div class="flex flex-col gap-1">
          <h1 class="text-4xl font-bold tracking-tight">Charts Gallery</h1>
          <span class="text-muted-foreground">shadcn-vue chart components powered by Unovis</span>
        </div>

        <!-- Tab navigation -->
        <div class="flex flex-row gap-2 flex-wrap">
          <Button v-for="tab in tabs" :key="tab.id" :variant="activeTab === tab.id ? 'default' : 'outline'" size="sm" @click="activeTab = tab.id">{{ tab.label }}</Button>
        </div>

        <!-- Area Charts -->
        <template v-if="activeTab === 'area'">
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <div class="flex flex-col gap-1">
                <span class="text-base font-semibold">Area Chart - Interactive</span>
                <span class="text-sm text-muted-foreground">Showing total visitors for the last 6 months</span>
              </div>
              <AreaChart :data="monthlyRevenue" :categories="['desktop', 'mobile', 'tablet', 'wearable']" index="month" class="h-[300px] w-full" />
            </div>
          </Card>
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <div class="flex flex-col gap-1">
                <span class="text-base font-semibold">Area Chart - Gradient</span>
                <span class="text-sm text-muted-foreground">Desktop vs Mobile with gradient fill</span>
              </div>
              <AreaChart :data="monthlyRevenue" :categories="['desktop', 'mobile']" index="month" :show-gradient="true" class="h-[300px] w-full" />
            </div>
          </Card>
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <div class="flex flex-col gap-1">
                <span class="text-base font-semibold">Area Chart - Axes</span>
                <span class="text-sm text-muted-foreground">Desktop visitors with formatted axis labels</span>
              </div>
              <AreaChart :data="visitorData" :categories="['visitors']" index="month" :show-gradient="true" class="h-[300px] w-full" />
            </div>
          </Card>
        </template>

        <!-- Bar Charts -->
        <template v-if="activeTab === 'bar'">
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <div class="flex flex-col gap-1">
                <span class="text-base font-semibold">Bar Chart - Grouped</span>
                <span class="text-sm text-muted-foreground">Quarterly sales breakdown by category</span>
              </div>
              <BarChart :data="quarterlySales" :categories="['product', 'service', 'license', 'support']" index="quarter" type="grouped" class="h-[300px] w-full" />
            </div>
          </Card>
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <div class="flex flex-col gap-1">
                <span class="text-base font-semibold">Bar Chart - Stacked</span>
                <span class="text-sm text-muted-foreground">Product vs Service stacked comparison</span>
              </div>
              <BarChart :data="quarterlySales" :categories="['product', 'service']" index="quarter" type="stacked" class="h-[300px] w-full" />
            </div>
          </Card>
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <div class="flex flex-col gap-1">
                <span class="text-base font-semibold">Bar Chart - Single</span>
                <span class="text-sm text-muted-foreground">Monthly visitors as single series</span>
              </div>
              <BarChart :data="visitorData" :categories="['visitors']" index="month" type="grouped" class="h-[300px] w-full" />
            </div>
          </Card>
        </template>

        <!-- Line Charts -->
        <template v-if="activeTab === 'line'">
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <div class="flex flex-col gap-1">
                <span class="text-base font-semibold">Line Chart - Default</span>
                <span class="text-sm text-muted-foreground">Desktop vs Mobile with smooth curve</span>
              </div>
              <LineChart :data="monthlyRevenue" :categories="['desktop', 'mobile']" index="month" :curve-type="CurveType.MonotoneX" class="h-[300px] w-full" />
            </div>
          </Card>
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <div class="flex flex-col gap-1">
                <span class="text-base font-semibold">Line Chart - Linear</span>
                <span class="text-sm text-muted-foreground">Using linear interpolation between points</span>
              </div>
              <LineChart :data="monthlyRevenue" :categories="['desktop', 'mobile']" index="month" :curve-type="CurveType.Linear" class="h-[300px] w-full" />
            </div>
          </Card>
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <div class="flex flex-col gap-1">
                <span class="text-base font-semibold">Line Chart - Sparkline</span>
                <span class="text-sm text-muted-foreground">Minimal trend line without axes, legend, or tooltip</span>
              </div>
              <LineChart :data="monthlyRevenue" :categories="['desktop']" index="month" :show-x-axis="false" :show-y-axis="false" :show-tooltip="false" :show-legend="false" :show-grid-line="false" class="h-[120px] w-full" />
            </div>
          </Card>
        </template>

        <!-- Pie Charts -->
        <template v-if="activeTab === 'pie'">
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <div class="flex flex-col gap-1">
                <span class="text-base font-semibold">Donut Chart - Default</span>
                <span class="text-sm text-muted-foreground">Traffic source breakdown as a donut</span>
              </div>
              <DonutChart :data="trafficSource" category="value" index="source" class="h-[300px] w-full" />
            </div>
          </Card>
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <div class="flex flex-col gap-1">
                <span class="text-base font-semibold">Pie Chart</span>
                <span class="text-sm text-muted-foreground">Same data rendered as a full pie (no center hole)</span>
              </div>
              <DonutChart :data="trafficSource" category="value" index="source" type="pie" class="h-[300px] w-full" />
            </div>
          </Card>
          <Card>
            <div class="flex flex-col p-6 gap-4">
              <div class="flex flex-col gap-1">
                <span class="text-base font-semibold">Donut Chart - More Data</span>
                <span class="text-sm text-muted-foreground">5 segments with percentage breakdown</span>
              </div>
              <DonutChart :data="trafficSource" category="value" index="source" class="h-[300px] w-full" />
            </div>
          </Card>
        </template>

      </div>
    </main>
  </div>
</template>

<style scoped>
/* Component styles */

</style>
