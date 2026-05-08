<template>
  <div class="flex flex-col h-screen">
    <!-- Top bar -->
    <div class="h-14 flex items-center px-4 border-b border-slate-200 gap-4 bg-white shrink-0">
      <input
        v-model="widgetName"
        class="font-medium text-slate-800 border-none focus:outline-none bg-transparent flex-1"
      />
      <div class="flex gap-2">
        <button
          class="px-3 py-1.5 rounded-md bg-violet-600 text-white text-sm font-medium hover:bg-violet-700 transition-colors"
          @click="saveToLocalStorage"
        >
          Save
        </button>
        <button
          class="px-3 py-1.5 rounded-md border border-slate-200 text-sm font-medium text-slate-700 hover:bg-slate-50 transition-colors"
          @click="copyJson"
        >
          Copy JSON
        </button>
        <button
          class="px-3 py-1.5 rounded-md border border-slate-200 text-sm font-medium text-slate-700 hover:bg-slate-50 transition-colors"
          @click="downloadJson"
        >
          Download
        </button>
      </div>
    </div>

    <!-- Main area: 3 panels + bottom data model -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Left: JSON editor -->
      <div class="w-[40%] border-r border-slate-200 flex flex-col bg-white">
        <div class="px-4 py-2 text-xs font-semibold text-slate-500 uppercase tracking-wider border-b border-slate-200 flex items-center justify-between">
          <span>JSON Editor</span>
          <span v-if="parseError" class="text-red-500 text-xs normal-case font-normal truncate max-w-[200px]">{{ parseError }}</span>
        </div>
        <div class="flex-1 flex overflow-hidden">
          <!-- Line numbers -->
          <div class="w-10 bg-slate-100 border-r border-slate-200 py-4 text-right pr-2 text-xs text-slate-400 font-mono select-none overflow-hidden">
            <div v-for="n in lineCount" :key="n">{{ n }}</div>
          </div>
          <textarea
            v-model="widgetJson"
            class="flex-1 p-4 font-mono text-sm resize-none focus:outline-none bg-slate-50"
            spellcheck="false"
          />
        </div>
      </div>

      <!-- Center: Preview + Data Model -->
      <div class="w-[35%] border-r border-slate-200 flex flex-col bg-slate-50">
        <div class="px-4 py-2 text-xs font-semibold text-slate-500 uppercase tracking-wider border-b border-slate-200 bg-white flex items-center justify-between">
          <span>Preview</span>
          <span v-if="parseError" class="text-amber-500 text-xs normal-case font-normal">JSON error — preview paused</span>
        </div>
        <div class="flex-1 p-4 overflow-auto">
          <div class="h-full bg-white rounded-lg border border-slate-200 overflow-hidden relative">
            <!-- Dot grid background -->
            <div class="absolute inset-0 opacity-20 pointer-events-none" style="background-image: radial-gradient(circle, #cbd5e1 1px, transparent 1px); background-size: 16px 16px;" />
            <div class="relative z-10 h-full">
              <A2UIRenderer
                v-if="debouncedResult.components.length > 0 && !parseError"
                :components="debouncedResult.components"
                :data-model="debouncedResult.dataModel"
              />
              <div v-else class="flex flex-col items-center justify-center h-full text-slate-400 text-sm gap-2">
                <div v-if="parseError" class="text-amber-500">
                  <svg class="w-8 h-8 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" /></svg>
                  Invalid JSON
                </div>
                <div v-else>
                  No components to render
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Bottom: Data Model Editor -->
        <div class="h-40 border-t border-slate-200 flex flex-col bg-white shrink-0">
          <div class="px-4 py-2 text-xs font-semibold text-slate-500 uppercase tracking-wider border-b border-slate-200 flex items-center justify-between">
            <span>Data Model</span>
            <span v-if="dataModelError" class="text-red-500 text-xs normal-case font-normal">{{ dataModelError }}</span>
          </div>
          <textarea
            v-model="dataModelJson"
            class="flex-1 w-full p-4 font-mono text-sm resize-none focus:outline-none bg-slate-50"
            spellcheck="false"
            placeholder="{}"
          />
        </div>
      </div>

      <!-- Right: AI Chat -->
      <div class="w-[25%] flex flex-col bg-white">
        <div class="px-4 py-2 text-xs font-semibold text-slate-500 uppercase tracking-wider border-b border-slate-200">
          AI Assistant
        </div>
        <div class="flex-1 p-4 overflow-auto">
          <div class="bg-slate-50 rounded-lg p-3 text-sm text-slate-600">
            <p class="font-medium mb-1">AI Chat</p>
            <p class="text-slate-500">Ask me to modify your widget or generate new components.</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Toast notification -->
    <transition enter-active-class="transition ease-out duration-200" enter-from-class="opacity-0 translate-y-2" enter-to-class="opacity-100 translate-y-0" leave-active-class="transition ease-in duration-150" leave-from-class="opacity-100 translate-y-0" leave-to-class="opacity-0 translate-y-2">
      <div v-if="toastVisible" class="fixed bottom-4 right-4 z-50">
        <div class="px-4 py-2.5 bg-slate-800 text-white text-sm rounded-lg shadow-lg flex items-center gap-2">
          <svg class="w-4 h-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
          {{ toastMessage }}
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import A2UIRenderer from './A2UIRenderer.vue'

