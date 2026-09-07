<template>
  <div class="playground-card" :style="cardStyle">
    <div class="card-toolbar">
      <div class="toolbar-left">
        <Code2 :size="16" />
        <span class="toolbar-title">Auto Playground</span>
        <!-- 旧 AutoPlayground 常驻行为改为显式选入（Playground 设计 §4.4，默认 false）。 -->
        <ExampleSelector
          v-if="exampleSelector"
          :api-base="apiBase || '/api'"
          @select="onLoadExample"
        />
        <span
          v-if="ideMode !== null"
          class="ide-entry"
          :title="ideMode ? '切换到全功能 IDE（文件树/调试/回放）' : 'IDE 模式仅在后端自服务页可用——本地运行 cargo run -p auto-playground 后访问'"
        >
          <button class="ide-btn" :disabled="!ideMode" @click="emit('ide-mode')">
            <AppWindow :size="14" />
            在 IDE 中打开
          </button>
        </span>
      </div>
      <div class="toolbar-right">
        <select
          v-if="toolbarOn.transpile"
          v-model="targetLang"
          class="target-select"
          :disabled="!!isDebugging"
        >
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
        <template v-else-if="toolbarOn.debug">
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
        <button v-if="toolbarOn.debug && isDebugging" class="stop-btn" @click="onDebugStop" title="Stop Debug">
          <Square :size="14" />
          Stop
        </button>
        <button
          v-else-if="toolbarOn.debug"
          class="debug-start-btn"
          @click="onDebugStart"
          :disabled="isLoading"
          title="Start Debug"
        >
          <Bug :size="14" />
          Debug
        </button>
        <label v-if="toolbarOn.live && !isDebugging" class="switch-widget" title="Toggle live transpile on edit">
          <span class="switch-label">Live</span>
          <span class="switch">
            <input
              type="checkbox"
              :checked="!!liveCompile"
              @change="onToggleLive($event)"
            />
            <span class="slider"></span>
          </span>
        </label>
        <button v-if="toolbarOn.share" class="icon-btn share-btn" @click="shareAction" title="Copy shareable link">
          <Share2 :size="14" />
        </button>
      </div>
    </div>
    <div class="card-body">
      <div v-if="fileTabs.length > 1" class="file-tabs">
        <button
          v-for="f in fileTabs"
          :key="f.path"
          class="file-tab"
          :class="{ active: f.path === activeFile, entry: f.path === ENTRY_FILE }"
          :title="f.path === ENTRY_FILE ? '入口文件（entry 锁定）' : f.path"
          @click="onSelectFile(f.path)"
        >
          <Lock v-if="f.path === ENTRY_FILE" :size="10" />
          <span>{{ f.path }}</span>
        </button>
      </div>
      <SnippetRunner
        ref="runner"
        :code="code"
        :api-base="apiBase"
        :autorun="autorun"
        :height="height"
        :action-bar="false"
        fill
        orientation="row"
        :is-debugging="!!isDebugging"
        :breakpoints="breakpoints ?? []"
        :current-debug-line="debugState?.line ?? null"
        :run-handler="runAction"
        @breakpoints-change="onBreakpointsChange"
      >
        <template #output>
          <div class="card-output">
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
              <ExpectedOutputPanel
                v-if="displayTab === 'Expected'"
                :expected="expectedOutput ?? ''"
                :actual="stdout ?? ''"
                :has-run="hasRun"
              />
              <ConsoleOutput
                v-else-if="displayTab === 'Output'"
                :stdout="stdout ?? ''"
                :stderr="stderr ?? ''"
                :result="resultCode ?? ''"
                :time-ms="timeMs ?? 0"
              />
              <BytecodePanel
                v-else-if="displayTab === 'Bytecode'"
                :bytecode="bytecode ?? []"
                :current-ip="debugState?.ip"
                @offset-click="onBytecodeOffsetClick"
              />
              <div v-else class="output-code-split">
                <FileTree
                  v-if="showTransFileTree"
                  :files="transFiles ?? []"
                  :selected="selectedTransFile ?? ''"
                  @select="onSelectTransFile"
                />
                <CodePreview
                  :code="transpiledCode ?? ''"
                  :language="displayTab"
                  :highlight-lines="highlightedOutputLines ?? []"
                  @line-click="onOutputLineClick"
                />
              </div>
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
                  <span class="frame-name">{{ frame.fn_name || '<root>' }}</span>
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
        </template>
      </SnippetRunner>
    </div>
  </div>
  <div class="toast" :class="{ visible: shareToast?.visible }">
    {{ shareToast?.message }}
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue'
import { Play, Loader2, Code2, Share2, Copy, Check, Bug, Square, ArrowDown, ArrowUp, SkipForward, Lock, AppWindow } from 'lucide-vue-next'
import SnippetRunner from './SnippetRunner.vue'
import BytecodePanel from './BytecodePanel.vue'
import CodePreview from './CodePreview.vue'
import ConsoleOutput from './ConsoleOutput.vue'
import ExampleSelector from './ExampleSelector.vue'
import ExpectedOutputPanel from './ExpectedOutputPanel.vue'
import FileTree from './FileTree.vue'
import type { OutputTab, PlaygroundCardToolbar } from '../types'

