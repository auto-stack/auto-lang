<template>
  <div class="debug-panel">
    <div class="debug-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        {{ tab.label }}
      </button>
    </div>
    <div class="debug-content">
      <!-- Stack -->
      <div v-if="activeTab === 'stack'" class="stack-view">
        <table>
          <thead>
            <tr><th>Index</th><th>Value</th></tr>
          </thead>
          <tbody>
            <tr
              v-for="item in stackDisplay"
              :key="item.idx"
              :class="{
                'is-top': item.isTop,
                'stack-pushed': item.isPushed,
                'stack-popped': item.isPopped,
              }"
            >
              <td>{{ item.idx }}</td>
              <td>{{ item.val }}</td>
            </tr>
          </tbody>
        </table>
        <div v-if="state.stack.length === 0" class="empty">Stack empty</div>
      </div>

      <!-- Call Stack -->
      <div v-if="activeTab === 'callstack'" class="callstack-view">
        <div
          v-for="(frame, idx) in reversedCallStack"
          :key="idx"
          class="frame-item"
        >
          <span class="frame-idx">#{{ reversedCallStack.length - 1 - idx }}</span>
          <span class="frame-name">{{ frame.fn_name ?? '<anonymous>' }}</span>
          <span class="frame-line">line {{ frame.line }}</span>
        </div>
        <div v-if="state.call_stack.length === 0" class="empty">No frames</div>
      </div>

      <!-- Locals -->
      <div v-if="activeTab === 'locals'" class="locals-view">
        <table>
          <thead>
            <tr><th>Slot</th><th>Value</th></tr>
          </thead>
          <tbody>
            <tr v-for="local in state.locals" :key="local.index">
              <td>[{{ local.index }}]</td>
              <td>{{ local.value }}</td>
            </tr>
          </tbody>
        </table>
        <div v-if="state.locals.length === 0" class="empty">No locals</div>
      </div>

      <!-- Registers -->
      <div v-if="activeTab === 'registers'" class="registers-view">
        <div class="reg-row"><span class="reg-name">IP</span><span class="reg-val">{{ formatHex(state.registers.ip) }}</span></div>
        <div class="reg-row"><span class="reg-name">BP</span><span class="reg-val">{{ formatHex(state.registers.bp) }}</span></div>
        <div class="reg-row"><span class="reg-name">SP</span><span class="reg-val">{{ formatHex(state.registers.sp) }}</span></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import type { DebugState } from '../types';

const props = defineProps<{
  state: DebugState;
}>();

const activeTab = ref<'stack' | 'callstack' | 'locals' | 'registers'>('stack');

const tabs = [
  { id: 'stack' as const, label: 'Stack' },
  { id: 'callstack' as const, label: 'Call Stack' },
  { id: 'locals' as const, label: 'Locals' },
  { id: 'registers' as const, label: 'Registers' },
];

// Stack diff animation
const pushedIndices = ref<Set<number>>(new Set());
const poppedIndices = ref<Set<number>>(new Set());

watch(() => props.state.stack, async (newStack, oldStack) => {
  const oldArr = oldStack || [];
  const newArr = newStack || [];
  const added = new Set<number>();
  const removed = new Set<number>();

  // Find pushed items (present in new but not at same index in old)
  for (let i = oldArr.length; i < newArr.length; i++) {
    added.add(i);
  }
  // Find popped items (present in old but not in new)
  for (let i = newArr.length; i < oldArr.length; i++) {
    removed.add(i);
  }

  pushedIndices.value = added;
  poppedIndices.value = removed;

  await nextTick();
  setTimeout(() => {
    pushedIndices.value = new Set();
    poppedIndices.value = new Set();
  }, 600);
}, { deep: true, flush: 'post' });

const stackDisplay = computed(() => {
  const arr = [...props.state.stack].reverse();
  return arr.map((val, revIdx) => {
    const idx = props.state.stack.length - 1 - revIdx;
    return {
      val,
      idx,
      isTop: revIdx === 0,
      isPushed: pushedIndices.value.has(idx),
      isPopped: poppedIndices.value.has(idx),
    };
  });
});

const reversedCallStack = computed(() => [...props.state.call_stack].reverse());

function formatHex(n: number): string {
  return `0x${n.toString(16).padStart(4, '0')} (${n})`;
}
</script>

<style scoped>
.debug-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1e1e1e;
  color: #d4d4d4;
  font-size: 13px;
}
.debug-tabs {
  display: flex;
  background: #2d2d2d;
  border-bottom: 1px solid #444;
}
.debug-tabs button {
  padding: 6px 14px;
  background: none;
  border: none;
  color: #ccc;
  cursor: pointer;
  font-size: 12px;
}
.debug-tabs button.active {
  background: #1e1e1e;
  color: #fff;
  border-bottom: 2px solid #0e639c;
}
.debug-content {
  flex: 1;
  overflow: auto;
  padding: 8px;
}
.stack-view table, .locals-view table {
  width: 100%;
  border-collapse: collapse;
}
.stack-view th, .locals-view th {
  text-align: left;
  padding: 4px;
  color: #858585;
  font-weight: 500;
  border-bottom: 1px solid #444;
}
.stack-view td, .locals-view td {
  padding: 3px 4px;
  font-family: monospace;
}
.stack-view .is-top td {
  background: #0e639c30;
  color: #fff;
}
.stack-view .stack-pushed td {
  animation: flash-green 0.6s ease;
}
.stack-view .stack-popped td {
  animation: flash-red 0.6s ease;
}
@keyframes flash-green {
  0% { background: #4caf5040; }
  100% { background: transparent; }
}
@keyframes flash-red {
  0% { background: #f4433640; }
  100% { background: transparent; }
}
.frame-item {
  padding: 4px;
  display: flex;
  gap: 8px;
}
.frame-idx { color: #858585; min-width: 28px; }
.frame-name { color: #9cdcfe; }
.frame-line { color: #6a9955; }
.reg-row {
  display: flex;
  gap: 12px;
  padding: 4px;
}
.reg-name { color: #569cd6; min-width: 40px; }
.reg-val { font-family: monospace; }
.empty { color: #858585; padding: 12px; text-align: center; }
</style>
