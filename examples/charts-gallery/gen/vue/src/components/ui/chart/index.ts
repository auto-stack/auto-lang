export { default as ChartCrosshair } from "./ChartCrosshair.vue"
export { default as ChartLegend } from "./ChartLegend.vue"
export { default as ChartSingleTooltip } from "./ChartSingleTooltip.vue"
export { default as ChartTooltip } from "./ChartTooltip.vue"

export function defaultColors(count: number = 3) {
  const colors: string[] = []
  const bases = ['var(--vis-primary-color)', 'var(--vis-secondary-color)', 'var(--vis-tertiary-color)', 'var(--vis-quaternary-color)']
  const cycle = bases.length
  const maxSteps = Math.ceil(count / cycle)
  for (let i = 0; i < count; i++) {
    const baseIdx = i % cycle
    const step = Math.floor(i / cycle)
    const base = bases[baseIdx]
    // Opacity drops gently: 1.0 → 0.75 → 0.5 for 4th+ color of same base
    const opacity = Math.max(0.5, 1 - (step / maxSteps) * 0.5)
    colors.push(`hsl(${base} / ${opacity})`)
  }
  return colors
}

export * from "./interface"
