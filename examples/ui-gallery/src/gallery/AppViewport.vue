<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick, createApp, type App } from 'vue'
import { findDemo } from '@/demos-registry'

const props = defineProps<{
  app: string
  reloadKey?: number
  viewportMode?: string
}>()

const containerRef = ref<HTMLElement | null>(null)
const currentApp = ref<App | null>(null)
const isLoading = ref(false)
const errorMsg = ref<string | null>(null)

const activeDemo = computed(() => findDemo(props.app))

const viewportStyle = computed(() => {
  switch (props.viewportMode) {
    case 'desktop':
      return { width: '1024px', maxWidth: '100%' }
    case 'tablet':
      return { width: '768px', maxWidth: '100%' }
    case 'mobile':
      return { width: '375px', maxWidth: '100%' }
    case 'full':
    default:
      return { width: '100%' }
  }
})

async function mountApp() {
  errorMsg.value = null
  if (currentApp.value) {
    try {
      currentApp.value.unmount()
    } catch (e) {
      console.warn('[AppViewport] unmount error:', e)
    }
    currentApp.value = null
  }
  if (containerRef.value) {
    containerRef.value.innerHTML = ''
  }

  const demo = activeDemo.value
  if (!demo) {
    errorMsg.value = `未找到示例: ${props.app}`
    return
  }
  if (!demo.loadable || !demo.load) {
    return
  }

  isLoading.value = true
  try {
    const mod = await demo.load()
    if (!containerRef.value) return
    const app = createApp(mod.default)
    app.config.errorHandler = (err) => {
      console.error(`[AppViewport] Demo ${demo.id} error:`, err)
      errorMsg.value = `运行错误: ${err instanceof Error ? err.message : String(err)}`
    }
    app.mount(containerRef.value)
    currentApp.value = app
  } catch (err) {
    console.error(`[AppViewport] Failed to load ${demo.id}:`, err)
    errorMsg.value = `加载失败: ${err instanceof Error ? err.message : String(err)}`
  } finally {
    isLoading.value = false
  }
}

watch(() => [props.app, props.reloadKey], () => {
  void nextTick(() => mountApp())
})

onMounted(() => {
  void mountApp()
})

onBeforeUnmount(() => {
  if (currentApp.value) {
    try {
      currentApp.value.unmount()
    } catch {}
    currentApp.value = null
  }
})
</script>

<template>
  <div class="app-viewport-wrapper w-full h-full flex flex-col items-center justify-center relative bg-muted/20">
    <div
      class="app-viewport-frame relative transition-all duration-300 overflow-hidden flex flex-col bg-background rounded-lg border shadow-sm"
      :style="viewportStyle"
      :class="{
        'h-full': !viewportMode || viewportMode === 'full',
        'h-[560px] my-auto': viewportMode && viewportMode !== 'full'
      }"
    >
      <!-- Loading State -->
      <div v-if="isLoading" class="absolute inset-0 z-20 flex items-center justify-center bg-background/80 backdrop-blur-sm">
        <div class="flex items-center gap-2 text-sm text-muted-foreground">
          <svg class="animate-spin h-5 w-5 text-primary" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
          </svg>
          <span>加载应用中...</span>
        </div>
      </div>

      <!-- Error Banner -->
      <div v-if="errorMsg" class="absolute inset-x-4 top-4 z-30 p-3 rounded-md bg-destructive/10 border border-destructive/20 text-destructive text-sm flex items-center justify-between">
        <span>{{ errorMsg }}</span>
        <button class="text-xs underline ml-2" @click="mountApp">重试</button>
      </div>

      <!-- Non-loadable / backend notice -->
      <div v-if="activeDemo && !activeDemo.loadable" class="p-8 flex flex-col items-center justify-center text-center my-auto">
        <div class="h-12 w-12 rounded-full bg-primary/10 flex items-center justify-center mb-3 text-primary">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <h3 class="text-base font-semibold">{{ activeDemo.title }}</h3>
        <p class="text-sm text-muted-foreground mt-1 max-w-md">
          本示例为全栈或原生系统应用（需搭配专用后端服务或本地文件系统运行）。请在终端运行下方命令进行独立体验：
        </p>
        <code class="mt-3 px-3 py-1.5 rounded bg-muted text-xs font-mono font-bold select-all">
          cd examples/ui/{{ activeDemo.id }} && auto run
        </code>
        <p class="text-xs text-muted-foreground mt-4">
          下方教程区仍提供完整的应用架构、Elm 模型与源码说明。
        </p>
      </div>

      <!-- Real App Container -->
      <div v-show="activeDemo && activeDemo.loadable" ref="containerRef" class="demo-mount-root w-full h-full flex-1 overflow-auto relative"></div>
    </div>
  </div>
</template>
