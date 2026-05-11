import { describe, it, expect, vi, beforeEach } from 'vitest'
import { nextTick } from 'vue'
import { useNotebook } from '../useNotebook'

describe('useNotebook', () => {
  beforeEach(() => {
    ;(globalThis as any).fetch = vi.fn()
  })

  it('should initialize with one default code cell', () => {
    const nb = useNotebook()
    expect(nb.cells.value.length).toBe(1)
    expect(nb.cells.value[0].type).toBe('code')
    expect(nb.cells.value[0].status).toBe('idle')
  })

  it('should add a cell', () => {
    const nb = useNotebook()
    const id = nb.addCell('markdown')
    expect(nb.cells.value.length).toBe(2)
    expect(nb.cells.value[1].type).toBe('markdown')
    expect(nb.cells.value[1].id).toBe(id)
  })

  it('should add a cell after specific id', () => {
    const nb = useNotebook()
    const firstId = nb.cells.value[0].id
    const newId = nb.addCell('code', firstId)
    expect(nb.cells.value.length).toBe(2)
    expect(nb.cells.value[1].id).toBe(newId)
  })

  it('should delete a cell', () => {
    const nb = useNotebook()
    const firstId = nb.cells.value[0].id
    nb.addCell('code')
    expect(nb.cells.value.length).toBe(2)
    nb.deleteCell(firstId)
    expect(nb.cells.value.length).toBe(1)
  })

  it('should ensure at least one cell remains after delete', () => {
    const nb = useNotebook()
    const firstId = nb.cells.value[0].id
    nb.deleteCell(firstId)
    expect(nb.cells.value.length).toBe(1)
    expect(nb.cells.value[0].type).toBe('code')
  })

  it('should move cell up', () => {
    const nb = useNotebook()
    const id1 = nb.cells.value[0].id
    const id2 = nb.addCell('markdown')
    nb.moveCell(id2, 'up')
    expect(nb.cells.value[0].id).toBe(id2)
    expect(nb.cells.value[1].id).toBe(id1)
  })

  it('should move cell down', () => {
    const nb = useNotebook()
    const id2 = nb.addCell('markdown')
    const id1 = nb.cells.value[0].id
    nb.moveCell(id1, 'down')
    expect(nb.cells.value[0].id).toBe(id2)
    expect(nb.cells.value[1].id).toBe(id1)
  })

  it('should mark cell dirty when source changes', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      json: async () => ({ session_id: 'test-session-123' }),
    } as any)
    ;(globalThis as any).fetch = mockFetch

    const nb = useNotebook()
    const cell = nb.cells.value[0]
    await nb.executeCell(cell)
    // After execution, cell should be clean
    expect(nb.isDirty(cell)).toBe(false)
    // Modify source
    cell.source = 'modified'
    await nextTick()
    expect(nb.isDirty(cell)).toBe(true)
  })

  it('should serialize to AutoDown format', () => {
    const nb = useNotebook()
    nb.cells.value = [
      { id: 'c1', type: 'code', source: 'let x = 1', status: 'idle', collapsed: false, depends_on: [] },
      { id: 'c2', type: 'markdown', source: '# Hello', status: 'idle', collapsed: false, depends_on: ['c1'] },
    ]
    const ad = nb.serializeToAd()
    expect(ad).toContain('/// cell:c1 type:code')
    expect(ad).toContain('let x = 1')
    expect(ad).toContain('/// cell:c2 type:markdown depends_on:c1')
    expect(ad).toContain('# Hello')
  })

  it('should load from AutoDown format', async () => {
    const nb = useNotebook()
    const source = `/// cell:c1 type:code
let x = 1

/// cell:c2 type:markdown depends_on:c1
# Hello
`
    await nb.loadFromAd(source)
    expect(nb.cells.value.length).toBe(2)
    expect(nb.cells.value[0].id).toBe('c1')
    expect(nb.cells.value[1].id).toBe('c2')
    expect(nb.cells.value[1].depends_on).toEqual(['c1'])
  })

  it('should load plain markdown without directives as single cell', async () => {
    const nb = useNotebook()
    await nb.loadFromAd('# Title\n\nSome text.')
    expect(nb.cells.value.length).toBe(1)
    expect(nb.cells.value[0].type).toBe('markdown')
    expect(nb.cells.value[0].source).toContain('# Title')
  })

  it('should extract code block from AI response into new code cell', () => {
    const nb = useNotebook()
    const aiId = nb.addCell('ai')
    const aiCell = nb.cells.value.find((c) => c.id === aiId)!
    aiCell.source = 'Here is the code:\n```auto\nlet x = 1 + 2\nprint(x)\n```\nEnjoy!'

    const newId = nb.extractCodeFromAI(aiId)
    expect(newId).not.toBeNull()
    expect(nb.cells.value.length).toBe(3) // default + ai + new code
    const newCell = nb.cells.value.find((c) => c.id === newId)
    expect(newCell).toBeDefined()
    expect(newCell!.type).toBe('code')
    expect(newCell!.source).toBe('let x = 1 + 2\nprint(x)')
  })

  it('should return null when no code block in AI response', () => {
    const nb = useNotebook()
    const aiId = nb.addCell('ai')
    const aiCell = nb.cells.value.find((c) => c.id === aiId)!
    aiCell.source = 'Just some plain text without code.'

    const newId = nb.extractCodeFromAI(aiId)
    expect(newId).toBeNull()
    expect(nb.cells.value.length).toBe(2)
  })

  it('should return null for non-ai cell', () => {
    const nb = useNotebook()
    const codeId = nb.cells.value[0].id
    const newId = nb.extractCodeFromAI(codeId)
    expect(newId).toBeNull()
  })
})
