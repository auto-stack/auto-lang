<template>
  <div class="flex flex-col h-screen">
    <!-- Control bar -->
    <div class="h-14 flex items-center px-4 border-b border-slate-200 gap-4 bg-white shrink-0">
      <h1 class="text-lg font-semibold text-slate-900">Theater</h1>
      <div class="ml-auto flex gap-2">
        <button
          class="px-3 py-1.5 rounded-md border border-slate-200 text-sm font-medium text-slate-700 hover:bg-slate-50 transition-colors"
          @click="togglePlay"
        >
          {{ isPlaying ? 'Pause' : 'Play' }}
        </button>
        <button
          class="px-3 py-1.5 rounded-md border border-slate-200 text-sm font-medium text-slate-700 hover:bg-slate-50 transition-colors"
          @click="reset"
        >
          Reset
        </button>
      </div>
    </div>

    <!-- Two panels -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Left: JSONL stream -->
      <div class="w-1/2 border-r border-slate-200 flex flex-col bg-slate-900">
        <div class="px-4 py-2 text-xs font-semibold text-slate-400 uppercase tracking-wider border-b border-slate-700">
          JSONL Stream
        </div>
        <div class="flex-1 p-4 font-mono text-sm text-slate-300 overflow-auto">
          <div v-if="streamLines.length === 0" class="text-slate-500">
            Click Play to start streaming...
          </div>
          <div v-for="(line, i) in streamLines" :key="i" class="mb-1">
            {{ line }}
          </div>
        </div>
      </div>

      <!-- Right: Preview -->
      <div class="w-1/2 flex flex-col bg-slate-50">
        <div class="px-4 py-2 text-xs font-semibold text-slate-500 uppercase tracking-wider border-b border-slate-200 bg-white">
          Live Preview
        </div>
        <div class="flex-1 p-4 overflow-auto">
          <div class="h-full bg-white rounded-lg border border-slate-200 overflow-hidden">
            <A2UIRenderer
              v-if="accumulatedComponents.length > 0"
              :components="accumulatedComponents"
              :data-model="{}"
            />
            <div v-else class="flex items-center justify-center h-full text-slate-400 text-sm">
              Waiting for stream...
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import A2UIRenderer from './A2UIRenderer.vue'

const isPlaying = ref(false)
const streamLines = ref<string[]>([])
const accumulatedComponents = ref<any[]>([])

let intervalId: ReturnType<typeof setInterval> | null = null
let currentIndex = 0

const mockChunks = [
  { id: 'root', component: 'Card', child: 'content' },
  { id: 'content', component: 'Column', children: ['header', 'body'], gap: 12 },
  { id: 'header', component: 'Row', children: ['icon', 'title'], gap: 8, align: 'center' },
  { id: 'icon', component: 'Icon', name: 'info', size: 20 },
  { id: 'title', component: 'Text', value: 'Welcome', variant: 'h4' },
  { id: 'body', component: 'Text', value: 'This is a streaming demo of A2UI components being built incrementally.', variant: 'body' },
]

function togglePlay() {
  if (isPlaying.value) {
    pause()
  } else {
    play()
  }
}

function play() {
  isPlaying.value = true
  intervalId = setInterval(() => {
    if (currentIndex >= mockChunks.length) {
      pause()
      return
    }
    const chunk = mockChunks[currentIndex]
    streamLines.value.push(JSON.stringify(chunk))
    accumulatedComponents.value.push(chunk)
    currentIndex++
  }, 800)
}

function pause() {
  isPlaying.value = false
  if (intervalId) {
    clearInterval(intervalId)
    intervalId = null
  }
}

function reset() {
  pause()
  currentIndex = 0
  streamLines.value = []
  accumulatedComponents.value = []
}
</script>
