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

    <!-- Three panels -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Left: JSON editor -->
      <div class="w-[40%] border-r border-slate-200 flex flex-col bg-white">
        <div class="px-4 py-2 text-xs font-semibold text-slate-500 uppercase tracking-wider border-b border-slate-200">
          JSON Editor
        </div>
        <textarea
          v-model="widgetJson"
          class="flex-1 w-full p-4 font-mono text-sm resize-none focus:outline-none bg-slate-50"
          spellcheck="false"
        />
      </div>

      <!-- Center: Preview -->
      <div class="w-[35%] border-r border-slate-200 flex flex-col bg-slate-50">
        <div class="px-4 py-2 text-xs font-semibold text-slate-500 uppercase tracking-wider border-b border-slate-200 bg-white">
          Preview
        </div>
        <div class="flex-1 p-4 overflow-auto">
          <div class="h-full bg-white rounded-lg border border-slate-200 overflow-hidden">
            <A2UIRenderer
              v-if="parsedComponents.length > 0"
              :components="parsedComponents"
              :data-model="parsedDataModel"
            />
            <div v-else class="flex items-center justify-center h-full text-slate-400 text-sm">
              No components to render
            </div>
          </div>
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import A2UIRenderer from './A2UIRenderer.vue'

const route = useRoute()

const widgetName = ref('Untitled widget')

// Default widget components
const defaultComponents = [
  { id: 'root', component: 'Card', child: 'content' },
  { id: 'content', component: 'Text', value: 'Hello World', variant: 'body' }
]

const widgetJson = ref(JSON.stringify(defaultComponents, null, 2))

// Parse JSON for preview
const parsedComponents = computed(() => {
  try {
    const parsed = JSON.parse(widgetJson.value)
    if (Array.isArray(parsed)) {
      return parsed
    }
    // If it's a full widget object with components property
    if (parsed && Array.isArray(parsed.components)) {
      widgetName.value = parsed.name || widgetName.value
      return parsed.components
    }
    return []
  } catch {
    return []
  }
})

const parsedDataModel = computed(() => {
  try {
    const parsed = JSON.parse(widgetJson.value)
    if (parsed && parsed.dataModel) {
      return parsed.dataModel
    }
    return {}
  } catch {
    return {}
  }
})

// Load widget data from localStorage by route param id
function loadWidgetById(id: string) {
  const widgets = JSON.parse(localStorage.getItem('a2ui-widgets') || '[]')
  const widget = widgets.find((w: any) => w.id === id)
  if (widget) {
    widgetName.value = widget.name || 'Untitled widget'
    widgetJson.value = JSON.stringify(widget.components || defaultComponents, null, 2)
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
    components: parsedComponents.value,
    dataModel: parsedDataModel.value,
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
}

function copyJson() {
  navigator.clipboard.writeText(widgetJson.value)
}

function downloadJson() {
  const blob = new Blob([widgetJson.value], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${widgetName.value.replace(/\s+/g, '-').toLowerCase()}.json`
  a.click()
  URL.revokeObjectURL(url)
}
</script>
