<template>
  <div class="code-view" :class="{ dark: isDark }" :data-example-id="exampleId">
    <div class="code-view-header">
      <div class="tabs">
        <button
          v-for="lang in availableLanguages"
          :key="lang.id"
          class="tab"
          :class="{ active: activeLang === lang.id }"
          @click="setLang(lang.id)"
        >
          {{ lang.label }}
        </button>
      </div>
      <div class="actions" v-if="isAuto && canRun">
        <button class="run-btn" @click="runCode" :disabled="isLoading">
          <Play v-if="!isLoading" :size="12" />
          <Loader2 v-else :size="12" class="spin" />
          {{ isLoading ? (zhLabels ? '运行中…' : 'Running...') : runLabel }}
        </button>
      </div>
    </div>
    <div class="code-view-body">
      <div ref="editorContainer" class="editor-pane"></div>
    </div>
    <div v-if="execution" class="input-pane">
      <label class="input-label" for="tapl-example-input">{{ zhLabels ? '程序输入' : 'Program input' }}</label>
      <textarea id="tapl-example-input" v-model="requestInput" rows="3" spellcheck="false" />
    </div>
    <div v-if="showOutput" class="output-pane">
      <div class="output-header">{{ execution?.mode === 'test' ? (zhLabels ? '测试结果' : 'Test results') : (zhLabels ? '输出' : 'Output') }}</div>
      <pre class="output-content">{{ testSummary || stdout || stderr || (zhLabels ? '无输出' : 'No output') }}</pre>
    </div>
    <details v-if="expectedText" class="expected-pane">
      <summary>{{ zhLabels ? '预期结果' : 'Expected result' }}</summary>
      <pre>{{ expectedText }}</pre>
    </details>
    <div v-if="caption" class="code-view-caption">{{ caption }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, onUnmounted, nextTick } from 'vue'
import { useRoute } from 'vitepress'
import { EditorState, Compartment, type Extension } from '@codemirror/state'
import { syntaxHighlighting, HighlightStyle } from '@codemirror/language'
import { tags } from '@lezer/highlight'
import { EditorView, keymap, lineNumbers } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { oneDarkTheme, oneDarkHighlightStyle } from '@codemirror/theme-one-dark'
import { rust } from '@codemirror/lang-rust'
import { cpp } from '@codemirror/lang-cpp'
import { javascript } from '@codemirror/lang-javascript'
import { python } from '@codemirror/lang-python'
import { autoLanguage } from 'auto-playground-vue/lang/auto'
import { Play, Loader2 } from 'lucide-vue-next'

const route = useRoute()
const zhLabels = computed(() => route.path.startsWith('/zh'))

const props = defineProps<{
  auto?: string
  rust?: string
  c?: string
  typescript?: string
  python?: string
  caption?: string
  runnable?: boolean
  apiUrl?: string
  exampleId?: string
  execution?: string
}>()

const activeLang = ref('auto')
const isLoading = ref(false)
const showOutput = ref(false)
const stdout = ref('')
const stderr = ref('')
const testSummary = ref('')
const execution = computed(() => {
  if (!props.execution) return undefined
  try {
    return JSON.parse(props.execution) as {
      id: string
      mode: string
      entry: string
      source: string
      files: { path: string; source: string }[]
      input: string
      timeout_secs: number
      expected?: { kind?: string; stdout?: string; value?: string; diagnostic?: string; contains?: string[] }
    }
  } catch {
    return undefined
  }
})
const requestInput = ref(execution.value?.input || '')
watch(execution, (next) => { requestInput.value = next?.input || '' })
const expectedText = computed(() => {
  const expected = execution.value?.expected
  if (!expected || expected.kind === 'pending') return ''
  if (expected.kind === 'stdout_contains') return (expected.contains || []).join('\n')
  return expected.stdout ?? expected.value ?? expected.diagnostic ?? ''
})
const canRun = computed(() => execution.value?.mode !== 'display-only' && (props.runnable ?? true))
const runLabel = computed(() => execution.value?.mode === 'test'
  ? (zhLabels.value ? '测试' : 'Test')
  : (zhLabels.value ? '运行' : 'Run'))