const props = withDefaults(defineProps<{
  /** 契约镜像 types.ts PlaygroundCardProps（defineProps 用内联类型——compiler-sfc 无法解析导入类型）。 */
  code: string
  apiBase?: string
  autorun?: boolean
  target?: 'run' | 'rust' | 'c' | 'python' | 'typescript' | 'abt'
  height?: string
  /** manifest 笔记 id（582 Notes Explorer 用；本期仅预留存值）。 */
  noteId?: string
  /** 期望输出（vm-golden 笔记 .expected.out）；非空时输出区加"期望输出"对照 tab。 */
  expectedOutput?: string | null
  /** 项目型笔记文件集（kind=project）；>1 文件时呈文件 tab（entry 锁 main.at）。 */
  files?: { path: string; content: string }[] | null
  /** 项目目录（相对服务端 examples/playground-demo）；运行走 files 形态。 */
  projectDir?: string | null
  /** IDE 模式入口：true=可用（点击发 ide-mode）；false=禁用+提示；null=不渲染（默认）。 */
  ideMode?: boolean | null
  /** 工具栏项开关（默认全开）。 */
  toolbar?: { transpile?: boolean; share?: boolean; debug?: boolean; live?: boolean }
  /** 是否渲染 ExampleSelector（默认 false；旧 AutoPlayground 常驻行为需显式选入）。 */
  exampleSelector?: boolean
}>(), {
  apiBase: '',
  autorun: false,
  target: 'run',
  height: '480px',
  toolbar: () => ({}),
  exampleSelector: false,
  expectedOutput: null,
  files: null,
  projectDir: null,
  ideMode: null,
})

const emit = defineEmits<{
  'ide-mode': []
}>()

const DEFAULT_CODE = `fn main() {
    let message = "Hello from Auto!"
    print(message)
}`

// code 为必填 prop；宿主未传时给旧默认（兼容别名页旧用法）。
// noteId（PlaygroundCardProps）：manifest 笔记 id，582 Notes Explorer 传入——本期仅存值，不参与渲染/请求。
const code = computed(() => props.code ?? DEFAULT_CODE)

const toolbarOn = computed<Required<PlaygroundCardToolbar>>(() => ({
  transpile: props.toolbar?.transpile ?? true,
  share: props.toolbar?.share ?? true,
  debug: props.toolbar?.debug ?? true,
  live: props.toolbar?.live ?? true,
}))

const runner = ref<InstanceType<typeof SnippetRunner> | null>(null)

