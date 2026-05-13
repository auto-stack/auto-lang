import type { SpecItem } from '@/types/specs'

/**
 * Extract a human-readable summary from an item's markdown content.
 * Each category has its own template structure.
 */

export function extractArchitectureSummary(item: SpecItem): string {
  const c = item.content
  const decision = c.match(/\*\*Decision:\*\*\s*(.+)/i)?.[1]?.trim()
  const hasMermaid = c.includes('```mermaid')
  const parts: string[] = []
  if (decision) parts.push(decision.slice(0, 120))
  if (hasMermaid) parts.push('📊 Diagram')
  return parts.join(' · ') || ''
}

export function extractDesignSummary(item: SpecItem): string {
  const c = item.content
  const iface = c.match(/```\w*\n([^(\n]+\([^)]*\))/m)?.[1]?.trim()
    || c.match(/\*\*Interface:\*\*\s*```\n?([^(\n]+\([^)]*\))/m)?.[1]?.trim()
  const module = item.module || c.match(/\*\*Module:\*\*\s*`?([^`\n]+)`?/m)?.[1]?.trim()
  const parts: string[] = []
  if (iface) parts.push(iface.slice(0, 120))
  if (module) parts.push(`📦 ${module}`)
  return parts.join(' · ') || ''
}

export function extractPlanSummary(item: SpecItem): string {
  const c = item.content
  const objective = c.match(/\*\*Objective:\*\*\s*(.+)/i)?.[1]?.trim()
  // Count phases by counting table rows (rough)
  const rows = c.match(/^\|[^|]+\|/gm)
  const phaseCount = rows ? rows.length - 1 : 0 // minus header
  const parts: string[] = []
  if (objective) parts.push(`Objective: ${objective.slice(0, 80)}`)
  if (phaseCount > 0) parts.push(`${phaseCount} phases`)
  return parts.join(' · ') || ''
}

export function extractTestSummary(item: SpecItem): string {
  const c = item.content
  const type = c.match(/\*\*Type:\*\*\s*(\w+)/i)?.[1]?.trim()
  const steps = c.match(/^\d+\./gm)
  const stepCount = steps ? steps.length : 0
  const parts: string[] = []
  if (type) parts.push(type)
  if (item.test_file) parts.push(`📄 ${item.test_file.split('/').pop()}`)
  if (stepCount > 0) parts.push(`${stepCount} steps`)
  return parts.join(' · ') || ''
}

export function extractReviewSummary(item: SpecItem): string {
  const c = item.content
  // Count checkmarks and warnings
  const passed = (c.match(/[☑✅✓]/g) || []).length
  const partial = (c.match(/[⚠️⚠]/g) || []).length
  const failed = (c.match(/[❌✗]/g) || []).length
  const issues = (c.match(/####\s+V\d+-I\d+/g) || []).length
  const parts: string[] = []
  if (passed || partial || failed) {
    parts.push(`${passed} passed${partial ? `, ${partial} partial` : ''}${failed ? `, ${failed} failed` : ''}`)
  }
  if (issues > 0) parts.push(`${issues} issue${issues > 1 ? 's' : ''}`)
  return parts.join(' · ') || ''
}

export function extractReportSummary(item: SpecItem): string {
  const c = item.content
  // Extract metrics count from table
  const metricRows = c.match(/^\|[^|]+\|[^|]+\|/gm)
  const metricCount = metricRows ? metricRows.length - 1 : 0
  const blockers = (c.match(/\*\*B\d+\*\*/g) || []).length
  const parts: string[] = []
  if (metricCount > 0) parts.push(`${metricCount} metrics`)
  if (blockers > 0) parts.push(`${blockers} blocker${blockers > 1 ? 's' : ''}`)
  return parts.join(' · ') || ''
}

export function extractApiSummary(item: SpecItem): string {
  const c = item.content
  const version = c.match(/\*\*Version:\*\*\s*(.+)/i)?.[1]?.trim()
  const endpoints = (c.match(/####\s+(GET|POST|PUT|DELETE|PATCH)/gi) || []).length
  const parts: string[] = []
  if (version) parts.push(`v${version}`)
  if (endpoints > 0) parts.push(`${endpoints} endpoint${endpoints > 1 ? 's' : ''}`)
  return parts.join(' · ') || ''
}

export function extractGenericSummary(item: SpecItem): string {
  // Fallback: extract first non-empty line that doesn't start with # or markdown syntax
  const lines = item.content.split('\n').filter(l => l.trim())
  for (const line of lines) {
    const trimmed = line.trim()
    if (trimmed.startsWith('#') || trimmed.startsWith('```') || trimmed.startsWith('<!--')) continue
    if (trimmed.startsWith('|') || trimmed.startsWith('- [')) continue
    return trimmed.slice(0, 160)
  }
  return ''
}
