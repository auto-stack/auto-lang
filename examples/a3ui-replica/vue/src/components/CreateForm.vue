<template>
  <div class="flex flex-col items-center justify-center min-h-screen px-4">
    <h1 class="text-[32px] font-light text-slate-900 mb-6 text-center">
      What would you like to build?
    </h1>
    <div class="w-full max-w-[640px] relative">
      <textarea
        v-model="prompt"
        placeholder="Describe your A2UI widget..."
        class="w-full bg-white border border-slate-300 rounded-xl px-5 py-4 text-base shadow-sm resize-none focus:outline-none focus:border-violet-500 focus:ring-4 focus:ring-violet-500/10 transition-all duration-200"
        rows="3"
        @keydown.enter.prevent="createWidget"
      />
      <button
        class="absolute right-2 bottom-3 bg-violet-600 text-white px-5 py-2 rounded-full text-sm font-medium hover:bg-violet-700 transition-colors"
        @click="createWidget"
      >
        Create
      </button>
    </div>
    <p class="text-xs text-slate-400 mt-3 text-center">Powered by CopilotKit</p>
    <button class="text-sm text-violet-600 mt-2 hover:underline" @click="createBlank">
      or <strong>Start Blank</strong>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const prompt = ref('')

function createWidget() {
  const name = prompt.value.trim() || 'Untitled widget'
  const widget = saveWidget(name)
  router.push(`/widget/${widget.id}?prompt=${encodeURIComponent(prompt.value)}`)
}

function createBlank() {
  const widget = saveWidget('Untitled widget')
  router.push(`/widget/${widget.id}`)
}

function saveWidget(name: string) {
  const widgets = JSON.parse(localStorage.getItem('a2ui-widgets') || '[]')
  const widget = {
    id: Math.random().toString(36).substring(2, 15),
    name,
    components: [
      { id: 'root', component: 'Card', child: 'content' },
      { id: 'content', component: 'Text', value: 'Hello World', variant: 'body' }
    ],
    dataModel: {},
    createdAt: Date.now(),
    updatedAt: Date.now()
  }
  widgets.push(widget)
  localStorage.setItem('a2ui-widgets', JSON.stringify(widgets))
  // Dispatch event to notify sidebar
  window.dispatchEvent(new CustomEvent('a2ui-widgets-changed'))
  return widget
}
</script>
