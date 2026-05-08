<script setup lang="ts">
import type { BulletLegendItemInterface } from "@unovis/ts"
import type { Component } from "vue"
import { omit } from "@unovis/ts"
import { VisTooltip } from "@unovis/vue"
import { createApp } from "vue"
import { ChartTooltip } from "."

const props = defineProps<{
  selector: string
  index: string
  items?: BulletLegendItemInterface[]
  valueFormatter?: (tick: number, i?: number, ticks?: number[]) => string
  customTooltip?: Component
  total?: number
}>()

// Use weakmap to store reference to each datapoint for Tooltip
const wm = new WeakMap()
function template(d: any, i: number, elements: (HTMLElement | SVGElement)[]) {
  const valueFormatter = props.valueFormatter ?? ((tick: number) => `${tick}`)
  if (props.index in d) {
    if (wm.has(d)) {
      return wm.get(d)
    }
    else {
      const componentDiv = document.createElement("div")
      const entries = Object.entries(omit(d, [props.index]))
      const dataEntries = entries.filter(([k]) => k !== 'description')
      const omittedData = entries.map(([key, value]) => {
        if (key === 'description') {
          return { name: '', color: '', value: String(value) }
        }
        const legendReference = props.items?.find(i => i.name === key)
          ?? (dataEntries.length === 1 ? props.items?.find(i => i.name === d[props.index]) : undefined)
        let formatted = valueFormatter(value)
        if (props.total && typeof value === 'number') {
          formatted += ` (${Math.round((value / props.total) * 100)}%)`
        }
        return { ...legendReference, value: formatted }
      })
      const TooltipComponent = props.customTooltip ?? ChartTooltip
      createApp(TooltipComponent, { title: d[props.index], data: omittedData }).mount(componentDiv)
      wm.set(d, componentDiv.innerHTML)
      return componentDiv.innerHTML
    }
  }

  else {
    const data = d.data

    if (wm.has(data)) {
      return wm.get(data)
    }
    else {
      const style = getComputedStyle(elements[i])
      const omittedData = [{ name: data.name, value: valueFormatter(data[props.index]), color: style.fill }]
      const componentDiv = document.createElement("div")
      const TooltipComponent = props.customTooltip ?? ChartTooltip
      createApp(TooltipComponent, { title: d[props.index], data: omittedData }).mount(componentDiv)
      wm.set(d, componentDiv.innerHTML)
      return componentDiv.innerHTML
    }
  }
}
</script>

<template>
  <VisTooltip
    :horizontal-shift="20" :vertical-shift="20" :triggers="{
      [selector]: template,
    }"
  />
</template>
