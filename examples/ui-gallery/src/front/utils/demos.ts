// src/front/utils/demos.ts
// Bridge to auto-man generated demos-registry.ts (Plan 549)

import { DEMOS, findDemo, getCategories, type DemoMeta } from '@/demos-registry'

export function getAllDemos(): DemoMeta[] {
  return DEMOS
}

export function getDemosCount(): number {
  return DEMOS.length
}

export function getDemoTitle(id: string): string {
  const d = findDemo(id)
  return d ? d.title : id
}

export function getDemoDesc(id: string): string {
  const d = findDemo(id)
  return d ? d.description : ''
}

export function getDemoDoc(id: string): string {
  const d = findDemo(id)
  return d ? d.doc : ''
}

export function getDemoSource(id: string): string {
  const d = findDemo(id)
  return d ? d.source : ''
}

export function getDemoPac(id: string): string {
  const d = findDemo(id)
  return d ? d.pac : ''
}

export function getDemoCategory(id: string): string {
  const d = findDemo(id)
  return d ? d.category : ''
}

export function isDemoLoadable(id: string): boolean {
  const d = findDemo(id)
  return d ? d.loadable : false
}

export function filterDemosBy(query: string, category: string): DemoMeta[] {
  const q = query.trim().toLowerCase()
  return DEMOS.filter((d: DemoMeta) => {
    const matchCategory = !category || category === 'all' || d.category === category
    const matchQuery =
      !q ||
      d.title.toLowerCase().includes(q) ||
      d.id.toLowerCase().includes(q) ||
      d.tags.some((t: string) => t.toLowerCase().includes(q))
    return matchCategory && matchQuery
  })
}
