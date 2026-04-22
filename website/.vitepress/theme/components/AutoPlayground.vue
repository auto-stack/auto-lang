<template>
  <div class="playground-wrapper">
    <div class="playground-toolbar">
      <div class="toolbar-left">
        <Code2 :size="16" />
        <span class="toolbar-title">Auto Playground</span>
        <select v-model="targetLang" class="target-select">
          <option value="run">Run</option>
          <option value="rust">→ Rust</option>
          <option value="c">→ C</option>
          <option value="typescript">→ TypeScript</option>
        </select>
      </div>
      <button class="run-btn" @click="runCode" :disabled="isLoading">
        <Play v-if="!isLoading" :size="14" />
        <Loader2 v-else :size="14" class="spin" />
        {{ isLoading ? 'Running...' : 'Run' }}
      </button>
    </div>
    <div class="playground-body">
      <div ref="editorContainer" class="editor-pane" />
      <div class="output-pane">
        <div class="output-tabs">
          <button
            v-for="tab in tabs"
            :key="tab"
            class="tab-btn"
            :class="{ active: activeTab === tab }"
            @click="activeTab = tab"
          >
            {{ tab }}
          </button>
        </div>
        <div class="output-content">
          <pre v-if="activeTab === 'Output'" class="output-text">{{ stdout || stderr || 'Click Run to see output...' }}</pre>
          <pre v-else-if="activeTab === 'Transpiled'" class="output-text">{{ transpiledCode || 'Select a transpile target and click Run...' }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, onUnmounted } from 'vue'
import { EditorState, type Extension } from '@codemirror/state'
import { EditorView, keymap, lineNumbers, highlightActiveLine } from '@codemirror/view'
import { defaultKeymap, indentWithTab, history, historyKeymap } from '@codemirror/commands'
import { oneDark } from '@codemirror/theme-one-dark'
import { Play, Loader2, Code2 } from 'lucide-vue-next'

const props = withDefaults(defineProps<{
  code?: string
  apiUrl?: string
}>(), {
  code: `fn main() {
    let message = "Hello from Auto!";
    println(message);
}`,
  apiUrl: 'http://localhost:3030',
})

const editorContainer = ref<HTMLDivElement>()
let editorView: EditorView | null = null

const isLoading = ref(false)
const activeTab = ref('Output')
const tabs = ['Output', 'Transpiled']
const targetLang = ref('run')
const stdout = ref('')
const stderr = ref('')
const transpiledCode = ref('')

onMounted(() => {
  if (!editorContainer.value) return

  const extensions: Extension[] = [
    lineNumbers(),
    highlightActiveLine(),
    history(),
    keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
    oneDark,
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        // code changed
      }
    }),
    keymap.of([{
      key: 'Ctrl-Enter',
      run: () => { runCode(); return true; }
    }]),
  ]

  const state = EditorState.create({
    doc: props.code,
    extensions,
  })

  editorView = new EditorView({
    state,
    parent: editorContainer.value,
  })
})

onUnmounted(() => {
  editorView?.destroy()
})

async function runCode() {
  if (!editorView || isLoading.value) return
  const code = editorView.state.doc.toString()
  isLoading.value = true
  stdout.value = ''
  stderr.value = ''
  transpiledCode.value = ''

  try {
    if (targetLang.value === 'run') {
      const res = await fetch(`${props.apiUrl}/api/run`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ source: code }),
      })
      const data = await res.json()
      stdout.value = data.stdout || ''
      stderr.value = data.stderr || ''
    } else {
      const res = await fetch(`${props.apiUrl}/api/trans`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ source: code, target: targetLang.value }),
      })
      const data = await res.json()
      transpiledCode.value = data.code || data.error || ''
      activeTab.value = 'Transpiled'
    }
  } catch (e) {
    stderr.value = 'Error: Could not connect to playground server.\nMake sure the backend is running on ' + props.apiUrl
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.playground-wrapper {
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid hsl(var(--border));
  background: #1e1e2e;
  margin: 1.5rem 0;
}

.playground-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: #181825;
  border-bottom: 1px solid #313244;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: #cdd6f4;
}

.toolbar-title {
  font-size: 0.85rem;
  font-weight: 600;
  font-family: 'JetBrains Mono', monospace;
}

.target-select {
  background: #313244;
  color: #cdd6f4;
  border: 1px solid #45475a;
  border-radius: 6px;
  padding: 0.25rem 0.5rem;
  font-size: 0.8rem;
  font-family: 'JetBrains Mono', monospace;
  cursor: pointer;
}

.run-btn {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.4rem 0.8rem;
  background: #27c93f;
  color: #1e1e2e;
  border: none;
  border-radius: 6px;
  font-size: 0.8rem;
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

.playground-body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  min-height: 300px;
}

.editor-pane {
  border-right: 1px solid #313244;
}

.editor-pane :deep(.cm-editor) {
  height: 100%;
  min-height: 300px;
  font-size: 13px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.output-pane {
  display: flex;
  flex-direction: column;
}

.output-tabs {
  display: flex;
  background: #181825;
  border-bottom: 1px solid #313244;
}

.tab-btn {
  padding: 0.4rem 0.8rem;
  background: none;
  border: none;
  color: #6c7086;
  font-size: 0.8rem;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}

.tab-btn.active {
  color: #cdd6f4;
  border-bottom-color: #6366f1;
}

.tab-btn:hover {
  color: #cdd6f4;
}

.output-content {
  flex: 1;
  padding: 0.75rem;
  overflow: auto;
}

.output-text {
  margin: 0;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 0.8rem;
  line-height: 1.5;
  color: #cdd6f4;
  white-space: pre-wrap;
  word-break: break-word;
}

@media (max-width: 768px) {
  .playground-body {
    grid-template-columns: 1fr;
  }
  .editor-pane {
    border-right: none;
    border-bottom: 1px solid #313244;
  }
}
</style>
