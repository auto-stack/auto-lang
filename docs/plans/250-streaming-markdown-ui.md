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

### Phase 2: Streaming Component Protocol (SCP)

Inspired by Tiptap's `streamTool`, design a lightweight Vue-native protocol:

**Concept**: The AI output is a stream of **directives**, not just text. Each directive targets a typed node in a document tree.

```
Directives are embedded in the AI's text stream as special tokens:

[[NODE:chart id="revenue" type="bar"]]
[[PATCH:revenue data.labels=["Q1","Q2"]]]
[[PATCH:revenue data.datasets.0.data+=[120]]]
[[CLOSE:revenue]]
```

The Furnace renderer maintains a `document: Map<string, StreamingNode>`:
- `[[NODE:type id="x" ...props]]` → create/update a node
- `[[PATCH:id path value]]` → incremental property update
- `[[CLOSE:id]]` → finalize node (disable loading state)

**Vue integration**: A `<StreamingDocument :directives="directives">` component renders nodes as dynamic Vue components:
```vue
<component
  v-for="node in nodes"
  :is="nodeRegistry[node.type]"
  v-bind="node.props"
  :streaming="!node.final"
/>
```

**Registry**: `nodeRegistry` maps type names to Vue components:
- `markdown` → `MarkdownRender` (markstream-vue)
- `chart` → `VueChart` (chart library wrapper)
- `table` → `DataTable`
- Any AutoUI widget → generated Vue SFC

### Phase 3: AutoUI Integration

Extend the Auto language with a `stream` block or `#[streaming]` annotation:

```auto
widget LiveDashboard {
    model {
        revenue_chart = #[streaming] ChartWidget { ... }
    }
    view {
        col {
            // This component receives PATCH directives from the AI
            .revenue_chart
        }
    }
}
```

The `a2vue` generator emits a wrapper that registers the component with the Streaming Component Protocol registry and handles `v-bind:streaming-props`.

### Phase 4: Furnace Tool Integration

Extend the Forge backend tools to emit SCP directives:
- `write_component(type, id, props)` → emit `[[NODE:...]]`
- `patch_component(id, path, value)` → emit `[[PATCH:...]]`
- This lets the AI stream live charts, tables, and forms directly into the chat.

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
