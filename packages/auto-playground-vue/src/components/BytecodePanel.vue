<template>
  <div class="bytecode-panel">
    <div
      v-for="line in bytecode"
      :key="line.offset"
      :class="['bytecode-line', {
        'is-current': line.offset === currentIp,
        'is-highlighted': highlightedOffsets?.includes(line.offset),
        'has-source': line.line !== undefined,
      }]"
      @click="$emit('offsetClick', line.offset)"
    >
      <span class="offset">{{ formatOffset(line.offset) }}</span>
      <span class="mnemonic">{{ line.mnemonic }}</span>
      <span class="operands">{{ line.operands }}</span>

    </div>
  </div>
</template>

<script setup lang="ts">
import type { BytecodeLine } from '../types';

defineProps<{
  bytecode: BytecodeLine[];
  currentIp?: number;
  highlightedOffsets?: number[];
}>();

defineEmits<{
  offsetClick: [offset: number];
}>();

function formatOffset(offset: number): string {
  return offset.toString(16).padStart(4, '0');
}
</script>

<style scoped>
.bytecode-panel {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
  overflow: auto;
  height: 100%;
  padding: 8px;
  background: #1e1e1e;
  color: #d4d4d4;
}
.bytecode-line {
  display: flex;
  gap: 12px;
  padding: 1px 4px;
  cursor: pointer;
  border-radius: 2px;
}
.bytecode-line:hover {
  background: #2a2d2e;
}
.bytecode-line.is-current {
  background: #0e639c;
  color: #fff;
}
.bytecode-line.is-highlighted {
  background: #7b4a0e;
  border-left: 3px solid #ff9d00;
}
.offset {
  color: #858585;
  min-width: 40px;
  user-select: none;
}
.mnemonic {
  color: #569cd6;
  min-width: 80px;
}
.operands {
  color: #9cdcfe;
  flex: 1;
}

</style>
