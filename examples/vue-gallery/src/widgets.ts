// Thin wrapper: merges generated (name+route) with meta (blurb+group).
// Re-exports widgetGroups for App.vue / Home.vue — API unchanged.
import { generatedWidgets } from './widgets.generated'
import { widgetMeta, type WidgetMeta } from './widgets.meta'

export interface WidgetEntry {
  name: string
  route: string
  blurb: string
}

export interface WidgetGroup {
  label: string
  widgets: WidgetEntry[]
}

function buildGroups(): WidgetGroup[] {
  const byGroup: Map<string, WidgetEntry[]> = new Map()
  for (const w of generatedWidgets) {
    const meta: WidgetMeta | undefined = widgetMeta[w.name]
    const group = meta?.group ?? 'Uncategorized'
    const blurb = meta?.blurb ?? ''
    const entry: WidgetEntry = { name: w.name, route: w.route, blurb }
    const arr = byGroup.get(group) ?? []
    arr.push(entry)
    byGroup.set(group, arr)
  }
  // Preserve a sensible group order.
  const groupOrder = ['Form', 'Layout', 'Feedback', 'Overlay / Nav', 'Uncategorized']
  const groups: WidgetGroup[] = []
  for (const label of groupOrder) {
    const widgets = byGroup.get(label)
    if (widgets && widgets.length > 0) {
      groups.push({ label, widgets })
    }
  }
  return groups
}

export const widgetGroups: WidgetGroup[] = buildGroups()
