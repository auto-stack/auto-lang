<template>
  <div class="flex flex-col h-screen">
    <!-- Top control bar -->
    <div class="h-14 flex items-center px-4 border-b border-slate-200 gap-4 bg-white shrink-0">
      <h1 class="text-lg font-semibold text-slate-900">Theater</h1>
      <div class="ml-auto flex items-center gap-3">
        <!-- Tabs -->
        <div class="flex rounded-lg border border-slate-200 overflow-hidden">
          <button
            v-for="tab in ['Events', 'Data', 'Config']"
            :key="tab"
            class="px-3 py-1.5 text-sm font-medium transition-colors"
            :class="activeTab === tab ? 'bg-slate-100 text-slate-900' : 'text-slate-500 hover:text-slate-700 hover:bg-slate-50'"
            @click="activeTab = tab"
          >
            {{ tab }}
          </button>
        </div>
        <!-- Playback controls -->
        <button
          class="w-8 h-8 flex items-center justify-center rounded-full bg-slate-900 text-white hover:bg-slate-700 transition-colors"
          @click="togglePlay"
        >
          <component :is="isPlaying ? Pause : Play" class="w-4 h-4" />
        </button>
        <!-- Speed -->
        <button class="px-2 py-1 rounded-md border border-slate-200 text-xs font-medium text-slate-600 hover:bg-slate-50">
          1x
        </button>
      </div>
    </div>

    <!-- Two panels -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Left: JSONL stream -->
      <div class="w-1/2 border-r border-slate-200 flex flex-col bg-slate-900">
        <div class="px-4 py-2 text-xs font-semibold text-slate-400 uppercase tracking-wider border-b border-slate-700 flex items-center gap-2">
          <span class="text-amber-400">⚡</span> JSONL Stream
        </div>
        <div class="flex-1 p-4 font-mono text-sm text-slate-300 overflow-auto">
          <div v-if="streamLines.length === 0" class="text-slate-500 italic">
            Press play to stream JSONL chunks...
          </div>
          <div v-for="(line, i) in streamLines" :key="i" class="mb-1">
            {{ line }}
          </div>
        </div>
      </div>

      <!-- Right: Preview with browser chrome -->
      <div class="w-1/2 flex flex-col bg-slate-50">
        <!-- Browser chrome -->
        <div class="px-3 py-2 bg-white border-b border-slate-200 flex items-center gap-3">
          <!-- Traffic lights -->
          <div class="flex gap-1.5">
            <div class="w-3 h-3 rounded-full bg-red-400" />
            <div class="w-3 h-3 rounded-full bg-amber-400" />
            <div class="w-3 h-3 rounded-full bg-green-400" />
          </div>
          <!-- Address bar -->
          <div class="flex-1 bg-slate-100 rounded-md px-3 py-1 text-xs text-slate-500 flex items-center gap-2">
            <span class="text-slate-400">&lt;/&gt;</span>
            React Renderer
          </div>
          <!-- URL -->
          <div class="text-xs text-slate-400">restaurant_finder</div>
        </div>
        <div class="flex-1 p-4 overflow-auto">
          <div class="h-full bg-white rounded-lg border border-slate-200 overflow-hidden shadow-sm">
            <A2UIRenderer
              v-if="accumulatedComponents.length > 0"
              :components="accumulatedComponents"
              :data-model="{}"
            />
            <div v-else class="flex flex-col items-center justify-center h-full text-slate-400 text-sm gap-3">
              <div class="w-12 h-12 rounded-full bg-slate-100 flex items-center justify-center">
                <Code class="w-6 h-6 text-slate-400" />
              </div>
              <div>&lt;A2UIRenderer /&gt;</div>
              <div class="text-xs text-slate-300">Press play to start streaming</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Play, Pause, Code } from 'lucide-vue-next'
import A2UIRenderer from './A2UIRenderer.vue'

const isPlaying = ref(false)
const activeTab = ref('Events')
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
</script>
