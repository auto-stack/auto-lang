<template>
  <div class="ship-view" :class="{ dark: isDark }">
    <div class="ship-header">
      <div class="ship-title">
        <span class="ship-badge ship-badge-dev">Dev</span>
        <span class="ship-arrow">→</span>
        <span class="ship-badge ship-badge-ship">Ship</span>
      </div>
      <div class="ship-actions">
        <button
          class="ship-btn ship-btn-primary"
          @click="runInVm"
          :disabled="vmLoading"
        >
          <Play v-if="!vmLoading" :size="12" />
          <Loader2 v-else :size="12" class="spin" />
          {{ vmLoading ? 'Running...' : 'Run in VM' }}
        </button>
        <button
          v-if="showRust"
          class="ship-btn ship-btn-secondary"
          @click="transpileToRust"
          :disabled="transpileLoading"
        >
          <RefreshCw v-if="!transpileLoading" :size="12" />
          <Loader2 v-else :size="12" class="spin" />
          {{ transpileLoading ? 'Transpiling...' : 'Transpile to Rust' }}
        </button>
        <button
          v-if="compareRun"
          class="ship-btn ship-btn-compare"
          @click="runBoth"
          :disabled="compareLoading"
        >
          <GitCompare v-if="!compareLoading" :size="12" />
          <Loader2 v-else :size="12" class="spin" />
          {{ compareLoading ? 'Comparing...' : 'Run Both & Compare' }}
        </button>
      </div>
    </div>

    <div class="ship-body">
      <!-- Left: Auto source editor -->
      <div class="ship-pane ship-pane-auto">
        <div class="ship-pane-label">
          <Terminal :size="12" /> Auto source
        </div>
        <div ref="autoContainer" class="ship-editor"></div>
      </div>

      <!-- Middle: Transpiled Rust (read-only) -->
      <div v-if="showRust" class="ship-pane ship-pane-rust">
        <div class="ship-pane-label">
          <FileCode2 :size="12" /> Rust (a2r output)
        </div>
        <div ref="rustContainer" class="ship-editor">
          <div v-if="!rustCode" class="ship-empty">
            Click "Transpile to Rust" to see the transpiled code.
          </div>
        </div>
      </div>
    </div>

    <!-- Output + compare row -->
    <div class="ship-outputs" :class="{ 'with-compare': compareRun }">
      <div class="ship-output">
        <div class="ship-output-header">VM output</div>
        <pre class="ship-output-content">{{ vmOutput || (vmLoading ? '...' : 'No output yet') }}</pre>
      </div>

      <div v-if="compareRun" class="ship-output">
        <div class="ship-output-header">
          Rust output
          <span
            v-if="consistent !== null"
            class="ship-consistent"
            :class="consistent ? 'ok' : 'diff'"
          >
            {{ consistent ? '✓ Consistent' : '✗ Differ' }}
          </span>
        </div>
        <pre class="ship-output-content">{{ rustOutput || (rustLoading ? '...' : 'No output yet') }}</pre>
      </div>
    </div>

    <div v-if="caption" class="ship-caption">{{ caption }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { EditorState, Compartment, type Extension } from '@codemirror/state'
import { syntaxHighlighting, HighlightStyle } from '@codemirror/language'
import { tags } from '@lezer/highlight'
import { EditorView, keymap, lineNumbers } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { oneDarkTheme, oneDarkHighlightStyle } from '@codemirror/theme-one-dark'
import { rust } from '@codemirror/lang-rust'
import { autoLanguage } from 'auto-playground-vue'
import { Play, Loader2, RefreshCw, GitCompare, Terminal, FileCode2 } from 'lucide-vue-next'

const props = withDefaults(
  defineProps<{
    auto: string
    caption?: string
    // Match CodeView.vue default — direct backend URL.
    // VitePress dev also proxies '/api' -> 3030 (see .vitepress/config.ts),
    // so '/api' would also work; we keep 'http://localhost:3030' for parity.
    apiUrl?: string
    showRust?: boolean
    compareRun?: boolean
  }>(),
  {
    apiUrl: 'http://localhost:3030',
    showRust: true,
    compareRun: false,
  },
)

const apiBase = props.apiUrl

// --- reactive state ---
const vmOutput = ref('')
const rustCode = ref('')
const rustOutput = ref('')
const consistent = ref<boolean | null>(null)

const vmLoading = ref(false)
const transpileLoading = ref(false)
const rustLoading = ref(false)
const compareLoading = ref(false)

// --- editor refs ---
const autoContainer = ref<HTMLDivElement>()
const rustContainer = ref<HTMLDivElement>()
let autoEditor: EditorView | null = null
let rustEditor: EditorView | null = null

const isDark = ref(false)
const themeCompartment = new Compartment()
const highlightCompartment = new Compartment()

const lightHighlightStyle = HighlightStyle.define([
  { tag: tags.keyword, color: '#a626a4' },
  { tag: tags.string, color: '#50a14f' },
  { tag: tags.number, color: '#986801' },
  { tag: tags.comment, color: '#a0a1a7' },
  { tag: tags.typeName, color: '#c18401' },
  { tag: tags.variableName, color: '#e45649' },
  { tag: tags.operator, color: '#0184bc' },
  { tag: tags.propertyName, color: '#4078f2' },
  { tag: tags.attributeName, color: '#c18401' },
  { tag: tags.macroName, color: '#0184bc' },
])

function checkTheme() {
  isDark.value = document.documentElement.classList.contains('dark')
}

let observer: MutationObserver | null = null

onMounted(() => {
  checkTheme()
  initAutoEditor()
  observer = new MutationObserver(checkTheme)
  observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] })
})