// 运行核状态（经 SnippetRunner defineExpose 透出的 usePlayground 面）。
const isLoading = computed(() => runner.value?.isLoading ?? false)
const isDebugging = computed(() => runner.value?.isDebugging ?? false)
const debugState = computed(() => runner.value?.debugState ?? null)
const stdout = computed(() => runner.value?.stdout ?? '')
const stderr = computed(() => runner.value?.stderr ?? '')
const resultCode = computed(() => runner.value?.resultCode ?? '')
const timeMs = computed(() => runner.value?.timeMs ?? 0)
const bytecode = computed(() => runner.value?.bytecode ?? [])
const transpiledCode = computed(() => runner.value?.transpiledCode ?? '')
const transFiles = computed(() => runner.value?.transFiles ?? [])
const selectedTransFile = computed(() => runner.value?.selectedTransFile ?? '')
const highlightedOutputLines = computed(() => runner.value?.highlightedOutputLines ?? [])
const liveCompile = computed(() => runner.value?.liveCompile ?? false)
const breakpoints = computed(() => runner.value?.breakpoints ?? [])
const shareToast = computed(() => runner.value?.shareToast)

const displayTab = ref<EmbedTab>('Output')
const targetLang = ref<'run' | Exclude<OutputTab, 'bytecode'>>(props.target)
const copied = ref(false)
// 本卡片实例是否已执行过运行（期望输出对照三态依据；卡片按 noteId 重挂载自动复位）。
const hasRun = ref(false)

type EmbedTab = 'Expected' | 'Output' | 'rust' | 'c' | 'python' | 'typescript' | 'abt' | 'Bytecode'

const tabs = computed<EmbedTab[]>(() => {
  const base: EmbedTab[] = ['Output', 'rust', 'c', 'python', 'typescript', 'abt', 'Bytecode']
  return props.expectedOutput ? ['Expected', ...base] : base
})

const tabLabels: Record<EmbedTab, string> = {
  Expected: '期望输出',
  Output: 'Output',
  rust: 'Rust',
  c: 'C',
  python: 'Python',
  typescript: 'TS',
  abt: 'ABT',
  Bytecode: 'Bytecode',
}

const cardStyle = computed(() =>
  props.height === 'auto' ? { minHeight: '480px' } : { height: props.height },
)

const canCopy = computed(() => displayTab.value !== 'Output' && displayTab.value !== 'Bytecode' && !!transpiledCode.value)

const showTransFileTree = computed(() => {
  const tab = displayTab.value
  return tab !== 'Output' && tab !== 'Bytecode' && transFiles.value.length > 1
})

async function runAction() {
  const r = runner.value
  if (!r) return
  syncProjectRun(r)
  if (targetLang.value === 'run') {
    await r.run()
    hasRun.value = true
    // 有期望输出的笔记：运行后直接呈对照（Playground 设计 §6.3）。
    displayTab.value = props.expectedOutput ? 'Expected' : 'Output'
  } else {
    r.switchTab(targetLang.value)
    displayTab.value = targetLang.value
  }
}

// ── 项目型笔记文件 tab（Plan 582 T7）：多文件切换编辑，运行走 files 形态 ──

const ENTRY_FILE = 'main.at'

const activeFile = ref(ENTRY_FILE)
const fileBuffers = ref(new Map<string, string>())

const fileTabs = computed(() => (props.files ?? []).map((f) => ({ path: f.path })))

function syncActiveBuffer() {
  const r = runner.value
  if (!r) return
  fileBuffers.value.set(activeFile.value, r.source)
}

function onSelectFile(path: string) {
  if (path === activeFile.value) return
  syncActiveBuffer()
  activeFile.value = path
  const r = runner.value
  if (r) r.source = fileBuffers.value.get(path) ?? ''
}

function syncProjectRun(r: NonNullable<typeof runner.value>) {
  if (!props.files || props.files.length === 0) return
  syncActiveBuffer()
  r.projectFiles = [...fileBuffers.value.entries()].map(([path, source]) => ({ path, source }))
  r.projectDir = props.projectDir ?? undefined
}

