<template>
  <div class="demo-view">
    <div class="demo-header">
      <h2>Streaming Component Demo</h2>
      <div class="demo-controls">
        <button class="demo-btn" @click="runMarkdownDemo">▶ Markdown</button>
        <button class="demo-btn" @click="runTableDemo">▶ Data Table</button>
        <button class="demo-btn" @click="runMixedDemo">▶ Mixed</button>
        <button class="demo-btn secondary" @click="reset">↺ Reset</button>
      </div>
    </div>

    <div class="demo-stage">
      <!-- Live stream view -->
      <div class="demo-panel">
        <div class="panel-label">StreamingRenderer</div>
        <StreamingRenderer :source="liveText" :streaming="isStreaming" />
      </div>

      <!-- Raw output view -->
      <div class="demo-panel raw">
        <div class="panel-label">Raw AI Output</div>
        <pre class="raw-text">{{ liveText }}</pre>
      </div>
    </div>

    <div class="demo-info">
      <h3>How it works</h3>
      <ul>
        <li>
          <strong>Markdown:</strong> Plain text is fed to
          <code>markstream-vue</code> → zero-flicker incremental rendering with
          typewriter cursor
        </li>
        <li>
          <strong>Table:</strong> The AI emits
          <code>```json {"type": "table", ...}</code> →
          <code>useStreamingDocument</code> parses partial JSON → renders
          <code>&lt;StreamingTable&gt;</code> component
        </li>
        <li>
          <strong>Mixed:</strong> Markdown + tables can appear in any order; the
          parser splits them into segments
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import StreamingRenderer from '@/components/StreamingRenderer.vue'

const liveText = ref('')
const isStreaming = ref(false)
let abortController: AbortController | null = null

const MARKDOWN_SAMPLE = `I've analyzed the authentication flow. Here's what I found:

## Current State

The app uses a **JWT-based** auth system with the following components:

- \`AuthProvider\` — React context wrapping the app
- \`useAuth()\` hook for consuming auth state
- \`/api/login\` and \`/api/refresh\` endpoints

## Issues Found

1. **Token refresh is missing** — when the access token expires, the user is kicked out instead of silently refreshing.
2. **No role-based guards** — the \`AdminPanel\` route is accessible to any logged-in user.
3. **Password reset flow is incomplete** — the email template is a placeholder.

Let me know if you'd like me to fix any of these.`

const TABLE_SAMPLE = `Here is the API coverage analysis:

\`\`\`json
{"type": "table", "columns": ["Endpoint", "Method", "Tested", "Coverage"], "rows": [{"Endpoint": "/api/users", "Method": "GET", "Tested": "Yes", "Coverage": "100%"}, {"Endpoint": "/api/users", "Method": "POST", "Tested": "Yes", "Coverage": "85%"}, {"Endpoint": "/api/users/:id", "Method": "PUT", "Tested": "No", "Coverage": "0%"}, {"Endpoint": "/api/auth/refresh", "Method": "POST", "Tested": "Yes", "Coverage": "60%"}, {"Endpoint": "/api/admin/ban", "Method": "POST", "Tested": "No", "Coverage": "0%"}]}
\`\`\`

**Recommendation:** Prioritize testing the untested PUT and admin endpoints.`

const MIXED_SAMPLE = `## Refactoring Plan

I'll reorganize the codebase into three layers:

\`\`\`json
{"type": "table", "columns": ["Layer", "Responsibility", "Files"], "rows": [{"Layer": "Domain", "Responsibility": "Business logic, entities, rules", "Files": "src/domain/*"}, {"Layer": "Application", "Responsibility": "Use cases, orchestration", "Files": "src/app/*"}, {"Layer": "Infrastructure", "Responsibility": "DB, HTTP, external APIs", "Files": "src/infra/*"}]}
\`\`\`

### Migration Order

1. **Move entities first** — they're the foundation, no downstream breakage.
2. **Extract use cases** — move service methods into application layer.
3. **Adapters last** — controllers and repositories can be swapped in easily.

This should take about **2 hours**. Shall I proceed?`