onUnmounted(() => {
  autoEditor?.destroy()
  rustEditor?.destroy()
  observer?.disconnect()
})

function reconfigureTheme() {
  const reconf = [
    highlightCompartment.reconfigure(
      syntaxHighlighting(isDark.value ? oneDarkHighlightStyle : lightHighlightStyle),
    ),
    themeCompartment.reconfigure(isDark.value ? oneDarkTheme : []),
  ]
  autoEditor?.dispatch({ effects: reconf })
  rustEditor?.dispatch({ effects: reconf })
}

watch(isDark, () => reconfigureTheme())

function buildExtensions(languageExt: Extension, readOnly: boolean): Extension[] {
  const ext: Extension[] = [
    lineNumbers(),
    history(),
    keymap.of([...defaultKeymap, ...historyKeymap]),
    highlightCompartment.of(
      syntaxHighlighting(isDark.value ? oneDarkHighlightStyle : lightHighlightStyle),
    ),
    themeCompartment.of(isDark.value ? oneDarkTheme : []),
    languageExt,
  ]
  if (readOnly) {
    ext.push(EditorView.editable.of(false))
    ext.push(EditorState.readOnly.of(true))
  }
  return ext
}

function initAutoEditor() {
  autoEditor?.destroy()
  autoEditor = null
  if (!autoContainer.value) return
  const state = EditorState.create({
    doc: props.auto,
    extensions: buildExtensions(autoLanguage, false),
  })
  autoEditor = new EditorView({ state, parent: autoContainer.value })
}

async function initOrUpdateRustEditor() {
  await nextTick()
  if (!rustCode.value || !rustContainer.value) return
  if (rustEditor) {
    rustEditor.dispatch({
      changes: { from: 0, to: rustEditor.state.doc.length, insert: rustCode.value },
    })
    return
  }
  const state = EditorState.create({
    doc: rustCode.value,
    extensions: buildExtensions(rust(), true),
  })
  rustEditor = new EditorView({ state, parent: rustContainer.value })
}

function currentAutoCode(): string {
  return autoEditor?.state.doc.toString() ?? props.auto
}

// --- VM run: POST /api/run ---
async function runInVm() {
  const source = currentAutoCode()
  if (!source || vmLoading.value) return
  vmLoading.value = true
  vmOutput.value = ''
  try {
    const res = await fetch(`${apiBase}/api/run`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source }),
    })
    const data = await res.json()
    // RunResponse: { stdout, result, time_ms, bytecode }
    vmOutput.value = data.stdout || data.result || ''
  } catch (e) {
    vmOutput.value = 'Error: Could not connect to playground server.'
  } finally {
    vmLoading.value = false
  }
}

// --- a2r transpile: POST /api/trans ---
async function transpileToRust() {
  const source = currentAutoCode()
  if (!source || transpileLoading.value) return
  transpileLoading.value = true
  consistent.value = null
  try {
    const res = await fetch(`${apiBase}/api/trans`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source, target: 'rust' }),
    })
    const data = await res.json()
    // TransResponse: { target, files: [{path, code}], source_map }
    const files: Array<{ path: string; code: string }> = data.files || []
    const mainFile =
      files.find((f) => f.path.endsWith('main.rs')) ||
      files.find((f) => f.path.endsWith('.rs')) ||
      files[0]
    rustCode.value = mainFile?.code ?? ''
    await initOrUpdateRustEditor()
  } catch (e) {
    rustCode.value = 'Error: Could not connect to playground server.'
    await initOrUpdateRustEditor()
  } finally {
    transpileLoading.value = false
  }
}

// --- Rust run: POST /api/run_code (supports language: "rust" via rustc) ---
async function runRust() {
  if (!rustCode.value || rustLoading.value) return
  rustLoading.value = true
  rustOutput.value = ''
  try {
    const res = await fetch(`${apiBase}/api/run_code`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ language: 'rust', code: rustCode.value }),
    })
    const data = await res.json()
    // RunCodeResponse: { stdout, stderr, exit_code, time_ms }
    rustOutput.value = data.stdout || data.stderr || ''
  } catch (e) {
    rustOutput.value = 'Error: Could not connect to playground server.'
  } finally {
    rustLoading.value = false
  }
}

