<template>
  <div class="debug-aux-panel">
    <div class="aux-section">
      <div class="aux-title">Registers</div>
      <div class="aux-row">
        <span class="aux-label">IP</span>
        <span class="aux-value">{{ formatHex(state?.registers.ip) }}</span>
      </div>
      <div class="aux-row">
        <span class="aux-label">BP</span>
        <span class="aux-value">{{ formatHex(state?.registers.bp) }}</span>
      </div>
      <div class="aux-row">
        <span class="aux-label">SP</span>
        <span class="aux-value">{{ formatHex(state?.registers.sp) }}</span>
      </div>
    </div>

    <div class="aux-section">
      <div class="aux-title">Stack</div>
      <div class="stack-list">
        <div
          v-for="(val, idx) in displayStack"
          :key="idx"
          :class="['stack-item', { 'is-top': idx === 0, 'is-pushed': hasPush && idx === 0, 'is-popped': hasPop && idx === 0 }]"
        >
          <span class="stack-idx">[{{ val.distFromTop }}]</span>
          <span class="stack-val">{{ val.value }}</span>
        </div>
      </div>
      <div v-if="!state?.stack.length" class="empty">Stack empty</div>
    </div>

    <div v-if="lastPoppedValue !== null" class="aux-section">
      <div class="aux-title">Last Popped</div>
      <div class="aux-row">
        <span class="aux-value pop-value">{{ lastPoppedValue }}</span>
      </div>
    </div>

    <div class="aux-section">
      <div class="aux-title">Call Stack</div>
      <div class="callstack-list">
        <div
          v-for="(frame, idx) in reversedCallStack"
          :key="idx"
          :class="['callstack-item', { 'is-current': idx === 0 }]"
        >
          <span class="cs-idx">#{{ reversedCallStack.length - 1 - idx }}</span>
          <span class="cs-name">{{ frame.fn_name ?? '<anonymous>' }}</span>
          <span class="cs-line">line {{ frame.line }}</span>
        </div>
      </div>
      <div v-if="!state?.call_stack.length" class="empty">No frames</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { DebugState } from '../types';

const props = defineProps<{
  state: DebugState | null;
}>();

const lastStack = ref<string[]>([]);
const lastPoppedValue = ref<string | null>(null);
const hasPush = ref(false);
const hasPop = ref(false);

watch(() => props.state?.stack, (newStack, oldStack) => {
  const ns = newStack || [];
  const os = oldStack || [];

  if (ns.length > os.length) {
    hasPush.value = true;
  } else if (ns.length < os.length) {
    hasPop.value = true;
    lastPoppedValue.value = os[os.length - 1] ?? null;
  }

  lastStack.value = [...ns];

  // Clear animation after 600ms
  setTimeout(() => {
    hasPush.value = false;
    hasPop.value = false;
  }, 600);
}, { deep: true, flush: 'post' });

const displayStack = computed(() => {
  if (!props.state) return [];
  const stack = props.state.stack;
  // Show top-of-stack first with distance-from-top index: [0] = top, [1] = next, ...
  return [...stack].reverse().map((val, distFromTop) => ({
    value: val,
    distFromTop,
  }));
});

const reversedCallStack = computed(() => {
  if (!props.state) return [];
  return [...props.state.call_stack].reverse();
});

function formatHex(n: number | undefined): string {
  if (n === undefined) return '-';
  return `0x${n.toString(16).padStart(4, '0')} (${n})`;
}
</script>

<style scoped>
.debug-aux-panel {
  width: 240px;
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

.aux-row {
  display: flex;
  justify-content: space-between;
  padding: 2px 0;
}

.aux-label {
  color: #569cd6;
  font-weight: 500;
}

.aux-value {
  color: #9cdcfe;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.pop-value {
  color: #ce9178;
  font-weight: 600;
}

.stack-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.stack-item {
  display: flex;
  gap: 6px;
  padding: 2px 4px;
  border-radius: 2px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.stack-item.is-top {
  background: #0e639c30;
  border-left: 2px solid #0e639c;
}

.stack-item.is-pushed {
  background: #23863630;
  animation: flash-green 0.6s ease;
}

.stack-item.is-popped {
  background: #e5140030;
  animation: flash-red 0.6s ease;
}

.stack-idx {
  color: #858585;
  min-width: 28px;
  user-select: none;
}

.stack-val {
  color: #d4d4d4;
}

.callstack-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
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
}

.cs-idx {
  color: #858585;
  min-width: 20px;
}

.cs-name {
  color: #dcdcaa;
  flex: 1;
}

.cs-line {
  color: #6a9955;
  font-size: 11px;
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
