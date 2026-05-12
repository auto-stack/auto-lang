# Plan: Streaming UI Architecture for AutoForge & AutoUI

## Problem

The Furnace chat UI re-renders markdown on **every SSE delta** via `marked.parse()` inside a Vue `computed()` property. This causes visual flicker, DOM thrashing, and broken incomplete markdown blocks.

But the real opportunity is bigger: **how do we streamify ANY UI component**, not just markdown? AutoUI generates Vue SFCs from `.at` files. In the future, the AI should be able to stream live-updating charts, tables, forms, and custom widgets — not just text.

## Research Findings

### markstream-vue (Vue-native StreamDown equivalent)
- Drop-in Vue 3 streaming markdown renderer with zero flicker
- Two render modes: **virtual window** (long docs) and **incremental batching** (AI typing effect)
- Progressive Mermaid diagrams, streaming Monaco/Shiki code blocks, KaTeX math
- Supports **custom Vue components inline** via `setCustomComponents()`
- SSR + worker pre-parsing support
- Peer deps are optional (only bundle what you use)

### Tiptap AI Toolkit
- `streamTool({ toolCallId, toolName, input, hasFinished })` — streams structured tool output into a ProseMirror document
- ANY component can be a Tiptap node extension; the AI streams JSON `input` that maps to node props
- `AiCaret` extension shows where the AI is inserting
- Review UI with Tracked Changes or suggestion decorations
- **Key insight**: the document is a tree of typed nodes; streaming means incrementally updating nodes

### AutoUI Architecture
- `.at` → AST → AURA IR → Vue SFC via `a2vue`
- WidgetRegistry maps tags to Vue components (shadcn-vue or native HTML)
- No runtime plugin system; generated apps are static at build time
- Custom PascalCase tags auto-import from `@/components/{Name}.vue`

## Goal

1. **Immediate**: Replace flickering `marked`-based renderer with `markstream-vue`
2. **Strategic**: Design a **Streaming Component Protocol** that lets the AI stream structured updates to ANY Vue component (markdown, charts, tables, forms, custom AutoUI widgets)

## Approach

### Phase 1: Drop-in markstream-vue for Furnace

Replace `MarkdownRenderer.vue` with `markstream-vue`:

```vue
<template>
  <MarkdownRender
    :content="source"
    :final="!streaming"
    :max-live-nodes="streaming ? 0 : 320"
    :batch-rendering="streaming"
    :render-batch-size="16"
    :render-batch-delay="8"
    :typewriter="streaming"
  />
</template>
```

- `final=false` during streaming → incomplete blocks (unclosed fences, `$$`) show loading state instead of broken HTML
- `max-live-nodes=0` + `batch-rendering` → incremental typewriter effect with no flicker
- `typewriter=true` → built-in streaming cursor

**Dependencies**: add `markstream-vue` to `packages/auto-smith-ui/package.json`.

### Phase 2: Streaming Component Protocol (SCP) — JSON Code Blocks

**Implemented:** The AI outputs standard JSON inside markdown code blocks. The frontend detects ` ```json ` blocks, parses partial/incomplete JSON using a recovery parser, and renders recognized component types.

```
Here is the user list:
```json
{
  "type": "table",
  "columns": ["Name", "Email"],
  "rows": [
    {"Name": "Alice", "Email": "alice@example.com"}
  ]
}
```
```

**Frontend parsing:**
- `findJSONBlocks()` scans streaming text for ` ```json ` fences
- `parsePartialJSON()` completes open braces/brackets/strings so `JSON.parse()` succeeds on incomplete data
- If JSON has a recognized `type` field (e.g., `"table"`), it renders as a live component
- Otherwise, it falls back to a normal markdown code block

**Registry:** `nodeRegistry` maps type names to Vue components:
- `table` → `StreamingTable`
- Future: `chart` → `StreamingChart`, `form` → `StreamingForm`, etc.

### Phase 3: AutoUI Integration — Future Direction (Auto Code)

**Long-term goal:** The AI outputs **Auto code** (`.at` widgets) directly, not JSON. Auto is the native language of the ecosystem — JSON is a temporary bridge.

**Layered architecture:**

