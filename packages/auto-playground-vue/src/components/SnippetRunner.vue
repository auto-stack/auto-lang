<template>
  <div
    class="snippet-runner"
    :class="{ fill, row: fill && orientation === 'row', sized: !fill && height !== 'auto' }"
    :style="containerStyle"
  >
    <div v-if="actionBar" class="snippet-actionbar">
      <button class="run-action" :class="{ busy: isLoading }" :title="actionTitle" @click="triggerAction">
        <Play v-if="!isLoading" :size="13" />
        <Loader2 v-else :size="13" class="spin" />
      </button>
      <span v-if="target !== 'run'" class="target-hint">→ {{ targetLabel }}</span>
      <span class="spacer"></span>
      <button
        class="output-toggle"
        :class="{ open: outputOpen }"
        title="Toggle output"
        @click="outputOpen = !outputOpen"
      >
        <ChevronDown :size="13" />
      </button>
    </div>
    <div class="snippet-editor" :style="editorStyle">
      <CodeEditor
        :model-value="source"
        @update:model-value="source = $event"
        :on-run="triggerAction"
        :is-debugging="isDebugging"
        :breakpoints="breakpoints"
        :current-debug-line="currentDebugLine ?? null"
        @breakpoints-change="(lines: number[]) => emit('breakpoints-change', lines)"
      />
    </div>
    <div v-if="outputOpen || fill" class="snippet-output">
      <slot name="output">
        <!-- 无后端报错提示（临时态，582 换统一降级卡） -->
        <div v-if="noBackend" class="no-backend-hint">
          后端不可用——本地启动：<code>cargo run -p auto-playground</code>
        </div>
        <ConsoleOutput
          v-if="target === 'run'"
          :stdout="stdout"
          :stderr="stderr"
          :result="resultCode"
          :time-ms="timeMs"
        />
        <CodePreview v-else :code="transpiledCode" :language="target" />
      </slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Play, Loader2, ChevronDown } from 'lucide-vue-next'
import CodeEditor from './CodeEditor.vue'
import CodePreview from './CodePreview.vue'
import ConsoleOutput from './ConsoleOutput.vue'
import { usePlayground } from '../composables/usePlayground'
import type { PlaygroundTarget } from '../types'

const props = withDefaults(defineProps<{
  /** 初始代码（必填）。契约镜像 types.ts SnippetRunnerProps（defineProps 用内联类型——compiler-sfc 无法解析导入类型）。 */
  code: string
  /** 后端地址；'' = 同源 /api。 */
  apiBase?: string
  /** 挂载即运行。 */
  autorun?: boolean
  /** 首选动作（默认 'run'）。 */
  target?: 'run' | 'rust' | 'c' | 'python' | 'typescript' | 'abt'
  /** 容器高；'auto' = 按行数。 */
  height?: string
  /** 嵌入态（PlaygroundCard 用）：隐藏自带动作条、主体撑满容器。 */
  actionBar?: boolean
  /** 嵌入态：编辑器/输出区按 flex 填充，忽略 height 推导。 */
  fill?: boolean
  /** 嵌入态布局：stack=上下（默认）| row=左右双栏（旧 Playground 形态）。 */
  orientation?: 'stack' | 'row'
  /** 嵌入态透传：编辑器调试面（断点/当前行）。 */
  isDebugging?: boolean
  breakpoints?: number[]
  currentDebugLine?: number | null
  /** 嵌入态覆盖：宿主接管动作位/编辑器 Ctrl+Enter 的执行。 */
  runHandler?: () => void
}>(), {
  apiBase: '',
  autorun: false,
  target: 'run',
  height: 'auto',
  actionBar: true,
  fill: false,
  orientation: 'stack',
  isDebugging: false,
  breakpoints: () => [],
  currentDebugLine: null,
})

const emit = defineEmits<{
  'breakpoints-change': [lines: number[]]
}>()

const playground = usePlayground({
  apiBase: props.apiBase || '/api',
  defaultSource: props.code,
  persistKey: false,
  preloadTargets: false,
})

const {
  source, stdout, stderr, resultCode, timeMs, isLoading,
  transpiledCode, liveCompile, transFiles, selectedTransFile,
  highlightedOutputLines, shareToast,
  debugState, bytecode, breakpoints, isDebugging,
  run, switchTab, selectTransFile, loadExample, share,
  highlightOutputLine, transpile,
  debugStart, debugSetBreakpoints, debugCommand, debugStop,
} = playground

const outputOpen = ref(false)

const TARGET_LABELS: Record<Exclude<PlaygroundTarget, 'run'>, string> = {
  rust: 'Rust',
  c: 'C',
  python: 'Python',
  typescript: 'TypeScript',
  abt: 'ABT',
}

const targetLabel = computed(() =>
  props.target === 'run' ? '' : TARGET_LABELS[props.target],
)

