<template>
  <div class="code-view" :class="{ dark: isDark }">
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
      <div class="actions" v-if="isAuto">
        <button class="run-btn" @click="runCode" :disabled="isLoading">
          <Play v-if="!isLoading" :size="12" />
          <Loader2 v-else :size="12" class="spin" />
          {{ isLoading ? 'Running...' : 'Run' }}
        </button>
      </div>
    </div>
    <div class="code-view-body">
      <div ref="editorContainer" v-show="isAuto" class="editor-pane"></div>
      <pre v-show="!isAuto" class="code-pane"><code>{{ currentCode }}</code></pre>
    </div>
    <div v-if="showOutput" class="output-pane">
      <div class="output-header">Output</div>
      <pre class="output-content">{{ stdout || stderr || 'No output' }}</pre>
    </div>
    <div v-if="caption" class="code-view-caption">{{ caption }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, onUnmounted, nextTick } from 'vue'
import { EditorState, type Extension } from '@codemirror/state'
import { EditorView, keymap, lineNumbers } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { oneDark } from '@codemirror/theme-one-dark'
import { Play, Loader2 } from 'lucide-vue-next'

const props = defineProps<{
  auto?: string
  rust?: string
  c?: string
  typescript?: string
  python?: string
  caption?: string
  runnable?: boolean
  apiUrl?: string
}>()

const activeLang = ref('auto')
const isLoading = ref(false)
const showOutput = ref(false)
const stdout = ref('')
const stderr = ref('')
const editorContainer = ref<HTMLDivElement>()
let editorView: EditorView | null = null
const isDark = ref(false)

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

const currentCode = computed(() => {
  const lang = languages.find((l) => l.id === activeLang.value)
  return lang?.code || ''
})

function checkTheme() {
  isDark.value = document.documentElement.classList.contains('dark')
}

let observer: MutationObserver | null = null

onMounted(() => {
  checkTheme()
  if (props.auto) {
    initEditor(props.auto)
  }
  // Watch for theme changes
  observer = new MutationObserver(checkTheme)
  observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] })
})

onUnmounted(() => {
  editorView?.destroy()
  observer?.disconnect()
})

watch(() => props.auto, (newCode) => {
  if (editorView && newCode) {
    editorView.dispatch({
      changes: { from: 0, to: editorView.state.doc.length, insert: newCode },
    })
  }
})

async function setLang(lang: string) {
  activeLang.value = lang
  if (lang === 'auto' && props.auto) {
    await nextTick()
    initEditor(props.auto)
  }
}

function initEditor(code: string) {
  if (editorView) {
    editorView.destroy()
    editorView = null
  }

  if (!editorContainer.value) return

  const extensions: Extension[] = [
    lineNumbers(),
    history(),
    keymap.of([...defaultKeymap, ...historyKeymap]),
    oneDark,
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        // Track changes if needed
      }
    }),
  ]

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

  try {
    const res = await fetch(`${props.apiUrl || 'http://localhost:3030'}/api/run`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source: code }),
    })
    const data = await res.json()
    stdout.value = data.stdout || ''
    stderr.value = data.stderr || ''
  } catch (e) {
    stderr.value = 'Error: Could not connect to playground server.'
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

.editor-pane {
  min-height: 120px;
}

.editor-pane :deep(.cm-editor) {
  min-height: 120px;
  font-size: 13px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.editor-pane :deep(.cm-gutters) {
  background: #181825;
  border-right: 1px solid #313244;
}

.code-pane {
  margin: 0;
  padding: 1rem;
  font-size: 13px;
  line-height: 1.6;
  color: #cdd6f4;
  overflow-x: auto;
  background: #1e1e2e;
  min-height: 120px;
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

.code-view-caption {
  padding: 0.5rem 0.75rem;
  font-size: 0.8rem;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted));
  border-top: 1px solid hsl(var(--border));
  font-style: italic;
}
</style>
