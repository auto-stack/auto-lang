<template>
  <section class="tapl-runner" :data-example-id="exampleId" :aria-label="zh ? 'TAPL 示例运行器' : 'TAPL example runner'">
    <p v-if="loadingCatalog" class="status">{{ zh ? '加载示例…' : 'Loading example…' }}</p>
    <p v-else-if="error" class="status error">{{ error }}</p>
    <template v-else-if="example">
      <label class="field-label" :for="sourceId">{{ zh ? '完整源代码' : 'Complete source' }}</label>
      <textarea :id="sourceId" v-model="source" class="source" spellcheck="false" />
      <label class="field-label" :for="inputId">{{ zh ? '程序输入' : 'Program input' }}</label>
      <textarea :id="inputId" v-model="input" class="input" rows="3" spellcheck="false" />
      <div class="actions">
        <button v-if="runnable" type="button" class="run" :disabled="running" @click="run">
          {{ running ? (zh ? '运行中…' : 'Running…') : (example.execution.mode === 'test' ? (zh ? '运行测试' : 'Run tests') : (zh ? '运行' : 'Run')) }}
        </button>
        <span v-else class="status">{{ example.execution.local_only_reason || example.reason || (zh ? '仅供阅读' : 'Display only') }}</span>
      </div>
      <div v-if="showOutput" class="result" aria-live="polite">
        <pre>{{ output || (zh ? '无输出' : 'No output') }}</pre>
      </div>
      <details v-if="expectedText" class="expected">
        <summary>{{ zh ? '预期结果' : 'Expected result' }}</summary>
        <pre>{{ expectedText }}</pre>
      </details>
    </template>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, useId } from 'vue'

type ExecutionFile = { path: string; source: string }
type MaterializedExample = {
  id: string
  reason?: string
  execution: {
    mode: string
    entry: string
    source: string
    files: ExecutionFile[]
    input: string
    timeout_secs: number
    local_only_reason?: string
    expected?: { kind?: string; stdout?: string; value?: string; diagnostic?: string; contains?: string[] }
  }
}

const props = defineProps<{ exampleId: string; displayCode?: string; locale?: string }>()
const zh = computed(() => props.locale?.startsWith('zh') || (typeof document !== 'undefined' && document.documentElement.lang.startsWith('zh')))
const sourceId = `tapl-source-${useId()}`
const inputId = `tapl-input-${useId()}`
const example = ref<MaterializedExample>()
const source = ref(props.displayCode || '')
const input = ref('')
const output = ref('')
const error = ref('')
const loadingCatalog = ref(true)
const running = ref(false)
const showOutput = ref(false)
const runnable = computed(() => ['run', 'test', 'expected-error'].includes(example.value?.execution.mode || ''))
const expectedText = computed(() => {
  const expected = example.value?.execution.expected
  if (!expected || expected.kind === 'pending') return ''
  if (expected.kind === 'stdout_contains') return (expected.contains || []).join('\n')
  return expected.stdout ?? expected.value ?? expected.diagnostic ?? ''
})

onMounted(async () => {
  try {
    const response = await fetch('/tapl-examples.json')
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    const catalog = await response.json() as { entries: MaterializedExample[] }
    example.value = catalog.entries.find((entry) => entry.id === props.exampleId)
    if (!example.value) throw new Error(`Unknown example: ${props.exampleId}`)
    source.value = example.value.execution.source || props.displayCode || ''
    input.value = example.value.execution.input || ''
  } catch {
    error.value = zh.value
      ? '无法加载此示例的执行描述。请检查书籍索引是否已重新生成。'
      : 'Could not load this example’s execution description. Rebuild the book catalog and retry.'
  } finally {
    loadingCatalog.value = false
  }
})

async function run() {
  const current = example.value
  if (!current || running.value || !runnable.value) return
  running.value = true
  showOutput.value = true
  output.value = ''
  try {
    const { execution } = current
    const files = execution.files.map((file) => ({
      ...file,
      source: file.path === execution.entry ? source.value : file.source,
    }))
    const response = await fetch(`/api/${execution.mode === 'test' ? 'test' : 'run'}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        source: source.value,
        files,
        entry: execution.entry,
        input: input.value,
        timeout_secs: execution.timeout_secs,
      }),
    })
    const data = await response.json()
    if (!response.ok) {
      output.value = data.message || data.error || `HTTP ${response.status}`
    } else if (execution.mode === 'test') {
      const reports = Array.isArray(data.reports) ? data.reports : []
      output.value = `${data.passed ?? 0}/${data.executed ?? 0} ${zh.value ? '项通过' : 'tests passed'}${data.failed ? ` · ${data.failed} ${zh.value ? '项失败' : 'failed'}` : ''}${reports.map((report: { qualified_name?: string; error?: string }) => report.error ? `\n${report.qualified_name || ''}: ${report.error}` : '').filter(Boolean).join('')}`
    } else {
      output.value = `${data.stdout || ''}${data.stderr ? `${data.stdout ? '\n' : ''}${data.stderr}` : ''}`
    }
  } catch (cause) {
    output.value = `${zh.value ? '网络错误：' : 'Network error: '}${cause instanceof Error ? cause.message : String(cause)}`
  } finally {
    running.value = false
  }
}
</script>

<style scoped>
.tapl-runner { display: grid; gap: 0.45rem; margin-top: 0.65rem; }
.field-label { color: var(--vp-c-text-2); font-size: 0.72rem; }
textarea { box-sizing: border-box; width: 100%; resize: vertical; padding: 0.55rem; border: 1px solid var(--vp-c-border); border-radius: 6px; background: var(--vp-c-bg); color: var(--vp-c-text-1); font: 0.78rem/1.5 var(--vp-font-family-mono); }
.source { min-height: 8rem; }
.actions { display: flex; justify-content: flex-end; align-items: center; min-height: 2rem; }
.run { border: 0; border-radius: 6px; padding: 0.35rem 0.75rem; color: white; background: var(--vp-c-brand-1); font-weight: 600; cursor: pointer; }
.run:disabled { opacity: 0.55; cursor: wait; }
.status { margin: 0; color: var(--vp-c-text-2); font-size: 0.76rem; }
.error { color: var(--vp-c-danger-1); }
.result { overflow: auto; border: 1px solid var(--vp-c-border); border-radius: 6px; padding: 0.5rem 0.65rem; background: var(--vp-c-bg-alt); }
.result pre, .expected pre { margin: 0; white-space: pre-wrap; font-size: 0.76rem; }
.expected { border: 1px solid var(--vp-c-border); border-radius: 6px; padding: 0.4rem 0.65rem; color: var(--vp-c-text-2); font-size: 0.76rem; }
</style>
