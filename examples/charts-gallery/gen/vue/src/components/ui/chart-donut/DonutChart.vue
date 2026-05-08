<script setup lang="ts" generic="T extends Record<string, any>">
import type { Component } from "vue"
import type { BaseChartProps } from "."
import { Donut } from "@unovis/ts"
import { VisDonut, VisSingleContainer, VisTooltip } from "@unovis/vue"
import { useMounted } from "@vueuse/core"
import { computed, ref } from "vue"
import { cn } from "@/lib/utils"
import { defaultColors } from '@/components/ui/chart'

const props = withDefaults(defineProps<Pick<BaseChartProps<T>, "data" | "colors" | "index" | "margin" | "showLegend" | "showTooltip" | "filterOpacity"> & {
  category: KeyOfT
  type?: "donut" | "pie"
  sortFunction?: (a: any, b: any) => number | undefined
  valueFormatter?: (tick: number, i?: number, ticks?: number[]) => string
  customTooltip?: Component
}>(), {
  type: "donut",
  margin: () => ({ top: 0, bottom: 0, left: 0, right: 0 }),
  sortFunction: () => undefined,
  filterOpacity: 0.2,
  showTooltip: true,
  showLegend: true,
})

type KeyOfT = Extract<keyof T, string>
type Data = typeof props.data[number]

const valueFormatter = props.valueFormatter ?? ((tick: number) => `${tick}`)
const category = computed(() => props.category as KeyOfT)
const index = computed(() => props.index as KeyOfT)

const isMounted = useMounted()
const activeSegmentKey = ref<string>()
const hoveredSegmentKey = ref<string>()
const colors = computed(() => props.colors?.length ? props.colors : defaultColors(props.data.filter(d => d[props.category]).filter(Boolean).length))
const legendItems = computed(() => props.data.map((item, i) => ({
  name: item[props.index],
  color: colors.value[i],
  inactive: false,
})))

const totalValue = computed(() => props.data.reduce((prev, curr) => {
  return prev + curr[props.category]
}, 0))

function setOpacity(elements: HTMLElement[], key: string | undefined, mode: 'click' | 'hover') {
  if (mode === 'hover' && activeSegmentKey.value) return
  elements.forEach((el, idx) => {
    const segKey = props.data[idx]?.[props.index]
    if (key === undefined) {
      el.style.opacity = '1'
    } else if (segKey === key) {
      el.style.opacity = '1'
    } else {
      el.style.opacity = `${props.filterOpacity}`
    }
  })
}

// Tooltip template for segment hover (single item)
function segmentTemplate(d: any) {
  const item = d.data
  const percent = Math.round((item[props.category] / totalValue.value) * 100)
  const color = colors.value.find((c, i) => props.data[i]?.[props.index] === item[props.index]) || '#888'
  return `
    <div style="background:#fff;border:1px solid #e2e8f0;border-radius:8px;padding:12px;min-width:180px;font-size:13px;box-shadow:0 4px 12px rgba(0,0,0,0.08);">
      <div style="font-weight:600;margin-bottom:6px;font-size:14px;">${item[props.index]}</div>
      <div style="display:flex;align-items:center;gap:6px;margin-bottom:4px;">
        <span style="display:inline-block;width:8px;height:8px;border-radius:50%;background:${color};"></span>
        <span>${item[props.category]} (${percent}%)</span>
      </div>
      ${item.description ? `<div style="color:#64748b;font-style:italic;margin-top:4px;border-top:1px solid #f1f5f9;padding-top:4px;">${item.description}</div>` : ''}
    </div>
  `
}

// Tooltip template for background/center hover (table of all items)
function backgroundTemplate() {
  const rows = props.data.map((item, i) => {
    const percent = Math.round((item[props.category] / totalValue.value) * 100)
    const isHovered = hoveredSegmentKey.value === item[props.index]
    const bg = isHovered ? 'background:#f8fafc;' : ''
    return `
      <div style="display:flex;align-items:center;justify-content:space-between;gap:12px;padding:3px 0;${bg}">
        <div style="display:flex;align-items:center;gap:6px;">
          <span style="display:inline-block;width:8px;height:8px;border-radius:50%;background:${colors.value[i]};"></span>
          <span>${item[props.index]}</span>
        </div>
        <span style="font-weight:600;">${item[props.category]} (${percent}%)</span>
      </div>
    `
  }).join('')
  return `
    <div style="background:#fff;border:1px solid #e2e8f0;border-radius:8px;padding:12px;min-width:200px;font-size:13px;box-shadow:0 4px 12px rgba(0,0,0,0.08);">
      <div style="font-weight:600;margin-bottom:8px;font-size:14px;">Traffic Sources</div>
      ${rows}
    </div>
  `
}
</script>

<template>
  <div :class="cn('w-full h-48 flex flex-col items-end', $attrs.class ?? '')">
    <VisSingleContainer :style="{ height: isMounted ? '100%' : 'auto' }" :margin="{ left: 20, right: 20 }" :data="data">
      <VisTooltip
        v-if="showTooltip"
        :horizontal-shift="20"
        :vertical-shift="20"
        :triggers="{
          [Donut.selectors.segment]: segmentTemplate,
          [Donut.selectors.centralLabel]: backgroundTemplate,
        }"
      />

      <VisDonut
        :value="(d: Data) => d[category]"
        :sort-function="sortFunction"
        :color="colors"
        :arc-width="type === 'donut' ? 20 : 0"
        :show-background="false"
        :central-label="type === 'donut' ? valueFormatter(totalValue) : ''"
        :events="{
          [Donut.selectors.segment]: {
            click: (d: Data, ev: PointerEvent, i: number, elements: HTMLElement[]) => {
              if (d?.data?.[index] === activeSegmentKey) {
                activeSegmentKey = undefined
                setOpacity(elements, undefined, 'click')
              } else {
                activeSegmentKey = d?.data?.[index]
                setOpacity(elements, activeSegmentKey, 'click')
              }
            },
            mouseenter: (d: Data, ev: PointerEvent, i: number, elements: HTMLElement[]) => {
              hoveredSegmentKey = d?.data?.[index]
              setOpacity(elements, hoveredSegmentKey, 'hover')
            },
            mouseleave: (d: Data, ev: PointerEvent, i: number, elements: HTMLElement[]) => {
              hoveredSegmentKey = undefined
              setOpacity(elements, activeSegmentKey, 'hover')
            },
          },
        }"
      />

      <slot />
    </VisSingleContainer>
  </div>
</template>
