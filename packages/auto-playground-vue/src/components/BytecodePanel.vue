<template>
  <ScrollArea ref="panelRef" class="bytecode-panel" @mouseover="onOver" @mouseleave="hideTip">
    <div
      v-for="line in bytecode"
      :key="line.offset"
      :data-offset="line.offset"
      :class="['bytecode-line', {
        'is-current': line.offset === currentIp,
        'is-selected': selectedOffsets?.includes(line.offset),
        'is-hover': highlightedOffsets?.includes(line.offset),
        'has-source': line.line !== undefined,
      }]"
      @click="$emit('offsetClick', line.offset)"
    >
      <span class="offset">{{ formatOffset(line.offset) }}</span>
      <span class="mnemonic">{{ line.mnemonic }}</span>
      <span class="operands"><template
        v-for="(part, i) in tokenize(line)"
        :key="i"
      ><span
        v-if="part.tip"
        class="tok"
        :data-tip="part.tip"
      >{{ part.text }}</span><template v-else>{{ part.text }}</template></template></span>

    </div>
    <div
      v-if="tip.visible"
      class="tok-tooltip"
      :style="{ left: tip.x + 'px', top: tip.y + 'px' }"
    >{{ tip.text }}</div>
  </ScrollArea>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from 'vue';
import ScrollArea from './ScrollArea.vue';
import type { BytecodeLine, BytecodeMeta } from '../types';

const props = defineProps<{
  bytecode: BytecodeLine[];
  bytecodeMeta?: BytecodeMeta | null;
  currentIp?: number;
  /** Pinned highlight (clicked source line) */
  selectedOffsets?: number[];
  /** Transient highlight (hovered source line) */
  highlightedOffsets?: number[];
}>();

defineEmits<{
  offsetClick: [offset: number];
}>();

// ---------------------------------------------------------------------------
// Operand tooltips: resolve str[N] / field[N] / method=N / nat#N / hex targets
// against the symbol tables delivered alongside the bytecode.
// ---------------------------------------------------------------------------

const MAX_TIP_LEN = 320;

interface OperandPart {
  text: string;
  tip?: string;
}

function truncate(s: string): string {
  const flat = s.replace(/\\/g, '\\\\').replace(/\n/g, '⏎').replace(/\r/g, '').replace(/\t/g, ' ');
  return flat.length > MAX_TIP_LEN ? flat.slice(0, MAX_TIP_LEN) + '…' : flat;
}

function resolveStringIndex(n: number): string | undefined {
  const s = props.bytecodeMeta?.strings[n];
  return s === undefined ? undefined : `"${truncate(s)}"`;
}

function resolveNative(n: number): string | undefined {
  const name = props.bytecodeMeta?.natives[String(n)];
  return name === undefined ? undefined : truncate(name);
}

function resolveFunction(addr: number): string | undefined {
  const fn = props.bytecodeMeta?.functions.find((f) => f.offset === addr);
  return fn ? `fn ${fn.name}` : undefined;
}

function resolvePart(text: string): string | undefined {
  let m: RegExpMatchArray | null;
  if ((m = text.match(/^str\[(\d+)\]$/))) return resolveStringIndex(Number(m[1]));
  if ((m = text.match(/^field\[(\d+)\]$/))) {
    // field names live in the same merged string pool
    return props.bytecodeMeta?.strings[Number(m[1])];
  }
  if ((m = text.match(/^nat#(\d+)$/))) return resolveNative(Number(m[1]));
  if ((m = text.match(/^method[=-](\d+)(,.*)?$/))) return resolveStringIndex(Number(m[1]));
  if ((m = text.match(/^(?:-> )?(?:addr=)?(0x[0-9a-fA-F]+)$/))) {
    return resolveFunction(parseInt(m[1], 16));
  }
  return undefined;
}

// Splits operands into plain text and reference tokens (str[N], hex targets…)
const TOKEN_RE = /(str\[\d+\]|field\[\d+\]|nat#\d+|(?:-> )?addr=0x[0-9a-fA-F]+|-> 0x[0-9a-fA-F]+|\b0x[0-9a-fA-F]+\b|method[=-]\d+)/g;

function tokenize(line: BytecodeLine): OperandPart[] {
  const operands = line.operands;
  if (!operands) return [{ text: '', tip: undefined }];

  // Global accessors carry their string-pool name index as a bare number
  const bareGlobal = /^(load\.global|store\.global)$/.test(line.mnemonic) && /^\d+$/.test(operands.trim());
  if (bareGlobal && props.bytecodeMeta) {
    const idx = Number(operands.trim());
    const tip = props.bytecodeMeta.strings[idx];
    return [{ text: operands, tip: tip === undefined ? undefined : String(idx) + ': "' + truncate(tip) + '"' }];
  }

  const parts: OperandPart[] = [];
  let last = 0;
  for (const match of operands.matchAll(TOKEN_RE)) {
    const start = match.index ?? 0;
    if (start > last) parts.push({ text: operands.slice(last, start) });
    const text = match[0];
    parts.push({ text, tip: resolvePart(text) });
    last = start + text.length;
  }
  if (last < operands.length) parts.push({ text: operands.slice(last) });
  return parts;
}

// ---------------------------------------------------------------------------
// Tooltip display (event delegation on the panel)
// ---------------------------------------------------------------------------

const panelRef = ref<InstanceType<typeof ScrollArea> | null>(null);
const tip = ref({ visible: false, text: '', x: 0, y: 0 });

function hideTip() {
  tip.value.visible = false;
}

function onOver(e: MouseEvent) {
  const target = (e.target as HTMLElement).closest?.('.tok') as HTMLElement | null;
  const host = (panelRef.value?.$el as HTMLElement) ?? null;
  if (!target || !host || !target.dataset.tip) {
    hideTip();
    return;
  }
  const hostRect = host.getBoundingClientRect();
  const tokRect = target.getBoundingClientRect();
  const TIP_WIDTH = Math.min(hostRect.width - 16, 480);
  tip.value = {
    visible: true,
    text: target.dataset.tip,
    x: Math.max(6, Math.min(tokRect.left - hostRect.left, hostRect.width - TIP_WIDTH - 10)),
    y: tokRect.bottom - hostRect.top + 4,
  };
}

function formatOffset(offset: number): string {
  return offset.toString(16).padStart(4, '0');
}

// Bring the selected line into view when the selection changes
// (block:'nearest' is a no-op for lines already visible). Hover highlights
// intentionally do NOT scroll the panel.
watch(() => props.selectedOffsets, async (offsets) => {
  if (!offsets?.length) return;
  await nextTick();
  panelRef.value?.$el
    ?.querySelector(`[data-offset="${offsets[0]}"]`)
    ?.scrollIntoView({ block: 'nearest' });
});
</script>

<style scoped>
.bytecode-panel {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
  /* Fill the preview pane (flex item of .pane-body); fixes shrink-to-fit */
  flex: 1;
  min-width: 0;
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
.bytecode-line.is-hover {
  background: rgba(86, 156, 214, 0.16);
}
.bytecode-line.is-selected {
  background: rgba(255, 157, 0, 0.2);
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
.tok {
  text-decoration: underline dotted rgba(156, 220, 254, 0.45);
  text-underline-offset: 3px;
  cursor: help;
}
.tok-tooltip {
  position: absolute;
  max-width: 480px;
  padding: 6px 10px;
  background: #252526;
  border: 1px solid #555;
  border-radius: 4px;
  color: #d4d4d4;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  pointer-events: none;
  z-index: 10;
}
</style>
