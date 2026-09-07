<template>
  <div class="expected-output-panel">
    <div v-if="status === 'pending'" class="eop-banner pending">
      <CircleDashed :size="13" />
      <span>期望输出（运行后在此对照实际结果）</span>
    </div>
    <div v-else-if="status === 'match'" class="eop-banner match">
      <CheckCircle2 :size="13" />
      <span>输出一致（{{ expectedLines.length }} 行）</span>
    </div>
    <div v-else class="eop-banner diff">
      <XCircle :size="13" />
      <span>输出有差异（{{ diffCount }} / {{ Math.max(expectedLines.length, actualLines.length) }} 行不一致）</span>
    </div>

    <div v-if="status === 'pending'" class="eop-lines">
      <div v-for="(line, i) in expectedLines" :key="i" class="eop-line">
        <span class="eop-ln">{{ i + 1 }}</span>
        <span class="eop-text">{{ line }}</span>
      </div>
      <div v-if="expectedLines.length === 0" class="eop-empty">（期望输出为空）</div>
    </div>

    <div v-else class="eop-lines">
      <template v-for="i in rowCount" :key="i">
        <div v-if="lineAt(expectedLines, i - 1) === lineAt(actualLines, i - 1)" class="eop-line same">
          <span class="eop-ln">{{ i }}</span>
          <span class="eop-sign">·</span>
          <span class="eop-text">{{ lineAt(expectedLines, i - 1) }}</span>
        </div>
        <template v-else>
          <div v-if="i - 1 < expectedLines.length" class="eop-line minus">
            <span class="eop-ln">{{ i }}</span>
            <span class="eop-sign">-</span>
            <span class="eop-text">{{ expectedLines[i - 1] }}</span>
          </div>
          <div v-if="i - 1 < actualLines.length" class="eop-line plus">
            <span class="eop-ln">{{ i }}</span>
            <span class="eop-sign">+</span>
            <span class="eop-text">{{ actualLines[i - 1] }}</span>
          </div>
        </template>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { CheckCircle2, XCircle, CircleDashed } from 'lucide-vue-next'

const props = defineProps<{
  /** 期望输出（.expected.out/.expected.result 内容）。 */
  expected: string
  /** 实际 stdout（expectedKind='stdout' 时为对照通道）。 */
  actual: string
  /** 实际终值（RunResponse.result；expectedKind='result' 时为对照通道）。 */
  actualResult?: string
  /** 期望语义（P581-D3）：缺省 'stdout'。 */
  expectedKind?: 'stdout' | 'result' | null
  /** 是否已运行过（false=未运行态，展示期望）。 */
  hasRun: boolean
}>()

/** 规范化：CRLF→LF、去行尾空白、去末尾空行（Playground 设计 §6.3）。 */
function toLines(s: string): string[] {
  const lines = s.replace(/\r\n/g, '\n').split('\n').map((l) => l.replace(/[ \t]+$/, ''))
  while (lines.length > 0 && lines[lines.length - 1] === '') lines.pop()
  return lines
}

const expectedLines = computed(() => toLines(props.expected))

// 对照通道按期望语义分派（P581-D3）：result 语义对照 RunResponse.result，否则 stdout。
const actualLines = computed(() => toLines(props.expectedKind === 'result' ? (props.actualResult ?? '') : props.actual))

const status = computed(() => {
  if (!props.hasRun) return 'pending' as const
  return linesEqual(expectedLines.value, actualLines.value) ? ('match' as const) : ('diff' as const)
})

function linesEqual(a: string[], b: string[]): boolean {
  if (a.length !== b.length) return false
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false
  }
  return true
}

const rowCount = computed(() => Math.max(expectedLines.value.length, actualLines.value.length))

const diffCount = computed(() => {
  let n = 0
  for (let i = 0; i < rowCount.value; i++) {
    if (lineAt(expectedLines.value, i) !== lineAt(actualLines.value, i)) n++
  }
  return n
})

function lineAt(lines: string[], i: number): string | undefined {
  return i < lines.length ? lines[i] : undefined
}
</script>

<style scoped>
.expected-output-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  font-family: 'JetBrains Mono', monospace;
}

.eop-banner {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.35rem 0.75rem;
  font-size: 0.75rem;
  font-weight: 600;
  border-bottom: 1px solid var(--nx-border, #313244);
  flex-shrink: 0;
}

.eop-banner.pending {
  color: var(--nx-text-3, #6c7086);
}

.eop-banner.match {
  color: var(--nx-green, #a6e3a1);
  background: rgba(39, 201, 63, 0.08);
}

.eop-banner.diff {
  color: var(--nx-red, #f38ba8);
  background: rgba(243, 139, 168, 0.08);
}

.eop-lines {
  flex: 1;
  min-height: 0;
  overflow: auto;
  font-size: 0.75rem;
}

.eop-line {
  display: flex;
  align-items: baseline;
  padding: 0 0.5rem 0 0;
  white-space: pre;
}

.eop-ln {
  flex-shrink: 0;
  width: 2.2rem;
  text-align: right;
  padding-right: 0.5rem;
  color: var(--nx-text-3, #585b70);
  user-select: none;
}

.eop-sign {
  flex-shrink: 0;
  width: 1.1rem;
  text-align: center;
  user-select: none;
}

.eop-line.same .eop-sign {
  color: var(--nx-text-3, #585b70);
}

.eop-line.minus {
  background: rgba(243, 139, 168, 0.12);
}

.eop-line.minus .eop-sign {
  color: var(--nx-red, #f38ba8);
  font-weight: 700;
}

.eop-line.plus {
  background: rgba(166, 227, 161, 0.1);
}

.eop-line.plus .eop-sign {
  color: var(--nx-green, #a6e3a1);
  font-weight: 700;
}

.eop-text {
  flex: 1;
  min-width: 0;
  color: var(--nx-text-1, #cdd6f4);
}

.eop-empty {
  padding: 0.6rem 0.75rem;
  color: var(--nx-text-3, #6c7086);
  font-size: 0.72rem;
}
</style>