const actionTitle = computed(() =>
  props.target === 'run' ? 'Run (Ctrl+Enter)' : `Transpile to ${targetLabel.value}`,
)

const noBackend = computed(() => stderr.value.startsWith('Network error'))

// height='auto'：按代码行数推导编辑器高（钳制在 80px–480px）；fill/显式值走 flex/容器高。
const editorStyle = computed(() => {
  if (props.fill || props.height !== 'auto') return {}
  const lines = source.value.split('\n').length
  const byLines = Math.min(480, Math.max(80, lines * 20 + 16))
  return { height: `${byLines}px` }
})

const containerStyle = computed(() =>
  props.fill || props.height === 'auto' ? {} : { height: props.height },
)

async function onAction() {
  if (props.target === 'run') {
    await run()
  } else {
    await transpile(props.target)
  }
  if (stdout.value || stderr.value || resultCode.value || transpiledCode.value) {
    outputOpen.value = true
  }
}

function triggerAction() {
  if (props.runHandler) {
    void props.runHandler()
    return
  }
  void onAction()
}

if (props.autorun) {
  onMounted(() => { triggerAction() })
}

// 暴露给 PlaygroundCard（壳层工具栏/输出 tabs 驱动同一运行核）。
defineExpose({
  source, stdout, stderr, resultCode, timeMs, isLoading,
  transpiledCode, liveCompile, transFiles, selectedTransFile,
  highlightedOutputLines, shareToast,
  debugState, bytecode, breakpoints, isDebugging,
  run, switchTab, selectTransFile, loadExample, share,
  highlightOutputLine, transpile,
  debugStart, debugSetBreakpoints, debugCommand, debugStop,
  action: triggerAction,
})
</script>

<style scoped>
.snippet-runner {
  border-radius: 10px;
  border: 1px solid #313244;
  background: #1e1e1e;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  margin: 1rem 0;
}

/* 嵌入态（PlaygroundCard）：撑满宿主容器，编辑器/输出区按 flex 分配。 */
.snippet-runner.fill {
  flex: 1 1 auto;
  min-height: 0;
  height: 100%;
  margin: 0;
  border: none;
  border-radius: 0;
}

.snippet-runner.fill .snippet-editor,
.snippet-runner.sized .snippet-editor {
  flex: 1 1 0;
}

.snippet-runner.fill .snippet-output {
  flex: 0 0 45%;
  max-height: none;
}

/* 双栏嵌入态（旧 Playground 形态）：编辑器 | 输出区 1:1。 */
.snippet-runner.fill.row {
  flex-direction: row;
}

.snippet-runner.fill.row .snippet-editor {
  flex: 1 1 50%;
  min-width: 0;
  border-right: 1px solid #313244;
}

.snippet-runner.fill.row .snippet-output {
  flex: 1 1 50%;
  border-top: none;
  border-left: 1px solid #313244;
}

@media (max-width: 768px) {
  .snippet-runner.fill.row {
    flex-direction: column;
  }

  .snippet-runner.fill.row .snippet-editor {
    border-right: none;
    border-bottom: 1px solid #313244;
  }

  .snippet-runner.fill.row .snippet-output {
    border-left: none;
    border-top: 1px solid #313244;
  }
}

.snippet-actionbar {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.25rem 0.5rem;
  background: #181825;
  border-bottom: 1px solid #313244;
  flex-shrink: 0;
}

.run-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 22px;
  border: none;
  border-radius: 5px;
  background: #27c93f;
  color: #1e1e2e;
  cursor: pointer;
  transition: opacity 0.2s;
}

.run-action:hover {
  opacity: 0.9;
}

.run-action.busy {
  opacity: 0.6;
  cursor: wait;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.target-hint {
  font-size: 0.7rem;
  font-family: 'JetBrains Mono', monospace;
  color: #6c7086;
}

.spacer {
  flex: 1;
}

.output-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: #6c7086;
  cursor: pointer;
  transition: background 0.15s, color 0.15s, transform 0.15s;
}

.output-toggle:hover {
  background: #313244;
  color: #cdd6f4;
}

.output-toggle.open {
  transform: rotate(180deg);
}

.snippet-editor {
  min-height: 0;
  overflow: hidden;
}

.snippet-editor :deep(.cm-editor) {
  height: 100%;
  min-height: 100%;
}

.snippet-output {
  border-top: 1px solid #313244;
  max-height: 260px;
  overflow: auto;
  flex-shrink: 0;
}

.no-backend-hint {
  padding: 6px 12px;
  font-size: 0.75rem;
  color: #f9e2af;
  background: #f9e2af14;
  border-bottom: 1px solid #313244;
}

.no-backend-hint code {
  font-family: 'JetBrains Mono', monospace;
  color: #cdd6f4;
}
</style>
