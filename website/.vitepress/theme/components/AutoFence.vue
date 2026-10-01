<template>
  <div class="auto-fence">
    <div class="af-toolbar">
      <button
        v-if="!locked"
        type="button"
        class="af-run"
        :class="{ open }"
        :aria-expanded="open"
        :aria-controls="panelId"
        :aria-label="open ? t.collapseAria : t.runAria"
        @click="open = !open"
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
      <SnippetRunner :code="source" autorun height="auto" />
    </div>
    <div v-show="!open" class="af-original">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, useId } from 'vue'
import { useData } from 'vitepress'
import { Play, Square, Lock } from 'lucide-vue-next'
import { SnippetRunner } from 'auto-playground-vue'

const props = defineProps<{
  /** 围栏原文（encodeURIComponent 编码传入；主题层 fence 渲染器注入）。 */
  code: string
}>()

const open = ref(false)

// PLAN-718 T-04：SSR 稳定的唯一 ID（aria-controls 关联运行面板）。
const panelId = `af-panel-${useId()}`

const { lang } = useData()
const zh = computed(() => lang.value.startsWith('zh'))

// PLAN-718 T-05：runner 异步加载与重试状态将扩展此处；T-04 仅 UI 与语义。
const t = computed(() => zh.value
  ? {
      run: '运行',
      collapse: '收起',
      runAria: '运行示例代码',
      collapseAria: '收起示例运行器',
      locked: '依赖模块示例',
      lockNote: '此示例依赖多文件模块，不能在书页内运行——请到',
    }
  : {
      run: 'Run',
      collapse: 'Collapse',
      runAria: 'Run example code',
      collapseAria: 'Collapse the example runner',
      locked: 'Needs modules',
      lockNote: 'This example depends on multi-file modules and cannot run inline — open it in the',
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

@media (prefers-reduced-motion: reduce) {
  .af-run {
    transition: none;
  }
}
</style>
