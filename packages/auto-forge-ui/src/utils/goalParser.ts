export interface GoalForm {
  title: string
  priority: string
  criteria: { text: string; checked: boolean }[]
  details: string
  depends_on: string[]
}

export function parseGoalContent(content: string, title: string): GoalForm {
  const criteria: { text: string; checked: boolean }[] = []
  const lines = content.split('\n')
  let inCriteria = false
  let inDetails = false
  const detailsLines: string[] = []

  for (const raw of lines) {
    const line = raw.trimEnd()
    if (line.startsWith('**Acceptance Criteria:**')) {
      inCriteria = true
      inDetails = false
      continue
    }
    if (line.startsWith('**Details:**')) {
      inCriteria = false
      inDetails = true
      continue
    }
    if (inCriteria) {
      const m = line.match(/^- \[([ x])\]\s*(.*)$/)
      if (m) {
        criteria.push({ text: m[2].trim(), checked: m[1] === 'x' })
      }
    } else if (inDetails) {
      detailsLines.push(line)
    }
  }

  return {
    title,
    priority: '',
    criteria,
    details: detailsLines.join('\n').trim(),
    depends_on: [],
  }
}

export function serializeGoalForm(form: GoalForm): string {
  const lines: string[] = []
  if (form.criteria.length > 0) {
    lines.push('**Acceptance Criteria:**')
    for (const c of form.criteria) {
      lines.push(`- [${c.checked ? 'x' : ' '}] ${c.text}`)
    }
    lines.push('')
  }
  if (form.details.trim()) {
    lines.push('**Details:**')
    lines.push(form.details)
  }
  return lines.join('\n')
}
