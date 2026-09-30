<script setup lang="ts">
// PLAN-715 T-05：首页轻量演示。
// - 初始为标明来源的预置示例（纯 <pre>，不加载 CodeMirror，不发任何请求）。
// - 点击「运行」才发同源 /api/run；成功替换输出区（stdout/stderr 分列），
//   失败/超时(10s)清楚反馈并保留预置示例；AbortController 可取消、离页清理。
// - 预置输出永远标注来源，不显示为实时成功（AC-07/AC-08）。
// - Rust 视图仅在有真实 a2r 产物的示例上出现（data/home-demo.ts）。
import { computed, onUnmounted, ref, shallowRef } from 'vue'
import { useRoute } from 'vitepress'
import {
  HOME_DEMO_EXAMPLES,
  DEMO_API_BASE,
  DEMO_TIMEOUT_MS,
  type HomeDemoExample,
} from '../data/home-demo'

const route = useRoute()
const zh = computed(() => route.path.startsWith('/zh'))

const selected = ref(0)
const example = computed<HomeDemoExample>(() => HOME_DEMO_EXAMPLES[selected.value])

type RunState = 'idle' | 'loading' | 'ok' | 'fail' | 'timeout' | 'aborted'
const runState = ref<RunState>('idle')
const liveStdout = ref('')
const liveStderr = ref('')
const liveNotice = ref('')
const controller = shallowRef<AbortController | null>(null)

const tabs = computed(() => {
  const t = [
    { id: 'auto', label: 'Auto' },
    { id: 'output', label: zh.value ? '输出' : 'Output' },
  ]
  if (example.value.rust) t.push({ id: 'rust', label: 'Rust' })
  return t
})
const activeTab = ref('auto')

const outputText = computed(() => {
  if (runState.value === 'ok' || runState.value === 'fail') {
    return liveStdout.value
  }
  return example.value.output
})
const errorText = computed(() => {
  // stderr 在成功运行时也要如实展示（HTTP 200 + stderr 是合法响应）
  if (liveStderr.value && (runState.value === 'ok' || runState.value === 'fail'
    || runState.value === 'timeout' || runState.value === 'aborted')) {
    return liveStderr.value
  }
  return ''
})

const outputSourceLabel = computed(() => {
  if (runState.value === 'ok') {
    return zh.value ? '实时运行结果（同源 /api）' : 'Live result (same-origin /api)'
  }
  return zh.value ? example.value.presetEvidenceZh : example.value.presetEvidenceEn
})

const t = computed(() => zh.value ? {
  headline: '写下即运行，发布即 Rust',
  sub: '同一份 Auto 源码：在 AutoVM 里即时运行，也可以转译为地道的 Rust。以下为仓库真实示例，点击运行会请求同源 /api。',
  run: '运行', running: '运行中…', rerun: '再次运行',
  cancel: '取消', copy: '复制', copied: '已复制', copyFailed: '复制失败，请手动选择代码',
  viewSource: '查看源码', viewOutput: '查看金样', viewRust: '查看 a2r 产物',
  errNetwork: '无法连接运行服务（静态部署下属预期）。预置示例仍可查看与复制。',
  errTimeout: '运行超过 10 秒已中止。预置示例未受影响。',
  errAborted: '已取消本次运行。',
  stdout: 'stdout', stderr: 'stderr',
  tabAuto: 'Auto 源码', tabOutput: '输出', tabRust: 'Rust（a2r）',
  tryMore: '打开 Playground 试试你自己的代码',
} : {
  headline: 'Script instantly, ship as Rust',
  sub: 'The same Auto source: it runs in the AutoVM right away and transpiles to idiomatic Rust. Real examples from this repository — pressing Run calls the same-origin /api.',
  run: 'Run', running: 'Running…', rerun: 'Run again',
  cancel: 'Cancel', copy: 'Copy', copied: 'Copied', copyFailed: 'Copy failed — select the code manually',
  viewSource: 'View source', viewOutput: 'View golden', viewRust: 'View a2r artifact',
  errNetwork: 'Could not reach the run service (expected on static hosting). The preset example stays viewable and copyable.',
  errTimeout: 'Run exceeded 10 seconds and was aborted. The preset example is unaffected.',
  errAborted: 'Run cancelled.',
  stdout: 'stdout', stderr: 'stderr',
  tabAuto: 'Auto source', tabOutput: 'Output', tabRust: 'Rust (a2r)',
  tryMore: 'Open the Playground to try your own code',
})

const copied = ref('')
let copyTimer: ReturnType<typeof setTimeout> | undefined

async function copyCode(code: string) {
  clearTimeout(copyTimer)
  try {
    await navigator.clipboard.writeText(code)
    copied.value = code
  } catch {
    copied.value = 'failed'
  }
  copyTimer = setTimeout(() => { copied.value = '' }, 2500)
}

function selectExample(i: number) {
  if (selected.value === i) return
  cancelRun()
  selected.value = i
  runState.value = 'idle'
  liveStdout.value = ''
  liveStderr.value = ''
  liveNotice.value = ''
  activeTab.value = 'auto'
}