// --- compare: run VM + Rust, then diff outputs ---
async function runBoth() {
  if (compareLoading.value) return
  compareLoading.value = true
  consistent.value = null
  try {
    // Ensure we have Rust code first.
    if (!rustCode.value) {
      await transpileToRust()
    }
    // Run both in parallel for snappy UX.
    await Promise.all([runInVm(), runRust()])
    // Compare after both settled. Use nextTick to ensure refs updated.
    await nextTick()
    consistent.value = (vmOutput.value.trim() === rustOutput.value.trim())
  } finally {
    compareLoading.value = false
  }
}
</script>

<style scoped>
.ship-view {
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  margin: 1.5rem 0;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.ship-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  flex-wrap: wrap;
  padding: 0.5rem 0.75rem;
  background: hsl(var(--muted));
  border-bottom: 1px solid hsl(var(--border));
}

.ship-title {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.8rem;
  font-weight: 600;
}

.ship-badge {
  padding: 0.15rem 0.5rem;
  border-radius: 4px;
  font-size: 0.7rem;
  letter-spacing: 0.03em;
  text-transform: uppercase;
}

.ship-badge-dev {
  background: rgba(99, 102, 241, 0.18);
  color: #6366f1;
}

.ship-badge-ship {
  background: rgba(244, 114, 93, 0.18);
  color: #f4625d;
}

.ship-arrow {
  color: hsl(var(--muted-foreground));
  font-size: 0.9rem;
}

.ship-actions {
  display: flex;
  gap: 0.4rem;
  flex-wrap: wrap;
}

.ship-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.35rem 0.75rem;
  border: none;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.2s;
  color: white;
}

.ship-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.ship-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.ship-btn-primary {
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
}

.ship-btn-secondary {
  background: linear-gradient(135deg, #f4625d 0%, #f59e0b 100%);
}

.ship-btn-compare {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.ship-body {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0;
}

@media (min-width: 768px) {
  .ship-body.with-rust-cols {
    grid-template-columns: 1fr 1fr;
  }
}

/* Always allow 2-col when showRust (middle column visible). */
.ship-body:has(.ship-pane-rust) {
  grid-template-columns: 1fr;
}

@media (min-width: 900px) {
  .ship-body:has(.ship-pane-rust) {
    grid-template-columns: 1fr 1fr;
  }
}

.ship-pane {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.ship-pane-rust {
  border-top: 1px solid hsl(var(--border));
}

@media (min-width: 900px) {
  .ship-pane-rust {
    border-top: none;
    border-left: 1px solid hsl(var(--border));
  }
}

.ship-pane-label {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.35rem 0.75rem;
  font-size: 0.7rem;
  font-weight: 600;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted));
  border-bottom: 1px solid hsl(var(--border));
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.ship-editor {
  min-height: 180px;
  position: relative;
}

.ship-editor :deep(.cm-editor) {
  min-height: 180px;
  font-size: 13px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.ship-editor :deep(.cm-gutters) {
  background: #f5f5f5;
  border-right: 1px solid #e2e2e3;
}

.ship-editor :deep(.cm-scroller) {
  background: #ffffff;
}

.ship-view.dark .ship-editor :deep(.cm-gutters) {
  background: #181825;
  border-right: 1px solid #313244;
}

.ship-view.dark .ship-editor :deep(.cm-scroller) {
  background: #1e1e2e;
}

.ship-empty {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  text-align: center;
  font-size: 0.8rem;
  color: hsl(var(--muted-foreground));
  font-family: inherit;
  font-style: italic;
}

.ship-outputs {
  display: grid;
  grid-template-columns: 1fr;
  border-top: 1px solid hsl(var(--border));
}

@media (min-width: 900px) {
  .ship-outputs.with-compare {
    grid-template-columns: 1fr 1fr;
  }
}

.ship-output {
  border-right: none;
  border-bottom: 1px solid hsl(var(--border));
  background: hsl(var(--muted));
}

.ship-outputs .ship-output:last-child {
  border-bottom: none;
}

@media (min-width: 900px) {
  .ship-outputs.with-compare .ship-output:first-child {
    border-right: 1px solid hsl(var(--border));
  }
  .ship-outputs.with-compare .ship-output {
    border-bottom: none;
  }
}

.ship-output-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.4rem 0.75rem;
  font-size: 0.72rem;
  font-weight: 600;
  color: hsl(var(--muted-foreground));
  border-bottom: 1px solid hsl(var(--border));
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.ship-output-content {
  margin: 0;
  padding: 0.75rem;
  font-size: 0.8rem;
  line-height: 1.5;
  color: hsl(var(--foreground));
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 220px;
  overflow-y: auto;
}

.ship-consistent {
  padding: 0.1rem 0.45rem;
  border-radius: 4px;
  font-size: 0.68rem;
  font-weight: 700;
}

.ship-consistent.ok {
  background: rgba(16, 185, 129, 0.18);
  color: #10b981;
}

.ship-consistent.diff {
  background: rgba(239, 68, 68, 0.18);
  color: #ef4444;
}

.ship-caption {
  padding: 0.5rem 0.75rem;
  font-size: 0.8rem;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted));
  border-top: 1px solid hsl(var(--border));
  font-style: italic;
}
</style>
