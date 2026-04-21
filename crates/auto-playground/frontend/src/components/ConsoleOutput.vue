<template>
  <div class="console-output">
    <div v-if="timeMs > 0" class="time-info">Completed in {{ timeMs }}ms</div>
    <pre v-if="stdout" class="stdout">{{ stdout }}</pre>
    <pre v-if="stderr" class="stderr">{{ stderr }}</pre>
    <div v-if="!stdout && !stderr" class="empty">Click Run or press Ctrl+Enter to execute</div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  stdout: string;
  stderr: string;
  timeMs: number;
}>();
</script>

<style scoped>
.console-output {
  padding: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  height: 100%;
  overflow: auto;
  background: #1e1e1e;
  color: #d4d4d4;
}
.stdout {
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
}
.stderr {
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
  color: #f48771;
}
.time-info {
  color: #6a9955;
  font-size: 12px;
  margin-bottom: 8px;
}
.empty {
  color: #6a6a6a;
  font-style: italic;
}
</style>
