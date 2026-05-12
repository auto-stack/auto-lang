import { computed, ref, type Ref } from 'vue'

export interface MarkdownSegment {
  type: 'markdown'
  text: string
}

export interface ComponentSegment {
  type: 'component'
  componentType: string
  props: Record<string, any>
  final: boolean
}

export type StreamingSegment = MarkdownSegment | ComponentSegment

/**
 * Attempt to parse partial/incomplete JSON by completing open structures.
 */
function parsePartialJSON(text: string): { value: any; valid: boolean } {
  const trimmed = text.trim()
  if (!trimmed) return { value: null, valid: false }

  // Try parsing as-is first
  try {
    return { value: JSON.parse(trimmed), valid: true }
  } catch {
    // Continue to recovery
  }

  // Recovery: complete open braces, brackets, and strings
  let inString = false
  let escape = false
  const stack: string[] = []

  for (let i = 0; i < trimmed.length; i++) {
    const ch = trimmed[i]
    if (escape) {
      escape = false
      continue
    }
    if (ch === '\\') {
      escape = true
      continue
    }
    if (ch === '"') {
      inString = !inString
      continue
    }
    if (inString) continue

    if (ch === '{' || ch === '[') {
      stack.push(ch === '{' ? '}' : ']')
    } else if ((ch === '}' || ch === ']') && stack.length > 0) {
      stack.pop()
    }
  }

  let completion = ''
  if (inString) completion += '"'
  completion += stack.reverse().join('')

  try {
    return { value: JSON.parse(trimmed + completion), valid: false }
  } catch {
    return { value: null, valid: false }
  }
}

interface JSONBlock {
  start: number
  end: number
  content: string
  closed: boolean
}

function findJSONBlocks(text: string): JSONBlock[] {
  const blocks: JSONBlock[] = []
  let i = 0
  while (i < text.length) {
    const fenceStart = text.indexOf('```json\n', i)
    if (fenceStart === -1) break

    const contentStart = fenceStart + 8
    const fenceEnd = text.indexOf('\n```', contentStart)

    if (fenceEnd !== -1) {
      blocks.push({
        start: fenceStart,
        end: fenceEnd + 4,
        content: text.slice(contentStart, fenceEnd),
        closed: true,
      })
      i = fenceEnd + 4
    } else {
      blocks.push({
        start: fenceStart,
        end: text.length,
        content: text.slice(contentStart),
        closed: false,
      })
      break
    }
  }
  return blocks
}

const COMPONENT_TYPES = new Set(['table']) // extend as needed

function isComponentJSON(value: any): value is { type: string } & Record<string, any> {
  return value && typeof value === 'object' && typeof value.type === 'string' && COMPONENT_TYPES.has(value.type)
}

function buildSegments(text: string): StreamingSegment[] {
  const blocks = findJSONBlocks(text)
  const segments: StreamingSegment[] = []
  let cursor = 0

  for (const block of blocks) {
    // Markdown before this block
    if (block.start > cursor) {
      segments.push({ type: 'markdown', text: text.slice(cursor, block.start) })
    }

    // Try to parse block content as component JSON
    const { value, valid } = parsePartialJSON(block.content)
    if (isComponentJSON(value)) {
      const { type, ...props } = value
      segments.push({
        type: 'component',
        componentType: type,
        props,
        final: valid && block.closed,
      })
    } else {
      // Not a recognized component — render as normal markdown code block
      const fence = block.closed
        ? text.slice(block.start, block.end)
        : text.slice(block.start, block.end) + '\n```'
      segments.push({ type: 'markdown', text: fence })
    }

    cursor = block.end
  }

  // Trailing markdown
  if (cursor < text.length) {
    segments.push({ type: 'markdown', text: text.slice(cursor) })
  }

  return segments
}

export function useStreamingDocument(rawText: Ref<string>) {
  const segments = computed(() => buildSegments(rawText.value))
  return { segments }
}
