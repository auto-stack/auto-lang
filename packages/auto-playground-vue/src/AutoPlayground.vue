<template>
  <div class="playground-wrapper" :style="wrapperStyle">
    <div class="playground-toolbar">
      <div class="toolbar-left">
        <Code2 :size="16" />
        <span class="toolbar-title">Auto Playground</span>
        <ExampleSelector :api-base="apiBase" @select="onLoadExample" />
      </div>
      <div class="toolbar-right">
        <select v-model="targetLang" class="target-select" :disabled="isDebugging">
          <option value="run">Run</option>
          <option value="rust">→ Rust</option>
          <option value="c">→ C</option>
          <option value="python">→ Python</option>
          <option value="typescript">→ TypeScript</option>
          <option value="abt">→ ABT</option>
        </select>
        <button v-if="!isDebugging" class="run-btn" @click="runAction" :disabled="isLoading">
          <Play v-if="!isLoading" :size="14" />
          <Loader2 v-else :size="14" class="spin" />
          {{ isLoading ? 'Running...' : 'Run' }}
        </button>
        <template v-else>
          <div class="debug-controls">
            <button class="debug-btn continue" @click="debugCommand('continue')" :disabled="isLoading" title="Continue">
              <Play :size="14" />
            </button>
            <button class="debug-btn step" @click="debugCommand('step')" :disabled="isLoading" title="Step Into">
              <ArrowDown :size="14" />
            </button>
            <button class="debug-btn step-over" @click="debugCommand('step_over')" :disabled="isLoading" title="Step Over">
              <SkipForward :size="14" />
            </button>
            <button class="debug-btn step-out" @click="debugCommand('step_out')" :disabled="isLoading" title="Step Out">
              <ArrowUp :size="14" />
            </button>
          </div>
        </template>
        <button v-if="isDebugging" class="stop-btn" @click="onDebugStop" title="Stop Debug">
          <Square :size="14" />
          Stop
        </button>
        <button v-else class="debug-start-btn" @click="onDebugStart" :disabled="isLoading" title="Start Debug">
          <Bug :size="14" />
          Debug
        </button>
        <label v-if="!isDebugging" class="switch-widget" title="Toggle live transpile on edit">
          <span class="switch-label">Live</span>
          <span class="switch">
            <input type="checkbox" v-model="liveCompile" />
            <span class="slider"></span>
          </span>
        </label>
        <button class="icon-btn share-btn" @click="share" title="Copy shareable link">
          <Share2 :size="14" />
        </button>
      </div>
    </div>
    <div class="playground-body">
      <div class="editor-pane">
        <CodeEditor
          :model-value="source"
          @update:model-value="source = $event"
          :on-run="runAction"
          :is-debugging="isDebugging"
          :breakpoints="breakpoints"
          :current-debug-line="debugState?.line ?? null"
          @breakpoints-change="onBreakpointsChange"
        />
      </div>
      <div class="output-pane">
        <div class="output-tabs">
          <button
            v-for="tab in tabs"
            :key="tab"
            class="tab-btn"
            :class="{ active: displayTab === tab }"
            @click="onSwitchTab(tab)"
          >
            {{ tabLabels[tab] }}
          </button>
          <div class="spacer" />
          <span v-if="isDebugging && debugState" class="debug-status" :class="debugState.status">
            {{ debugState.status }}
          </span>
          <button
            v-if="canCopy"
            class="icon-btn copy-btn"
            @click="copyCode"
            :title="copied ? 'Copied!' : 'Copy code'"
          >
            <Copy v-if="!copied" :size="14" />
            <Check v-else :size="14" />
          </button>
        </div>
        <div class="output-content">
          <ConsoleOutput
            v-if="displayTab === 'Output'"
            :stdout="stdout"
            :stderr="stderr"
            :result="resultCode"
            :time-ms="timeMs"
          />
          <BytecodePanel
            v-else-if="displayTab === 'Bytecode'"
            :bytecode="bytecode"
            :current-ip="debugState?.ip"
            @offset-click="onBytecodeOffsetClick"
          />
          <CodePreview
            v-else
            :code="transpiledCode"
            :language="displayTab"
            :highlight-lines="highlightedOutputLines"
          />
        </div>
        <!-- Debug state panel -->
        <div v-if="isDebugging && debugState" class="debug-panel">
          <div class="debug-section" v-if="debugState.stack.length">
            <div class="debug-section-title">Stack ({{ debugState.stack.length }})</div>
            <div class="debug-stack">
              <span v-for="(val, i) in debugState.stack.slice(-8)" :key="i" class="stack-item">{{ val }}</span>
            </div>
          </div>
          <div class="debug-section" v-if="debugState.call_stack.length">
            <div class="debug-section-title">Call Stack</div>
            <div v-for="(frame, i) in debugState.call_stack" :key="i" class="call-frame">
              <span class="frame-name">{{ frame.fn_name || '&lt;root&gt;' }}</span>
              <span class="frame-info">line {{ frame.line }}, bp={{ frame.bp }}</span>
            </div>
          </div>
          <div class="debug-section" v-if="debugState.locals.length">
            <div class="debug-section-title">Locals</div>
            <div class="debug-locals">
              <span v-for="(local, i) in debugState.locals" :key="i" class="local-item">
                <span class="local-idx">[{{ local.index }}]</span> {{ local.value }}
              </span>
            </div>
          </div>
          <div class="debug-registers" v-if="debugState.registers">
            IP={{ debugState.registers.ip }} BP={{ debugState.registers.bp }} SP={{ debugState.registers.sp }}
          </div>
        </div>
      </div>
    </div>
  </div>
  <div class="toast" :class="{ visible: shareToast.visible }">
    {{ shareToast.message }}
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { Play, Loader2, Code2, Share2, Copy, Check, Bug, Square, ArrowDown, ArrowUp, SkipForward } from 'lucide-vue-next'
import CodeEditor from './components/CodeEditor.vue'
import CodePreview from './components/CodePreview.vue'
import ConsoleOutput from './components/ConsoleOutput.vue'
import BytecodePanel from './components/BytecodePanel.vue'
import ExampleSelector from './components/ExampleSelector.vue'
import { usePlayground } from './composables/usePlayground'
import type { OutputTab, DebugCommand } from './types'