onMounted(() => {
  if (!props.files || props.files.length === 0) return
  const map = new Map<string, string>()
  for (const f of props.files) map.set(f.path, f.content)
  fileBuffers.value = map
  activeFile.value = map.has(ENTRY_FILE) ? ENTRY_FILE : props.files[0].path
})

watch(targetLang, (lang) => {
  if (lang !== 'run' && liveCompile.value) {
    runner.value?.switchTab(lang)
    displayTab.value = lang
  }
})

function onSwitchTab(tab: EmbedTab) {
  displayTab.value = tab
  if (tab !== 'Output' && tab !== 'Bytecode' && tab !== 'Expected') {
    targetLang.value = tab
    runner.value?.switchTab(tab)
  } else if (tab === 'Output') {
    targetLang.value = 'run'
  }
}

function onSelectTransFile(path: string) {
  runner.value?.selectTransFile(displayTab.value, path)
}

function onLoadExample(payload: { source: string; project_dir?: string }) {
  runner.value?.loadExample(payload)
  displayTab.value = 'Output'
  targetLang.value = 'run'
}

function onOutputLineClick(line: number) {
  runner.value?.highlightOutputLine(selectedTransFile.value, line)
}

function onToggleLive(e: Event) {
  const r = runner.value
  if (r) r.liveCompile = (e.target as HTMLInputElement).checked
}

async function shareAction() {
  await runner.value?.share()
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
  await runner.value?.debugStart()
  displayTab.value = 'Bytecode'
}

async function onDebugStop() {
  await runner.value?.debugStop()
  displayTab.value = 'Output'
}

function debugCommand(cmd: 'continue' | 'step' | 'step_over' | 'step_out') {
  void runner.value?.debugCommand(cmd)
}

function onBreakpointsChange(lines: number[]) {
  void runner.value?.debugSetBreakpoints(lines)
}

function onBytecodeOffsetClick(_offset: number) {
  // Could cross-highlight source line from bytecode offset
}

// 暴露给宿主（NotesExplorer 键盘 Ctrl+Enter 转发）。
defineExpose({
  run: runAction,
})
</script>

<style scoped>
.playground-card {
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid #313244;
  background: #1e1e1e;
  display: flex;
  flex-direction: column;
  margin: 1.5rem 0;
}

.card-toolbar {
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

.card-body {
  flex: 1 1 auto;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.file-tabs {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 0.25rem 0.5rem 0;
  background: #181825;
  border-bottom: 1px solid #313244;
  flex-shrink: 0;
}

.file-tab {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.25rem 0.6rem;
  border: 1px solid #313244;
  border-bottom: none;
  border-radius: 6px 6px 0 0;
  background: #11111b;
  color: #6c7086;
  font-size: 0.72rem;
  font-family: 'JetBrains Mono', monospace;
  cursor: pointer;
  transition: color 0.15s, background 0.15s;
}

.file-tab:hover {
  color: #cdd6f4;
}

.file-tab.active {
  background: #1e1e1e;
  color: #cdd6f4;
}

.file-tab.entry {
  color: #a6e3a1;
}

.file-tab.entry.active {
  color: #a6e3a1;
}

.ide-entry {
  display: inline-flex;
}

.ide-btn {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.3rem 0.6rem;
  border: 1px solid #45475a;
  border-radius: 6px;
  background: transparent;
  color: #a6adc8;
  font-size: 0.75rem;
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
}

.ide-btn:hover:not(:disabled) {
  color: #cdd6f4;
  border-color: #6366f1;
}

.ide-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
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

/* Card 输出区（Runner #output 插槽内容，Card 作用域样式生效） */
.card-output {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
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

.output-code-split {
  display: flex;
  flex-direction: row;
  height: 100%;
  min-height: 0;
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
</style>