function cancelRun() {
  controller.value?.abort()
  controller.value = null
}

async function run() {
  if (runState.value === 'loading') return
  cancelRun()
  const ac = new AbortController()
  controller.value = ac
  runState.value = 'loading'
  liveStdout.value = ''
  liveStderr.value = ''
  liveNotice.value = ''
  activeTab.value = 'output'
  let timedOut = false
  const timer = setTimeout(() => { timedOut = true; ac.abort() }, DEMO_TIMEOUT_MS)
  try {
    const res = await fetch(`${DEMO_API_BASE}/api/run`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source: example.value.source }),
      signal: ac.signal,
    })
    if (!res.ok) throw new Error('HTTP ' + res.status)
    const data = await res.json()
    liveStdout.value = String(data.stdout ?? '')
    liveStderr.value = String(data.stderr ?? '')
    if (ac.signal.aborted) return
    runState.value = 'ok'
  } catch (e) {
    if (ac.signal.aborted) {
      if (timedOut) {
        runState.value = 'timeout'
        liveStderr.value = t.value.errTimeout
      } else {
        runState.value = 'aborted'
        liveStderr.value = t.value.errAborted
      }
    } else {
      runState.value = 'fail'
      liveStderr.value = t.value.errNetwork
    }
  } finally {
    clearTimeout(timer)
    if (controller.value === ac) controller.value = null
  }
}

onUnmounted(() => {
  cancelRun()
  clearTimeout(copyTimer)
})
</script>

<template>
  <div class="home-demo">
    <div class="demo-copy">
      <h2 class="demo-headline">{{ t.headline }}</h2>
      <p class="demo-sub">{{ t.sub }}</p>
      <div class="demo-picker" role="tablist" :aria-label="zh ? '示例选择' : 'Example picker'">
        <button
          v-for="(ex, i) in HOME_DEMO_EXAMPLES"
          :key="ex.id"
          type="button"
          role="tab"
          :aria-selected="selected === i"
          @click="selectExample(i)"
        >
          <strong>{{ zh ? ex.titleZh : ex.titleEn }}</strong>
          <span>{{ zh ? ex.descZh : ex.descEn }}</span>
        </button>
      </div>
      <a class="demo-playground-link" href="/playground">{{ t.tryMore }} →</a>
    </div>

    <div class="demo-window" aria-live="polite">
      <div class="demo-header">
        <div class="demo-tabs">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            type="button"
            :class="{ active: activeTab === tab.id }"
            @click="activeTab = tab.id"
          >{{ tab.id === 'auto' ? t.tabAuto : tab.id === 'output' ? t.tabOutput : t.tabRust }}</button>
        </div>
        <div class="demo-header-actions">
          <button
            v-if="activeTab !== 'output'"
            type="button"
            class="demo-copy-btn"
            @click="copyCode(activeTab === 'auto' ? example.source : example.rust!)"
          >{{ copied === (activeTab === 'auto' ? example.source : example.rust!) ? t.copied : t.copy }}</button>
          <button
            v-if="activeTab === 'auto'"
            type="button"
            class="demo-run-btn"
            :disabled="runState === 'loading'"
            @click="run()"
          >
            <span v-if="runState === 'loading'" class="demo-spin" aria-hidden="true" />
            {{ runState === 'loading' ? t.running : runState === 'idle' ? t.run : t.rerun }}
          </button>
          <button
            v-if="runState === 'loading'"
            type="button"
            class="demo-cancel-btn"
            @click="cancelRun()"
          >{{ t.cancel }}</button>
        </div>
      </div>

      <div v-show="activeTab === 'auto'" class="demo-pane">
        <span class="demo-file">{{ example.file }}</span>
        <pre class="demo-code"><code>{{ example.source }}</code></pre>
        <a class="demo-evidence-link" :href="example.sourceHref" target="_blank" rel="noopener">{{ t.viewSource }} ↗</a>
      </div>

      <div v-show="activeTab === 'output'" class="demo-pane">
        <div v-if="outputText" class="demo-output">
          <span class="demo-file">{{ t.stdout }}</span>
          <pre class="demo-code"><code>{{ outputText }}</code></pre>
        </div>
        <div v-if="errorText" class="demo-error">
          <span class="demo-file">{{ t.stderr }}</span>
          <pre class="demo-code"><code>{{ errorText }}</code></pre>
        </div>
        <p class="demo-output-source" :class="{ live: runState === 'ok' }">{{ outputSourceLabel }}</p>
        <a class="demo-evidence-link" :href="example.outputHref" target="_blank" rel="noopener">{{ t.viewOutput }} ↗</a>
      </div>

      <div v-show="activeTab === 'rust' && example.rust" class="demo-pane">
        <span class="demo-file">main.rs · a2r</span>
        <pre class="demo-code"><code>{{ example.rust }}</code></pre>
        <a class="demo-evidence-link" :href="example.rustHref" target="_blank" rel="noopener">{{ t.viewRust }} ↗</a>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-demo {
  display: grid;
  grid-template-columns: minmax(0, 5fr) minmax(0, 7fr);
  gap: 2.5rem;
  align-items: center;
  max-width: var(--site-max-width, 1200px);
  margin: 0 auto;
  padding: 4rem 2rem;
}

