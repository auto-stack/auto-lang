<template>
  <div class="auto-fence">
    <div class="af-toolbar">
      <button
        v-if="!locked"
        ref="toggleBtn"
        type="button"
        class="af-run"
        :class="{ open }"
        :aria-expanded="open"
        :aria-controls="panelId"
        :aria-label="open ? t.collapseAria : t.runAria"
        @click="toggle"
      >
        <Play v-if="!open" :size="13" />
        <Square v-else :size="13" />
        {{ open ? t.collapse : t.run }}
      </button>
      <span v-else class="af-lock">
        <Lock :size="13" />
        {{ t.locked }}
      </span>
    </div>
    <!-- PLAN-718 T-04：模块依赖提示从悬浮工具栏移入独立说明区（可换行、不遮代码），
         Playground 链接按当前 locale。 -->
    <p v-if="locked" class="af-lock-note">
      {{ t.lockNote }}
      <a :href="playgroundHref">Playground</a>
    </p>
    <div v-if="open" :id="panelId" class="af-runner">
      <!-- PLAN-718 T-05：runner（CodeMirror 链）按需异步加载——SSR 静态页只保留
           原始高亮代码；点击运行才拉取 chunk；失败给本地化说明与重试/收起，不冒充
           程序运行错误。 -->
      <component
        :is="runnerComp"
        v-if="runnerState === 'ready' && runnerComp"
        :code="source"
        autorun
        height="auto"
      />
      <ReaderLoadingState v-else-if="runnerState === 'loading'" :label="t.loading" />
      <div v-else-if="runnerState === 'error'" class="af-load-error">
        <p class="af-error-text">{{ t.runnerFailed }}</p>
        <div class="af-error-actions">
          <button type="button" class="af-run" @click="loadRunner">{{ t.retry }}</button>
          <button type="button" class="af-run" @click="collapse">{{ t.collapse }}</button>
        </div>
      </div>
    </div>
    <!-- PLAN-718 T-05：加载/失败等待态保持原文可读（AC-10），runner 就绪后才让位。 -->
    <div v-show="!open || runnerState !== 'ready'" class="af-original">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, useId, shallowRef, type Component } from 'vue'
import { useData } from 'vitepress'
import { Play, Square, Lock } from 'lucide-vue-next'
import ReaderLoadingState from './ReaderLoadingState.vue'

const props = defineProps<{
  /** 围栏原文（encodeURIComponent 编码传入；主题层 fence 渲染器注入）。 */
  code: string
}>()

const open = ref(false)

// PLAN-718 T-04：SSR 稳定的唯一 ID（aria-controls 关联运行面板）。
const panelId = `af-panel-${useId()}`

const { lang } = useData()
const zh = computed(() => lang.value.startsWith('zh'))

const t = computed(() => zh.value
  ? {
      run: '运行',
      collapse: '收起',
      runAria: '运行示例代码',
      collapseAria: '收起示例运行器',
      locked: '依赖模块示例',
      lockNote: '此示例依赖多文件模块，不能在书页内运行——请到',
      loading: '运行器加载中…',
      runnerFailed: '运行器未加载（网络或资源错误）。原始代码保持不变，可重试或收起。',
      retry: '重试',
    }
  : {
      run: 'Run',
      collapse: 'Collapse',
      runAria: 'Run example code',
      collapseAria: 'Collapse the example runner',
      locked: 'Needs modules',
      lockNote: 'This example depends on multi-file modules and cannot run inline — open it in the',
      loading: 'Loading the runner…',
      runnerFailed: 'The runner failed to load (network or asset error). The original code is unchanged — retry or collapse.',
      retry: 'Retry',
    })

const playgroundHref = computed(() => (zh.value ? '/zh/playground' : '/playground'))

const source = computed(() => {
  try {
    return decodeURIComponent(props.code)
  } catch {
    return props.code
  }
})

// 启发式 import 检测（与 build-playground-notes.mjs isStandalone 同规则）：
// 含 use/import 顶层声明的围栏是多文件依赖片段，书页内不可独立运行。
const locked = computed(() => /^[ \t]*(use|import)[ \t]/m.test(source.value))

