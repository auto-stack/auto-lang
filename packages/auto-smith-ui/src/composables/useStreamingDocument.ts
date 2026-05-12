import { computed, ref, type Ref } from 'vue'

export interface StreamingNode {
  id: string
  type: string
  props: Record<string, any>
  final: boolean
}

export type Directive =
  | { kind: 'NODE'; type: string; id: string; props: Record<string, any> }
  | { kind: 'PATCH'; id: string; path: string; op: string; value: any }
  | { kind: 'CLOSE'; id: string }

function parseDirective(inner: string): Directive | null {
  // [[NODE type id propsJson]]
  const nodeMatch = inner.match(/^NODE\s+(\S+)\s+(\S+)\s+(.+)$/)
  if (nodeMatch) {
    try {
      return {
        kind: 'NODE',
        type: nodeMatch[1],
        id: nodeMatch[2],
        props: JSON.parse(nodeMatch[3]),
      }
    } catch {
      return null
    }
  }

  // [[PATCH id path op valueJson]]
  const patchMatch = inner.match(/^PATCH\s+(\S+)\s+(\S+)\s+(\S+)\s+(.+)$/)
  if (patchMatch) {
    try {
      return {
        kind: 'PATCH',
        id: patchMatch[1],
        path: patchMatch[2],
        op: patchMatch[3],
        value: JSON.parse(patchMatch[4]),
      }
    } catch {
      return null
    }
  }

  // [[CLOSE id]]
  const closeMatch = inner.match(/^CLOSE\s+(\S+)$/)
  if (closeMatch) {
    return { kind: 'CLOSE', id: closeMatch[1] }
  }

  return null
}

function parseDirectives(text: string): { directives: Directive[]; cleanText: string } {
  const directives: Directive[] = []
  const parts: string[] = []
  let lastIndex = 0

  const regex = /\[\[(.*?)\]\]/g
  let match: RegExpExecArray | null

  while ((match = regex.exec(text)) !== null) {
    parts.push(text.slice(lastIndex, match.index))
    const dir = parseDirective(match[1].trim())
    if (dir) directives.push(dir)
    lastIndex = regex.lastIndex
  }
  parts.push(text.slice(lastIndex))

  return { directives, cleanText: parts.join('') }
}

function getPath(obj: any, path: string): any {
  return path.split('.').reduce((o, key) => (o ? o[key] : undefined), obj)
}

function setPath(obj: any, path: string, value: any) {
  const keys = path.split('.')
  const last = keys.pop()!
  const target = keys.reduce((o, key) => {
    if (!o[key]) o[key] = {}
    return o[key]
  }, obj)
  target[last] = value
}

function applyDirective(nodes: StreamingNode[], dir: Directive) {
  if (dir.kind === 'NODE') {
    const existing = nodes.find((n) => n.id === dir.id)
    if (existing) {
      existing.type = dir.type
      existing.props = { ...existing.props, ...dir.props }
    } else {
      nodes.push({ id: dir.id, type: dir.type, props: dir.props, final: false })
    }
  } else if (dir.kind === 'PATCH') {
    const node = nodes.find((n) => n.id === dir.id)
    if (!node) return
    if (dir.op === 'set') {
      setPath(node.props, dir.path, dir.value)
    } else if (dir.op === 'append') {
      const arr = getPath(node.props, dir.path)
      if (Array.isArray(arr)) {
        const values = Array.isArray(dir.value) ? dir.value : [dir.value]
        arr.push(...values)
      }
    } else if (dir.op === 'merge') {
      const obj = getPath(node.props, dir.path)
      if (typeof obj === 'object' && obj !== null) {
        Object.assign(obj, dir.value)
      }
    }
  } else if (dir.kind === 'CLOSE') {
    const node = nodes.find((n) => n.id === dir.id)
    if (node) node.final = true
  }
}

export function useStreamingDocument(rawText: Ref<string>) {
  const nodes = ref<StreamingNode[]>([])
  const cleanText = ref('')
  const processed = new Set<string>()

  const update = (text: string) => {
    const { directives, cleanText: cleaned } = parseDirectives(text)
    cleanText.value = cleaned

    for (const dir of directives) {
      const key = JSON.stringify(dir)
      if (processed.has(key)) continue
      processed.add(key)
      applyDirective(nodes.value, dir)
    }
  }

  // React to prop changes
  const reactiveCleanText = computed(() => {
    update(rawText.value)
    return cleanText.value
  })

  // Ensure nodes stay reactive
  const reactiveNodes = computed(() => nodes.value)

  return { cleanText: reactiveCleanText, nodes: reactiveNodes }
}
