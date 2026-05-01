<template>
  <div class="debug-aux-panel">
    <!-- Variables -->
    <div class="aux-section">
      <div class="aux-title">Variables</div>

      <!-- Arguments -->
      <div v-if="props.state?.args?.length" class="var-group">
        <div class="var-group-title">Arguments</div>
        <div v-for="arg in props.state.args" :key="'arg'+arg.index" class="var-row">
          <span class="var-name">arg{{ arg.index }}</span>
          <span class="var-value">{{ arg.value }}</span>
        </div>
      </div>

      <!-- Locals -->
      <div v-if="props.state?.locals?.length" class="var-group">
        <div class="var-group-title">Locals</div>
        <div v-for="local in props.state.locals" :key="'loc'+local.index" class="var-row">
          <span class="var-name">local{{ local.index }}</span>
          <span class="var-value">{{ local.value }}</span>
        </div>
      </div>

      <!-- Stack Top (evaluation stack) -->
      <div v-if="displayStack.length" class="var-group">
        <div class="var-group-title">Stack Top</div>
        <div
          v-for="(item, idx) in displayStack"
          :key="'stk'+idx"
          :class="['var-row', { 'is-top': idx === 0, 'is-pushed': hasPush && idx === 0, 'is-popped': hasPop && idx === 0 }]"
        >
          <span class="var-name">[{{ item.distFromTop }}]</span>
          <span class="var-value">{{ item.value }}</span>
        </div>
      </div>

      <div v-if="!hasVariables" class="empty">No variables</div>
    </div>

    <!-- Call Stack -->
    <div class="aux-section">
      <div class="aux-title">Call Stack</div>
      <div class="callstack-list">
        <div
          v-for="(frame, idx) in displayCallStack"
          :key="idx"
          :class="['callstack-item', { 'is-current': idx === 0 }]"
        >
          <span class="cs-name">{{ frame.fn_name ?? '<main>' }}</span>
          <span class="cs-line">:{{ frame.line }}</span>
        </div>
      </div>
      <div v-if="!props.state?.call_stack?.length" class="empty">No frames</div>
    </div>

    <!-- Registers (compact) -->
    <div class="aux-section compact">
      <div class="aux-title">Registers</div>
      <div class="reg-row">
        <span class="reg-label">IP</span>
        <span class="reg-value">{{ formatHex(state?.registers.ip) }}</span>
      </div>
      <div class="reg-row">
        <span class="reg-label">BP</span>
        <span class="reg-value">{{ formatHex(state?.registers.bp) }}</span>
      </div>
      <div class="reg-row">
        <span class="reg-label">SP</span>
        <span class="reg-value">{{ formatHex(state?.registers.sp) }}</span>
      </div>
    </div>

    <!-- Stdout -->
    <div class="aux-section stdout-section" v-if="state?.stdout">
      <div class="aux-title">Output</div>
      <pre class="stdout-content">{{ state.stdout }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { DebugState } from '../types';

const props = defineProps<{
  state: DebugState | null;
}>();

const hasPush = ref(false);
const hasPop = ref(false);

watch(() => props.state?.stack, (newStack, oldStack) => {
  const ns = newStack || [];
  const os = oldStack || [];
  if (ns.length > os.length) {
    hasPush.value = true;
  } else if (ns.length < os.length) {
    hasPop.value = true;
  }
  setTimeout(() => {
    hasPush.value = false;
    hasPop.value = false;
  }, 600);
}, { deep: true, flush: 'post' });

const displayStack = computed(() => {
  if (!props.state) return [];
  const stack = props.state.stack;
  // Show top 8 items of the evaluation stack
  const topN = Math.min(stack.length, 8);
  const topSlice = stack.slice(-topN);
  return [...topSlice].reverse().map((val, distFromTop) => ({
    value: val,
    distFromTop,
  }));
});

const displayCallStack = computed(() => {
  if (!props.state) return [];
  // Current function is not in call_stack; prepend it
  const frames = [...props.state.call_stack];
  frames.push({
    fn_name: null,
    line: props.state.line,
    return_ip: props.state.registers.ip,
    bp: props.state.registers.bp,
    n_args: props.state.args?.length ?? 0,
    n_locals: props.state.locals?.length ?? 0,
  });
  return frames.reverse();
});

const hasVariables = computed(() => {
  return (props.state?.args?.length ?? 0) > 0
    || (props.state?.locals?.length ?? 0) > 0
    || displayStack.value.length > 0;
});

function formatHex(n: number | undefined): string {
  if (n === undefined) return '-';
  return `0x${n.toString(16).padStart(4, '0')}`;
}
</script>

<style scoped>
.debug-aux-panel {
  width: 260px;
  min-width: 200px;
  border-left: 1px solid #444;
  background: #1e1e1e;
  color: #d4d4d4;
  font-size: 12px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0;
}

.aux-section {
  border-bottom: 1px solid #333;
  padding: 8px;
}

.aux-title {
  font-weight: 600;
  font-size: 11px;
  color: #999;
  text-transform: uppercase;
  margin-bottom: 6px;
  letter-spacing: 0.5px;
}

/* Variable groups */
.var-group {
  margin-bottom: 6px;
}

.var-group-title {
  font-size: 11px;
  color: #569cd6;
  font-weight: 500;
  margin-bottom: 2px;
  padding-left: 2px;
}

.var-row {
  display: flex;
  justify-content: space-between;
  padding: 2px 4px;
  border-radius: 2px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  gap: 8px;
}

.var-row.is-top {
  background: #0e639c30;
  border-left: 2px solid #0e639c;
}

.var-row.is-pushed {
  animation: flash-green 0.6s ease;
}

.var-row.is-popped {
  animation: flash-red 0.6s ease;
}

.var-name {
  color: #9cdcfe;
  user-select: none;
}

.var-value {
  color: #ce9178;
  text-align: right;
}

/* Call Stack */
.callstack-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.callstack-item {
  display: flex;
  gap: 6px;
  padding: 2px 4px;
  border-radius: 2px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  align-items: center;
}

.callstack-item.is-current {
  background: #0e639c30;
  border-left: 2px solid #0e639c;
}

.cs-name {
  color: #dcdcaa;
  flex: 1;
}

.cs-line {
  color: #6a9955;
  font-size: 11px;
}

/* Registers */
.reg-row {
  display: flex;
  justify-content: space-between;
  padding: 1px 0;
}

.reg-label {
  color: #569cd6;
  font-weight: 500;
}

.reg-value {
  color: #9cdcfe;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

/* Stdout */
.stdout-section {
  flex: 1;
  min-height: 60px;
  display: flex;
  flex-direction: column;
}

.stdout-content {
  flex: 1;
  margin: 0;
  padding: 4px;
  background: #252526;
  border-radius: 3px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 11px;
  color: #ccc;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
}

.empty {
  color: #666;
  font-style: italic;
  padding: 4px;
  text-align: center;
}

@keyframes flash-green {
  0% { background: #23863660; }
  100% { background: transparent; }
}

@keyframes flash-red {
  0% { background: #e5140060; }
  100% { background: transparent; }
}
</style>