```
┌─────────────────────────────────────────┐
│  AI Output Format (evolves over time)   │
│  • Now: JSON code blocks                │
│  • Next: Auto code → server transpiler  │
│  • Future: Auto code → browser parser   │
├─────────────────────────────────────────┤
│  Incremental Parser                     │
│  • parsePartialJSON()    ← today        │
│  • Auto → AURA IR transpiler ← next     │
│  • parsePartialAuto()    ← future       │
├─────────────────────────────────────────┤
│  Common Intermediate: AURA IR           │
│  • AuraNode tree (Element, Text,        │
│    ForLoop, Conditional, Component)     │
│  • State definitions                    │
│  • Handler bindings                     │
├─────────────────────────────────────────┤
│  Vue Runtime Renderer                   │
│  • Maps AuraNode → Vue <component :is>  │
│  • Binds state via Vue reactivity       │
│  • Handles events → VM / callbacks      │
└─────────────────────────────────────────┘
```

**Evolution path:**

| Phase | AI Output | Parser | Renderer |
|-------|-----------|--------|----------|
| **Now** | ` ```json {"type":"table",...} ` | `parsePartialJSON()` | Component registry (`StreamingTable`) |
| **Next** | ` ```auto widget X { view {...} } ` | Server-side transpiler (Auto → AURA IR JSON) | AURA runtime renderer |
| **Future** | Raw Auto code inline with text | Incremental Auto parser in browser | Full AURA runtime renderer |

**Why AURA IR is the common language:**
- AURA IR is already ~80% framework-agnostic (view tree is pure; handlers carry Vue bias)
- The `WidgetRegistry` already maps tags to multi-backend components
- `LogicPayload::Bytecode` was designed for dynamic execution but is currently unused
- A runtime renderer for AURA IR would support Vue today and React/Svelte tomorrow

**Phase Next implementation:**
1. AI writes Auto code in a code block
2. Backend transpiles it to AURA IR JSON (reuses existing `extract.rs` → serialize)
3. AURA IR JSON is streamed to frontend
4. Frontend `A2UIRuntimeRenderer` interprets `AuraNode` tree directly

**AURA Runtime Renderer sketch:**
```vue
<template>
  <component
    v-for="node in auraTree"
    :is="nodeTypeToVueComponent(node)"
    v-bind="extractProps(node)"
  >
    <A2UIRuntimeRenderer v-if="node.children" :nodes="node.children" />
  </component>
</template>
```

This renderer understands `AuraNode::Element`, `AuraNode::ForLoop`, `AuraNode::Conditional`, `AuraNode::Text`, etc. — the same structures the `a2vue` generator consumes today.

### Phase 4: Furnace Tool Integration

Extend the Forge backend tools for structured output:
- **Now:** `write_table(columns, rows)` → backend formats JSON code block
- **Next:** `write_widget(autoCode)` → backend transpiles to AURA IR, streams JSON
- **Future:** Direct Auto code streaming with no tool boundary

The tool layer stays thin — it's just a translator between AI intent and the frontend protocol.

## Files to Touch (Phase 1 only)

- `packages/auto-smith-ui/package.json` — add `markstream-vue`
- `packages/auto-smith-ui/src/components/MarkdownRenderer.vue` — replace with markstream-vue wrapper
- `packages/auto-smith-ui/src/views/FurnaceView.vue` — pass `streaming` flag

## Trade-offs

| Approach | Pros | Cons |
|---|---|---|
| **A. markstream-vue (Phase 1)** | Battle-tested Vue-native streaming markdown; incremental batching + virtual window; supports inline custom components; minimal code change | New dependency; peer deps (Shiki/Monaco) are optional but can bloat if enabled |
| **B. Hand-roll stabilizer on `marked`** | Zero new deps | Reinventing the wheel; won't catch all edge cases; no virtual windowing |
| **C. Tiptap AI Toolkit** | Full document model; any component as extension; proven review/caret UX | Heavy ProseMirror dependency; React-first docs; paid Pro features; overkill for chat |

**Recommended: A for Phase 1, then evolve toward a custom SCP inspired by Tiptap's architecture.**

markstream-vue gives us immediate parity with StreamDown. The Streaming Component Protocol (Phase 2+) gives us a Tiptap-like extension model without the ProseMirror weight — pure Vue, purpose-built for AI-generated UIs.

## Acceptance Criteria (Phase 1)

- [ ] `npm install markstream-vue` in auto-smith-ui
- [ ] Streaming assistant message shows zero-flicker markdown with stable incomplete blocks
- [ ] Typing cursor built into markstream-vue appears during streaming
- [ ] Completed messages render instantly with `final=true`
- [ ] Build passes (`npm run build` in `packages/auto-smith-ui`)