.demo-copy {
  min-width: 0;
}

.demo-headline {
  margin: 0 0 0.9rem;
  font-size: clamp(1.6rem, 3vw, 2.2rem);
  font-weight: 800;
  letter-spacing: -0.02em;
  color: hsl(var(--foreground));
}

.demo-sub {
  margin: 0 0 1.5rem;
  font-size: 1rem;
  line-height: 1.75;
  color: hsl(var(--muted-foreground));
  max-width: var(--site-text-width, 720px);
}

.demo-picker {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-bottom: 1.25rem;
}

.demo-picker button {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  padding: 0.7rem 0.9rem;
  border: 1px solid hsl(var(--border));
  border-radius: var(--radius);
  background: hsl(var(--card));
  color: hsl(var(--foreground));
  text-align: left;
  cursor: pointer;
  min-width: 0;
}

.demo-picker button strong {
  font-size: 0.95rem;
}

.demo-picker button span {
  font-size: 0.8rem;
  line-height: 1.6;
  color: hsl(var(--muted-foreground));
}

.demo-picker button[aria-selected="true"] {
  border-color: var(--vp-c-brand-1);
  box-shadow: 0 0 0 1px var(--vp-c-brand-1) inset;
}

.demo-picker button:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 2px;
}

.demo-playground-link {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--vp-c-brand-1);
  text-decoration: none;
}

.demo-playground-link:hover {
  text-decoration: underline;
}

.demo-playground-link:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 2px;
}

.demo-window {
  min-width: 0;
  border: 1px solid hsl(var(--border));
  border-radius: 12px;
  background: #1e1e2e;
  overflow: hidden;
  box-shadow: 0 16px 44px rgba(0, 0, 0, 0.22);
}

.demo-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  flex-wrap: wrap;
  padding: 0.45rem 0.6rem;
  background: #181825;
  border-bottom: 1px solid #313244;
}

.demo-tabs {
  display: flex;
  gap: 2px;
}

.demo-tabs button {
  padding: 0.32rem 0.7rem;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: #a6adc8;
  font-size: 0.78rem;
  font-weight: 500;
  cursor: pointer;
}

.demo-tabs button:hover {
  color: #cdd6f4;
}

.demo-tabs button.active {
  color: #cdd6f4;
  background: #313244;
}

.demo-tabs button:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 1px;
}

.demo-header-actions {
  display: flex;
  gap: 0.4rem;
}

.demo-run-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.32rem 0.85rem;
  border: none;
  border-radius: 6px;
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  color: #fff;
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
}

.demo-run-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.demo-run-btn:focus-visible,
.demo-cancel-btn:focus-visible,
.demo-copy-btn:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 1px;
}

.demo-cancel-btn {
  padding: 0.32rem 0.7rem;
  border: 1px solid #41445b;
  border-radius: 6px;
  background: transparent;
  color: #a6adc8;
  font-size: 0.78rem;
  cursor: pointer;
}

.demo-copy-btn {
  padding: 0.32rem 0.7rem;
  border: 1px solid #41445b;
  border-radius: 6px;
  background: transparent;
  color: #cdd6f4;
  font-size: 0.78rem;
  cursor: pointer;
}

.demo-copy-btn:hover {
  background: #313244;
}

.demo-spin {
  width: 10px;
  height: 10px;
  border: 2px solid rgba(255, 255, 255, 0.4);
  border-top-color: #fff;
  border-radius: 50%;
  animation: demo-spin 0.8s linear infinite;
}

@keyframes demo-spin {
  to { transform: rotate(360deg); }
}

.demo-pane {
  position: relative;
  padding: 0.9rem 1rem 1.6rem;
}

.demo-file {
  display: block;
  margin-bottom: 0.5rem;
  font-family: var(--vp-font-family-mono);
  font-size: 0.72rem;
  color: #6c7086;
}

.demo-code {
  margin: 0;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 0.84rem;
  line-height: 1.65;
  color: #cdd6f4;
  overflow-x: auto;
}

.demo-output .demo-code {
  color: #a6e3a1;
}

.demo-error .demo-code {
  color: #f38ba8;
}

.demo-evidence-link {
  position: absolute;
  right: 1rem;
  bottom: 0.45rem;
  font-size: 0.72rem;
  color: #89b4fa;
  text-decoration: none;
}

.demo-evidence-link:hover {
  text-decoration: underline;
}

.demo-evidence-link:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 2px;
}

.demo-output-source {
  margin: 0.75rem 0 0.25rem;
  font-size: 0.75rem;
  line-height: 1.6;
  color: #a6adc8;
}

.demo-output-source.live {
  color: #a6e3a1;
}

@media (max-width: 900px) {
  .home-demo {
    grid-template-columns: 1fr;
    gap: 1.5rem;
    padding: 3rem 1.25rem;
  }
}

@media (prefers-reduced-motion: reduce) {
  .demo-spin {
    animation: none;
  }
}
</style>