// ---------------------------------------------------------------- PLAN-718 T-05
// runner 异步加载。in-flight promise 去重：快速开合/双击不会并发重复加载，
// 也不会挂载多个 runner 实例（v-if 单实例 + 组件引用复用）。
// 重试：HTML module map 会缓存失败的模块脚本（同 URL 重试秒拒；query 变体在
// build 解析时被去重抹平，?url 对 .ts 是资产拷贝不是 chunk——均已实测不可用）。
// 因此用三个真实 shim 入口（不同模块 id → 不同入口 chunk → 不同 module-map 键，
// 重型依赖 chunk 跨 shim 共享）：重试=换下一个入口；两次后错误态保持。
type RunnerState = 'idle' | 'loading' | 'ready' | 'error'
const runnerState = ref<RunnerState>('idle')
const runnerComp = shallowRef<Component | null>(null)
let loadPromise: Promise<void> | null = null
let attempt = 0
const toggleBtn = ref<HTMLButtonElement>()

function loadAttempt(n: number): Promise<typeof import('auto-playground-vue')> {
  if (n === 0) return import('./runner-entry.ts')
  if (n === 1) return import('./runner-entry-retry1.ts')
  return import('./runner-entry-retry2.ts')
}

function loadRunner(): Promise<void> {
  if (runnerState.value === 'ready') return Promise.resolve()
  if (loadPromise) return loadPromise
  runnerState.value = 'loading'
  loadPromise = loadAttempt(attempt)
    .then((m) => {
      runnerComp.value = m.SnippetRunner
      runnerState.value = 'ready'
    })
    .catch(() => {
      if (attempt < 2) attempt++
      runnerState.value = 'error'
      loadPromise = null
    })
  return loadPromise
}

function toggle() {
  open.value = !open.value
  if (open.value) {
    void loadRunner()
  } else {
    collapse()
  }
}

/** 收起：若焦点在将卸载的运行面板内，先归还到开关按钮（焦点不丢进不可见元素）。 */
function collapse() {
  const active = document.activeElement
  if (open.value && active instanceof HTMLElement && panelId) {
    const panel = document.getElementById(panelId)
    if (panel && panel.contains(active)) toggleBtn.value?.focus()
  }
  open.value = false
}
</script>

<style scoped>
.auto-fence {
  margin: 0.75rem 0;
}

/* PLAN-718 T-04：工具栏为常规流内行（不再 absolute 悬浮在代码首行上，
   也不与 VitePress 原生 copy 按钮叠放）。 */
.af-toolbar {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  min-height: 30px;
  margin-bottom: 0.25rem;
}

.af-run {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.25rem 0.65rem;
  border: 1px solid var(--vp-c-border, #313244);
  border-radius: 6px;
  background: var(--vp-c-bg-alt, #181825);
  color: var(--vp-c-text-2, #a6adc8);
  font-size: 0.72rem;
  font-weight: 600;
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
}

.af-run:hover {
  color: var(--vp-c-brand-1, #6366f1);
  border-color: var(--vp-c-brand-1, #6366f1);
}

.af-run:focus-visible {
  outline: 3px solid var(--vp-c-brand-1, #6366f1);
  outline-offset: 2px;
}

.af-run.open {
  color: var(--vp-c-text-2, #a6adc8);
}

.af-lock {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.25rem 0.6rem;
  border: 1px dashed var(--vp-c-border, #313244);
  border-radius: 6px;
  background: var(--vp-c-bg-alt, #181825);
  color: var(--vp-c-text-3, #6c7086);
  font-size: 0.72rem;
}

.af-lock-note {
  display: block;
  margin: 0 0 0.5rem;
  padding: 0.5rem 0.75rem;
  border: 1px dashed var(--vp-c-border, #313244);
  border-radius: var(--site-radius, 0.5rem);
  background: var(--vp-c-bg-soft, #11111b);
  color: var(--vp-c-text-2, #a6adc8);
  font-size: 0.8rem;
  line-height: 1.6;
  overflow-wrap: anywhere;
}

.af-lock-note a {
  color: var(--vp-c-brand-1, #89b4fa);
  font-weight: 600;
}

.af-lock-note a:focus-visible {
  outline: 3px solid var(--vp-c-brand-1, #6366f1);
  outline-offset: 2px;
  border-radius: 4px;
}

.af-runner {
  margin: 0.25rem 0 0.5rem;
}

.af-load-error {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  border: 1px solid var(--vp-c-border, #313244);
  border-left: 3px solid var(--vp-c-yellow, #e0af68);
  border-radius: var(--site-radius, 0.5rem);
  background: var(--vp-c-bg-soft, #11111b);
}

.af-error-text {
  margin: 0;
  font-size: 0.8rem;
  line-height: 1.6;
  color: var(--vp-c-text-2, #a6adc8);
  overflow-wrap: anywhere;
}

.af-error-actions {
  display: flex;
  gap: 0.5rem;
}

@media (prefers-reduced-motion: reduce) {
  .af-run {
    transition: none;
  }
}
</style>