const props = withDefaults(defineProps<{
  code?: string
  apiUrl?: string
  height?: string
}>(), {
  code: `fn main() {
    let message = "Hello from Auto!"
    print(message)
}`,
  apiUrl: '',
  height: '500px'
})

const apiBase = props.apiUrl ? `${props.apiUrl}/api` : '/api'

const {
  source, stdout, stderr, resultCode, timeMs, isLoading,
  transpiledCode, liveCompile,
  highlightedOutputLines, shareToast,
  debugState, bytecode, breakpoints, isDebugging,
  run, switchTab, loadExample, share,
  debugStart, debugSetBreakpoints, debugCommand, debugStop,
} = usePlayground({
  apiBase,
  defaultSource: props.code,
  persistKey: false,
  preloadTargets: false,
})

const displayTab = ref<EmbedTab>('Output')
const targetLang = ref<'run' | Exclude<OutputTab, 'bytecode'>>('run')
const copied = ref(false)

const tabs = ['Output', 'rust', 'c', 'python', 'typescript', 'abt', 'Bytecode'] as const
type EmbedTab = typeof tabs[number]

const tabLabels: Record<EmbedTab, string> = {
  Output: 'Output',
  rust: 'Rust',
  c: 'C',
  python: 'Python',
  typescript: 'TS',
  abt: 'ABT',
  Bytecode: 'Bytecode',
}

const wrapperStyle = computed(() => ({
  height: props.height,
}))

const canCopy = computed(() => displayTab.value !== 'Output' && displayTab.value !== 'Bytecode' && transpiledCode.value)

async function runAction() {
  if (targetLang.value === 'run') {
    await run()
    displayTab.value = 'Output'
  } else {
    switchTab(targetLang.value)
    displayTab.value = targetLang.value
  }
}

watch(targetLang, (lang) => {
  if (lang !== 'run' && liveCompile.value) {
    switchTab(lang)
    displayTab.value = lang
  }
})

function onSwitchTab(tab: EmbedTab) {
  displayTab.value = tab
  if (tab !== 'Output' && tab !== 'Bytecode') {
    targetLang.value = tab
    switchTab(tab)
  } else if (tab === 'Output') {
    targetLang.value = 'run'
  }
}

function onLoadExample(code: string) {
  loadExample(code)
  displayTab.value = 'Output'
  targetLang.value = 'run'
}

async function copyCode() {
  if (!transpiledCode.value) return
  try {
    await navigator.clipboard.writeText(transpiledCode.value)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  } catch { /* ignore */ }
}

async function onDebugStart() {
  await debugStart()
  displayTab.value = 'Bytecode'
}

async function onDebugStop() {
  await debugStop()
  displayTab.value = 'Output'
}

function onBreakpointsChange(lines: number[]) {
  debugSetBreakpoints(lines)
}

function onBytecodeOffsetClick(_offset: number) {
  // Could cross-highlight source line from bytecode offset
}
</script>

<style scoped>
.playground-wrapper {
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid #313244;
  background: #1e1e1e;
  display: flex;
  flex-direction: column;
  margin: 1.5rem 0;
}

