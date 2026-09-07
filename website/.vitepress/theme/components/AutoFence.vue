<template>
  <div class="auto-fence">
    <div class="af-toolbar">
      <button v-if="!locked" class="af-run" :class="{ open }" @click="open = !open">
        <Play v-if="!open" :size="13" />
        <Square v-else :size="13" />
        {{ open ? '收起' : 'Run' }}
      </button>
      <span v-else class="af-lock">
        <Lock :size="13" />
        此示例依赖多文件模块，不能在书页内运行——请到
        <a href="/playground">Playground 笔记站</a> 查看
      </span>
    </div>
    <div v-if="open" class="af-runner">
      <SnippetRunner :code="source" autorun height="auto" />
    </div>
    <div v-show="!open" class="af-original">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Play, Square, Lock } from 'lucide-vue-next'
import { SnippetRunner } from 'auto-playground-vue'

const props = defineProps<{
  /** 围栏原文（encodeURIComponent 编码传入；主题层 fence 渲染器注入）。 */
  code: string
}>()

const open = ref(false)

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
  position: relative;
  margin: 0.75rem 0;
}

.af-toolbar {
  position: absolute;
  top: 8px;
  right: 12px;
  z-index: 2;
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

.af-lock a {
  color: var(--vp-c-brand-1, #89b4fa);
}

.af-runner {
  margin-top: 0.5rem;
}
</style>
