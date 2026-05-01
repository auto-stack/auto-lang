<template>
  <div class="replay-toolbar">
    <button
      @click="isPlaying ? $emit('pause') : $emit('play')"
      :title="isPlaying ? 'Pause' : 'Play'"
    >
      <span class="icon">{{ isPlaying ? '⏸' : '▶' }}</span>
    </button>
    <button @click="$emit('stepBackward')" title="Step Backward (←)">
      <span class="icon">⏮</span>
    </button>
    <button @click="$emit('stepForward')" title="Step Forward (→)">
      <span class="icon">⏭</span>
    </button>

    <div class="timeline">
      <input
        type="range"
        :min="0"
        :max="Math.max(0, totalFrames - 1)"
        :value="currentFrame"
        @input="onSeek"
        class="timeline-slider"
      />
      <span class="frame-info">Frame {{ currentFrame + 1 }} / {{ totalFrames }}</span>
    </div>

    <div class="replay-badge">🔁 Replay Mode</div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  isPlaying: boolean;
  currentIndex: number;
  totalFrames: number;
}>();

const emit = defineEmits<{
  play: [];
  pause: [];
  stepForward: [];
  stepBackward: [];
  seek: [index: number];
}>();

const currentFrame = computed(() => props.currentIndex);

function onSeek(e: Event) {
  const val = parseInt((e.target as HTMLInputElement).value, 10);
  emit('seek', val);
}
</script>

<script lang="ts">
import { computed } from 'vue';
</script>

<style scoped>
.replay-toolbar {
  display: flex;
  gap: 4px;
  padding: 4px 12px;
  background: #1e3a2f;
  border-bottom: 1px solid #2d5a3f;
  align-items: center;
  flex-shrink: 0;
}
.replay-toolbar button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 28px;
  background: #2d4a3f;
  border: 1px solid #3d6a5f;
  border-radius: 3px;
  color: #ccc;
  cursor: pointer;
  font-size: 14px;
  padding: 0;
}
.replay-toolbar button:hover {
  background: #3d5a4f;
  color: #fff;
}
.timeline {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0 8px;
}
.timeline-slider {
  flex: 1;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: #3d6a5f;
  border-radius: 2px;
  outline: none;
}
.timeline-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 12px;
  height: 12px;
  background: #4ec9b0;
  border-radius: 50%;
  cursor: pointer;
}
.timeline-slider::-moz-range-thumb {
  width: 12px;
  height: 12px;
  background: #4ec9b0;
  border-radius: 50%;
  cursor: pointer;
  border: none;
}
.frame-info {
  font-size: 11px;
  color: #aaa;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  white-space: nowrap;
  min-width: 80px;
}
.replay-badge {
  font-size: 11px;
  color: #4ec9b0;
  background: #1a3a2f;
  padding: 2px 8px;
  border-radius: 3px;
  border: 1px solid #2d5a3f;
  white-space: nowrap;
}
</style>