const editorContainer = ref<HTMLDivElement>()
let editorView: EditorView | null = null
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

const languages = [
  { id: 'auto', label: 'Auto', code: props.auto },
  { id: 'rust', label: 'Rust', code: props.rust },
  { id: 'c', label: 'C', code: props.c },
  { id: 'typescript', label: 'TypeScript', code: props.typescript },
  { id: 'python', label: 'Python', code: props.python },
]

const availableLanguages = computed(() =>
  languages.filter((lang) => lang.code && lang.code.trim().length > 0)
)

const isAuto = computed(() => activeLang.value === 'auto')

const currentLang = computed(() =>
  languages.find((l) => l.id === activeLang.value)
)

function checkTheme() {
  isDark.value = document.documentElement.classList.contains('dark')
}

let observer: MutationObserver | null = null

onMounted(() => {
  checkTheme()
  if (currentLang.value?.code) {
    initEditor(currentLang.value.code, currentLang.value.id)
  }
  observer = new MutationObserver(checkTheme)
  observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] })
})

onUnmounted(() => {
  editorView?.destroy()
  observer?.disconnect()
})

async function setLang(lang: string) {
  activeLang.value = lang
  const langData = languages.find((l) => l.id === lang)
  if (langData?.code) {
    await nextTick()
    initEditor(langData.code, langData.id)
  }
}

function getLanguageExtension(langId: string): Extension {
  switch (langId) {
    case 'auto':
      return autoLanguage
    case 'rust':
      return rust()
    case 'c':
      return cpp()
    case 'typescript':
      return javascript({ typescript: true })
    case 'python':
      return python()
    default:
      return []
  }
}

function initEditor(code: string, langId: string) {
  if (editorView) {
    editorView.destroy()
    editorView = null
  }

  if (!editorContainer.value) return

  const isReadOnly = langId !== 'auto'

  const extensions: Extension[] = [
    lineNumbers(),
    history(),
    keymap.of([...defaultKeymap, ...historyKeymap]),
    highlightCompartment.of(syntaxHighlighting(isDark.value ? oneDarkHighlightStyle : lightHighlightStyle)),
    themeCompartment.of(isDark.value ? oneDarkTheme : []),
    getLanguageExtension(langId),
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        // Track changes if needed
      }
    }),
  ]

  if (isReadOnly) {
    extensions.push(EditorView.editable.of(false))
    extensions.push(EditorState.readOnly.of(true))
  }

  watch(isDark, (dark) => {
    editorView?.dispatch({
      effects: [
        highlightCompartment.reconfigure(syntaxHighlighting(dark ? oneDarkHighlightStyle : lightHighlightStyle)),
        themeCompartment.reconfigure(dark ? oneDarkTheme : []),
      ],
    })
  })

  const state = EditorState.create({
    doc: code,
    extensions,
  })

  editorView = new EditorView({
    state,
    parent: editorContainer.value,
  })
}