const route = useRoute()

const widgetName = ref('Untitled widget')

// Toast notification
const toastVisible = ref(false)
const toastMessage = ref('')
let toastTimer: ReturnType<typeof setTimeout> | null = null
function showToast(message: string) {
  toastMessage.value = message
  toastVisible.value = true
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toastVisible.value = false }, 2000)
}

// Default widget components
const defaultComponents = [
  { id: 'root', component: 'Card', child: 'content' },
  { id: 'content', component: 'Text', value: 'Hello World', variant: 'body' }
]

const widgetJson = ref(JSON.stringify(defaultComponents, null, 2))
const dataModelJson = ref('{}')

// Debounced parse result
const debouncedResult = ref<{ components: any[], dataModel: any }>({
  components: [],
  dataModel: {}
})
const parseError = ref('')
const dataModelError = ref('')

let debounceTimer: ReturnType<typeof setTimeout> | null = null

function performParse() {
  // Parse components
  try {
    const parsed = JSON.parse(widgetJson.value)
    if (Array.isArray(parsed)) {
      debouncedResult.value.components = parsed
    } else if (parsed && Array.isArray(parsed.components)) {
      debouncedResult.value.components = parsed.components
    } else {
      debouncedResult.value.components = []
    }
    parseError.value = ''
  } catch (e: any) {
    parseError.value = e.message || 'Invalid JSON'
  }

  // Parse data model
  try {
    debouncedResult.value.dataModel = JSON.parse(dataModelJson.value)
    dataModelError.value = ''
  } catch (e: any) {
    dataModelError.value = e.message || 'Invalid JSON'
  }
}

watch([widgetJson, dataModelJson], () => {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(performParse, 300)
}, { immediate: true })

// Line numbers
const lineCount = computed(() => widgetJson.value.split('\n').length)

// Load widget data from localStorage by route param id
function loadWidgetById(id: string) {
  const widgets = JSON.parse(localStorage.getItem('a2ui-widgets') || '[]')
  const widget = widgets.find((w: any) => w.id === id)
  if (widget) {
    widgetName.value = widget.name || 'Untitled widget'
    widgetJson.value = JSON.stringify(widget.components || defaultComponents, null, 2)
    dataModelJson.value = JSON.stringify(widget.dataModel || {}, null, 2)
  }
}

watch(() => route.params.id, (id) => {
  if (id && typeof id === 'string') {
    loadWidgetById(id)
  }
}, { immediate: true })

// Also support route query data (for direct sharing)
watch(() => route.query.data, (data) => {
  if (data && typeof data === 'string') {
    try {
      const widget = JSON.parse(data)
      widgetName.value = widget.name || 'Untitled widget'
      widgetJson.value = JSON.stringify(widget.components || defaultComponents, null, 2)
      dataModelJson.value = JSON.stringify(widget.dataModel || {}, null, 2)
    } catch {
      // ignore
    }
  }
}, { immediate: true })

function saveToLocalStorage() {
  const id = route.params.id as string
  if (!id) return
  const widgets = JSON.parse(localStorage.getItem('a2ui-widgets') || '[]')
  const idx = widgets.findIndex((w: any) => w.id === id)
  const updated = {
    id,
    name: widgetName.value,
    components: debouncedResult.value.components,
    dataModel: debouncedResult.value.dataModel,
    updatedAt: Date.now(),
    createdAt: widgets[idx]?.createdAt || Date.now()
  }
  if (idx >= 0) {
    widgets[idx] = updated
  } else {
    widgets.push(updated)
  }
  localStorage.setItem('a2ui-widgets', JSON.stringify(widgets))
  window.dispatchEvent(new CustomEvent('a2ui-widgets-changed'))
  showToast('Widget saved')
}

function copyJson() {
  const fullWidget = {
    name: widgetName.value,
    components: debouncedResult.value.components,
    dataModel: debouncedResult.value.dataModel
  }
  navigator.clipboard.writeText(JSON.stringify(fullWidget, null, 2))
  showToast('JSON copied to clipboard')
}

function downloadJson() {
  const fullWidget = {
    name: widgetName.value,
    components: debouncedResult.value.components,
    dataModel: debouncedResult.value.dataModel
  }
  const blob = new Blob([JSON.stringify(fullWidget, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${widgetName.value.replace(/\s+/g, '-').toLowerCase()}.json`
  a.click()
  URL.revokeObjectURL(url)
  showToast('JSON downloaded')
}
</script>
