<template>
  <div class="output-chart">
    <div v-if="error" class="chart-error">{{ error }}</div>
    <svg v-else-if="chartData" :viewBox="`0 0 ${width} ${height}`" class="chart-svg">
      <!-- Background -->
      <rect :width="width" :height="height" fill="#1e1e2e" rx="4" />
      
      <!-- Title -->
      <text v-if="chartData.title" :x="width / 2" y="20" text-anchor="middle" fill="#cdd6f4" font-size="14" font-weight="600">
        {{ chartData.title }}
      </text>

      <!-- Axes -->
      <line :x1="margin.left" :y1="margin.top" :x2="margin.left" :y2="height - margin.bottom" stroke="#45475a" stroke-width="1" />
      <line :x1="margin.left" :y1="height - margin.bottom" :x2="width - margin.right" :y2="height - margin.bottom" stroke="#45475a" stroke-width="1" />

      <!-- Bars (bar chart) -->
      <template v-if="chartData.type === 'bar'">
        <rect
          v-for="(bar, i) in bars"
          :key="i"
          :x="bar.x"
          :y="bar.y"
          :width="bar.width"
          :height="bar.height"
          :fill="bar.color"
          rx="2"
        />
        <!-- X labels -->
        <text
          v-for="(label, i) in xLabels"
          :key="`l${i}`"
          :x="label.x"
          :y="label.y"
          text-anchor="middle"
          fill="#6c7086"
          font-size="10"
        >{{ label.text }}</text>
      </template>

      <!-- Line (line chart) -->
      <template v-if="chartData.type === 'line'">
        <polyline
          :points="linePoints"
          fill="none"
          stroke="#6366f1"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        <circle
          v-for="(pt, i) in linePointsArr"
          :key="i"
          :cx="pt.x"
          :cy="pt.y"
          r="3"
          fill="#6366f1"
        />
        <!-- X labels -->
        <text
          v-for="(label, i) in xLabels"
          :key="`l${i}`"
          :x="label.x"
          :y="label.y"
          text-anchor="middle"
          fill="#6c7086"
          font-size="10"
        >{{ label.text }}</text>
      </template>

      <!-- Y axis labels -->
      <text
        v-for="(label, i) in yLabels"
        :key="`yl${i}`"
        :x="label.x"
        :y="label.y"
        text-anchor="end"
        fill="#6c7086"
        font-size="9"
      >{{ label.text }}</text>
    </svg>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  source: string
}>()

const width = 400
const height = 220
const margin = { top: 32, right: 16, bottom: 32, left: 40 }
const chartWidth = width - margin.left - margin.right
const chartHeight = height - margin.top - margin.bottom

const chartData = computed(() => {
  try {
    const parsed = JSON.parse(props.source)
    if (!parsed.type) {
      // Try to infer from shape
      if (Array.isArray(parsed.data) && parsed.data.every((d: any) => typeof d === 'number')) {
        return { type: 'bar', data: parsed.data, labels: parsed.labels || parsed.data.map((_: any, i: number) => String(i + 1)), title: parsed.title }
      }
      return null
    }
    return parsed as { type: 'bar' | 'line'; data: number[]; labels?: string[]; title?: string }
  } catch {
    return null
  }
})

const error = computed(() => {
  if (!chartData.value) return 'Invalid chart data. Expected JSON with {type, data, labels?}.'
  if (!Array.isArray(chartData.value.data)) return 'Chart data must be an array of numbers.'
  return null
})

const data = computed(() => chartData.value?.data ?? [])
const labels = computed(() => chartData.value?.labels ?? data.value.map((_val: number, i: number) => String(i + 1)))

const maxValue = computed(() => {
  const vals = data.value
  if (vals.length === 0) return 1
  return Math.max(...vals, 0) * 1.1
})

const bars = computed(() => {
  if (chartData.value?.type !== 'bar') return []
  const vals = data.value
  const n = vals.length
  const gap = 8
  const barW = n > 0 ? (chartWidth - gap * (n - 1)) / n : 0
  const colors = ['#6366f1', '#89b4fa', '#cba6f7', '#f9e2af', '#a6e3a1', '#f38ba8']
  return vals.map((v: number, i: number) => {
    const h = (v / maxValue.value) * chartHeight
    return {
      x: margin.left + i * (barW + gap),
      y: margin.top + chartHeight - h,
      width: Math.max(barW, 2),
      height: h,
      color: colors[i % colors.length],
    }
  })
})

const xLabels = computed(() => {
  const n = labels.value.length
  if (n === 0) return []
  const gap = 8
  const barW = chartData.value?.type === 'bar' ? (chartWidth - gap * (n - 1)) / n : chartWidth / (n - 1 || 1)
  return labels.value.map((text: string, i: number) => {
    const x = chartData.value?.type === 'bar'
      ? margin.left + i * (barW + gap) + barW / 2
      : margin.left + (i / Math.max(n - 1, 1)) * chartWidth
    return { x, y: height - margin.bottom + 14, text }
  })
})

const yLabels = computed(() => {
  const steps = 4
  return Array.from({ length: steps + 1 }, (_: unknown, i: number) => {
    const v = (maxValue.value / steps) * i
    const y = margin.top + chartHeight - (v / maxValue.value) * chartHeight + 3
    return { x: margin.left - 6, y, text: Math.round(v).toString() }
  })
})

const linePointsArr = computed(() => {
  if (chartData.value?.type !== 'line') return []
  const vals = data.value
  const n = vals.length
  if (n === 0) return []
  return vals.map((v: number, i: number) => {
    const x = margin.left + (i / Math.max(n - 1, 1)) * chartWidth
    const y = margin.top + chartHeight - (v / maxValue.value) * chartHeight
    return { x, y }
  })
})

const linePoints = computed(() => linePointsArr.value.map((p: {x: number, y: number}) => `${p.x},${p.y}`).join(' '))
</script>

<style scoped>
.output-chart {
  padding: 0.5rem 0.75rem;
}

.chart-svg {
  width: 100%;
  max-width: 400px;
  height: auto;
  display: block;
  margin: 0 auto;
}

.chart-error {
  color: #f38ba8;
  font-size: 0.8rem;
  padding: 0.5rem;
  background: #f38ba811;
  border-radius: 4px;
}
</style>