async function runCode() {
  const code = editorView?.state.doc.toString() || props.auto || ''
  if (!code || isLoading.value) return

  isLoading.value = true
  showOutput.value = true
  stdout.value = ''
  stderr.value = ''
  testSummary.value = ''

  try {
    const descriptor = execution.value
    const files = descriptor?.files.map((file) => ({
      ...file,
      source: file.path === descriptor.entry ? code : file.source,
    })) || []
    const request: Record<string, unknown> = { source: code }
    if (descriptor) {
      request.entry = descriptor.entry
      request.files = files
      request.input = requestInput.value
      request.timeout_secs = descriptor.timeout_secs
    }
    const apiBase = (props.apiUrl || '').replace(/\/$/, '')
    const endpoint = descriptor?.mode === 'test' ? 'test' : 'run'
    const res = await fetch(`${apiBase}/api/${endpoint}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    })
    const data = await res.json()
    if (!res.ok) {
      stderr.value = data.message || data.error || `HTTP ${res.status}`
    } else if (descriptor?.mode === 'test') {
      const reports = Array.isArray(data.reports) ? data.reports : []
      testSummary.value = `${data.passed ?? 0}/${data.executed ?? 0} ${zhLabels.value ? '项通过' : 'tests passed'}${data.failed ? ` · ${data.failed} ${zhLabels.value ? '项失败' : 'failed'}` : ''}${reports.map((report: { qualified_name?: string; error?: string }) => report.error ? `\n${report.qualified_name || ''}: ${report.error}` : '').filter(Boolean).join('')}`
    } else {
      stdout.value = data.stdout || ''
      stderr.value = data.stderr || ''
    }
  } catch (e) {
    stderr.value = zhLabels.value
      ? '错误：无法连接 Playground 服务。'
      : 'Error: Could not connect to playground server.'
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.code-view {
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  margin: 1.5rem 0;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.code-view-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: hsl(var(--muted));
  border-bottom: 1px solid hsl(var(--border));
}

.tabs {
  display: flex;
  gap: 2px;
}

.tab {
  padding: 0.35rem 0.75rem;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: hsl(var(--muted-foreground));
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.tab:hover {
  color: hsl(var(--foreground));
  background: hsl(var(--accent));
}

.tab.active {
  color: hsl(var(--foreground));
  background: hsl(var(--accent));
}

.actions {
  display: flex;
  gap: 0.5rem;
}

.run-btn {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.35rem 0.75rem;
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.2s;
}

.run-btn:hover {
  opacity: 0.9;
}

.run-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.code-view-body {
  min-height: 120px;
}

.input-pane {
  padding: 0.6rem 0.75rem;
  border-top: 1px solid hsl(var(--border));
}

.input-label {
  display: block;
  margin-bottom: 0.35rem;
  color: hsl(var(--muted-foreground));
  font-size: 0.72rem;
}

.input-pane textarea {
  box-sizing: border-box;
  width: 100%;
  resize: vertical;
  padding: 0.5rem;
  border: 1px solid hsl(var(--border));
  border-radius: 5px;
  background: hsl(var(--background));
  color: hsl(var(--foreground));
  font: inherit;
  font-size: 0.8rem;
}

.editor-pane {
  min-height: 120px;
}

.editor-pane :deep(.cm-editor) {
  min-height: 120px;
  font-size: 13px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.editor-pane :deep(.cm-gutters) {
  background: #f5f5f5;
  border-right: 1px solid #e2e2e3;
}

.editor-pane :deep(.cm-scroller) {
  background: #ffffff;
}

.code-view.dark .editor-pane :deep(.cm-gutters) {
  background: #181825;
  border-right: 1px solid #313244;
}

.code-view.dark .editor-pane :deep(.cm-scroller) {
  background: #1e1e2e;
}

.output-pane {
  border-top: 1px solid hsl(var(--border));
  background: hsl(var(--muted));
}

.output-header {
  padding: 0.4rem 0.75rem;
  font-size: 0.75rem;
  font-weight: 600;
  color: hsl(var(--muted-foreground));
  border-bottom: 1px solid hsl(var(--border));
}

.output-content {
  margin: 0;
  padding: 0.75rem;
  font-size: 0.8rem;
  line-height: 1.5;
  color: hsl(var(--foreground));
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 200px;
  overflow-y: auto;
}

.expected-pane {
  padding: 0.5rem 0.75rem;
  color: hsl(var(--muted-foreground));
  font-size: 0.78rem;
}

.expected-pane pre {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.code-view-caption {
  padding: 0.5rem 0.75rem;
  font-size: 0.8rem;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted));
  border-top: 1px solid hsl(var(--border));
  font-style: italic;
}
</style>