function reset() {
  const prev = abortController
  abortController = null
  if (prev) {
    prev.abort()
  }
  liveText.value = ''
  isStreaming.value = false
}

async function typeText(text: string, _speedMs = 30) {
  const controller = new AbortController()
  abortController = controller
  isStreaming.value = true
  liveText.value = ''

  let i = 0
  let buffer = ''
  let lastFrame = 0
  const FRAME_INTERVAL = 120 // ms between visible updates

  while (i < text.length) {
    if (controller.signal.aborted) break

    // Accumulate ~8 characters per "tick" into a buffer
    const charsPerTick = 8
    buffer += text.slice(i, i + charsPerTick)
    i += charsPerTick

    // Only flush to liveText on animation frames, throttled
    const now = performance.now()
    if (now - lastFrame >= FRAME_INTERVAL) {
      liveText.value += buffer
      buffer = ''
      lastFrame = now
      // Yield to browser so markstream-vue can render
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()))
    } else {
      // Small delay to throttle character generation
      await new Promise((r) => setTimeout(r, 16))
    }
  }

  // Flush any remaining buffer
  if (buffer && !controller.signal.aborted) {
    liveText.value += buffer
  }

  if (abortController === controller) {
    isStreaming.value = false
  }
}

function runMarkdownDemo() {
  reset()
  setTimeout(() => typeText(MARKDOWN_SAMPLE, 8), 50)
}

function runTableDemo() {
  reset()
  setTimeout(() => typeText(TABLE_SAMPLE, 10), 50)
}

function runMixedDemo() {
  reset()
  setTimeout(() => typeText(MIXED_SAMPLE, 9), 50)
}
</script>

<style scoped>
.demo-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.demo-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.6rem 1.25rem;
  flex-shrink: 0;
}

.demo-header h2 {
  font-size: 0.85rem;
  font-weight: 500;
}

.demo-controls {
  display: flex;
  gap: 0.4rem;
}

.demo-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  background: linear-gradient(135deg, var(--vp-c-brand-1) 0%, var(--vp-c-brand-2) 100%);
  color: #fff;
  border: none;
  border-radius: 6px;
  padding: 0.3rem 0.7rem;
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s;
}

.demo-btn.secondary {
  background: transparent;
  color: var(--af-muted);
  border: 1px solid var(--af-border);
}

.demo-btn:hover:not(.secondary) {
  opacity: 0.85;
}

.demo-btn.secondary:hover {
  background: hsl(var(--muted-foreground) / 0.05);
  color: var(--af-fg);
}

.demo-stage {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  padding: 0.5rem 1.25rem;
  overflow: hidden;
  min-height: 0;
}

.demo-panel {
  border: 1px solid var(--af-border);
  border-radius: 8px;
  padding: 0.75rem 1rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.panel-label {
  font-size: 0.65rem;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  color: var(--af-muted);
  margin-bottom: 0.5rem;
  flex-shrink: 0;
}

.demo-panel.raw {
  background: hsl(var(--muted-foreground) / 0.02);
}

.raw-text {
  font-size: 0.75rem;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  color: var(--af-fg);
  white-space: pre-wrap;
  word-break: break-word;
  margin: 0;
  overflow-wrap: break-word;
}

.demo-info {
  flex-shrink: 0;
  padding: 0.75rem 1.25rem;
  border-top: 1px solid var(--af-border);
  font-size: 0.8rem;
}

.demo-info h3 {
  font-size: 0.8rem;
  font-weight: 500;
  margin-bottom: 0.4rem;
}

.demo-info ul {
  margin: 0;
  padding-left: 1.25rem;
  color: var(--af-fg);
  line-height: 1.5;
}

.demo-info li {
  margin-bottom: 0.25rem;
}

.demo-info code {
  background: hsl(var(--muted-foreground) / 0.06);
  padding: 0.1rem 0.3rem;
  border-radius: 4px;
  font-size: 0.75rem;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

@media (max-width: 900px) {
  .demo-stage {
    grid-template-columns: 1fr;
    grid-template-rows: 1fr 1fr;
  }
}
</style>