.playground-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: #181825;
  border-bottom: 1px solid #313244;
  flex-shrink: 0;
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

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
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

.target-select:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.run-btn, .debug-start-btn, .stop-btn {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.4rem 0.8rem;
  border: none;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.2s;
}

.run-btn {
  background: #27c93f;
  color: #1e1e2e;
}

.debug-start-btn {
  background: #89b4fa;
  color: #1e1e2e;
}

.stop-btn {
  background: #f38ba8;
  color: #1e1e2e;
}

.run-btn:hover, .debug-start-btn:hover, .stop-btn:hover {
  opacity: 0.9;
}

.run-btn:disabled, .debug-start-btn:disabled, .stop-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.debug-controls {
  display: flex;
  gap: 2px;
}

.debug-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  background: #313244;
  color: #cdd6f4;
  border: 1px solid #45475a;
  border-radius: 4px;
  padding: 0.3rem 0.5rem;
  cursor: pointer;
  transition: background 0.15s;
}

.debug-btn:hover:not(:disabled) {
  background: #45475a;
}

.debug-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.debug-btn.continue {
  color: #a6e3a1;
}

.debug-btn.step {
  color: #89b4fa;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  color: #6c7086;
  border: none;
  border-radius: 4px;
  padding: 0.35rem;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.icon-btn:hover {
  background: #313244;
  color: #cdd6f4;
}

.share-btn {
  margin-left: 0.25rem;
}

.copy-btn {
  margin-right: 0.25rem;
}

.switch-widget {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  user-select: none;
  margin-left: 0.25rem;
}

.switch-label {
  font-size: 0.75rem;
  color: #6c7086;
  font-weight: 500;
}

.switch {
  position: relative;
  display: inline-block;
  width: 32px;
  height: 18px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #45475a;
  transition: .2s;
  border-radius: 18px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 14px;
  width: 14px;
  left: 2px;
  bottom: 2px;
  background-color: white;
  transition: .2s;
  border-radius: 50%;
}

.switch input:checked + .slider {
  background-color: #27c93f;
}

.switch input:checked + .slider:before {
  transform: translateX(14px);
}

.playground-body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.editor-pane {
  border-right: 1px solid #313244;
  min-height: 0;
  overflow: hidden;
}

.editor-pane :deep(.cm-editor) {
  height: 100%;
  min-height: 100%;
}

.output-pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.output-tabs {
  display: flex;
  background: #181825;
  border-bottom: 1px solid #313244;
  align-items: center;
  flex-shrink: 0;
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

.spacer {
  flex: 1;
}

.debug-status {
  font-size: 0.7rem;
  font-weight: 600;
  padding: 0.15rem 0.5rem;
  border-radius: 10px;
  text-transform: uppercase;
  font-family: 'JetBrains Mono', monospace;
}

.debug-status.paused {
  background: #f9e2af33;
  color: #f9e2af;
}

.debug-status.running {
  background: #a6e3a133;
  color: #a6e3a1;
}

.debug-status.error {
  background: #f38ba833;
  color: #f38ba8;
}

.debug-status.finished {
  background: #89b4fa33;
  color: #89b4fa;
}

.output-content {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.output-content :deep(.bytecode-panel) {
  height: 100%;
}

/* Debug state panel */
.debug-panel {
  border-top: 1px solid #313244;
  background: #181825;
  padding: 0.5rem 0.75rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  color: #a6adc8;
  max-height: 160px;
  overflow-y: auto;
  flex-shrink: 0;
}

.debug-section {
  margin-bottom: 0.4rem;
}

.debug-section-title {
  color: #89b4fa;
  font-weight: 600;
  margin-bottom: 0.2rem;
}

.debug-stack {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.stack-item {
  background: #313244;
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 0.7rem;
}

.call-frame {
  display: flex;
  justify-content: space-between;
  padding: 1px 0;
}

.frame-name {
  color: #cba6f7;
}

.frame-info {
  color: #6c7086;
}

.debug-locals {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.local-item {
  background: #313244;
  padding: 1px 6px;
  border-radius: 3px;
}

.local-idx {
  color: #6c7086;
}

.debug-registers {
  color: #585b70;
  font-size: 0.7rem;
  margin-top: 0.3rem;
}

.toast {
  position: fixed;
  top: 16px;
  left: 50%;
  transform: translateX(-50%) translateY(-120%);
  background: #252526;
  color: #fff;
  padding: 10px 20px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  border: 1px solid #444;
  z-index: 1000;
  opacity: 0;
  transition: all 0.3s ease;
  pointer-events: none;
}

.toast.visible {
  transform: translateX(-50%) translateY(0);
  opacity: 1;
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
