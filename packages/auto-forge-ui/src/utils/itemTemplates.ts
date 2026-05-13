import type { Status } from '@/types/specs'

export const ITEM_TEMPLATES: Record<string, string> = {
  goals: `**Acceptance Criteria:**
- [ ] <criterion 1>
- [ ] <criterion 2>

**Details:**
<Describe the goal in ≤500 words. What problem does it solve? Who benefits?>
`,

  tests: `**Type:** Unit
**Scope:** <related goal ID>

**Fixture:**
\`\`\`
// Setup code, mocks, test data
\`\`\`

**Steps:**
1. <imperative action>
2. <imperative action>
3. <imperative action>

**Expected Outcome:**
<Unambiguous result that a human or AI can verify>

**Test File:** 
`,

  apis: `**Endpoint:** 

**Description:** 

**Request:**
\`\`\`json
{
  // request schema
}
\`\`\`

**Response:**
\`\`\`json
{
  // response schema
}
\`\`\`

**Errors:**
- 400 Bad Request — 
- 404 Not Found — 
- 500 Internal Server Error — 
`,

  plans: `## Phase 1: <Phase Name>

- [ ] <task 1>
- [ ] <task 2>
- [ ] <task 3>
`,

  architecture: `**Decision:** <What architectural decision is being made?>

**Rationale:** <Why this approach? What alternatives were considered?>

**Components:**
- <component 1>
- <component 2>

**Trade-offs:**
- Pros: 
- Cons: 
`,

  designs: `**Interface:** <Module / Component / API name>

**Responsibilities:**
- <responsibility 1>
- <responsibility 2>

**Dependencies:**
- <dependency 1>
- <dependency 2>

**Notes:**
<Additional design considerations>
`,

  reviews: `### Review Date

- ⏳ <item to review>
- ⏳ <item to review>

## Action Items

1. <action item>
2. <action item>
`,

  reports: `## Summary

<Brief summary of what this report covers>

## Metrics

| Metric | Value | Target |
|--------|-------|--------|
| | — | |

## Findings

1. <finding>
2. <finding>

## Recommendations

1. <recommendation>
2. <recommendation>
`,
}

export function getDefaultStatus(sectionType: string): Status {
  switch (sectionType) {
    case 'goals': return 'proposed'
    case 'architecture': return 'draft'
    case 'designs': return 'draft'
    case 'plans': return 'draft'
    case 'tests': return 'draft'
    case 'reviews': return 'draft'
    case 'reports': return 'draft'
    case 'apis': return 'draft'
    default: return 'draft'
  }
}

export function getNextId(sectionType: string, existingIds: string[]): string {
  const prefixMap: Record<string, string> = {
    goals: 'G', architecture: 'A', designs: 'D', plans: 'P',
    tests: 'S', reviews: 'V', reports: 'X', apis: 'I',
  }
  const prefix = prefixMap[sectionType] || sectionType.charAt(0).toUpperCase()

  // Find the maximum numeric suffix among existing IDs with this prefix
  let maxNum = 0
  const re = new RegExp(`^${prefix}\\d+$`)
  for (const id of existingIds) {
    if (re.test(id)) {
      const num = parseInt(id.slice(prefix.length), 10)
      if (!isNaN(num) && num > maxNum) maxNum = num
    }
  }
  return `${prefix}${maxNum + 1}`
}
