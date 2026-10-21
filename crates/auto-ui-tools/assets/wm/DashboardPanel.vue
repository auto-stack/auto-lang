<script setup lang="ts">
// PLAN-024：dashboard 面板 DOM 叶（vue 宿主）——vm 轨 dashboard.at 的
// 对拍面（第四 overlay 槽）：顶部居中浮层 + 3 列网格直显各 App `view mini`
// 活渲染面。活渲染语义 = Mini.vue 与 App.vue 共享同一 store 模块单例
// （生成器 store composable 的 module-scope ref 同源），App 窗开着时卡内
// 数值与主窗同帧一致；scrim/Esc 关闭（PLAN-012 W2 同型）。
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { Component } from 'vue'
import { APPS } from '../apps-registry'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

interface MiniEntry {
  id: string
  title: string
  comp: Component
}

const minis = ref<MiniEntry[]>([])
const loaded = ref(false)

watch(
  () => props.open,
  async (open) => {
    if (open && !loaded.value) {
      loaded.value = true
      for (const a of APPS) {
        if (!a.mini || !a.loadMini) continue
        try {
          const mod = await a.loadMini()
          minis.value.push({ id: a.id, title: a.title, comp: mod.default })
        } catch (err) {
          console.error(`[dashboard] Mini load failed: ${a.id}`, err)
        }
      }
    }
  },
  { immediate: true },
)

function onKey(e: KeyboardEvent): void {
  if (e.key === 'Escape' && props.open) {
    emit('close')
  }
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <!-- 打开态：scrim（外点关闭）+ 顶部居中面板；关闭态不渲染（与 vm 轨
       "仅 visible 推层" 同语义——装配层门控）。 -->
  <div
    v-if="open"
    class="absolute inset-0 z-50"
    data-testid="dashboard-panel"
  >
    <div class="absolute inset-0" @click="emit('close')" />
    <div class="absolute inset-x-0 top-16 flex justify-center pointer-events-none">
      <div
        class="pointer-events-auto rounded-xl border bg-card/80 shadow-xl overflow-hidden backdrop-blur"
        style="width: min(920px, calc(100vw - 32px))"
      >
        <div class="flex items-center justify-between px-4 pt-3">
          <span class="text-sm font-medium text-foreground">Dashboard</span>
          <button
            class="h-7 w-7 rounded-lg text-muted-foreground hover:bg-primary/10"
            aria-label="close dashboard"
            data-testid="dashboard-close"
            @click="emit('close')"
          >
            ×
          </button>
        </div>
        <div class="grid grid-cols-3 gap-3 p-4">
          <div
            v-for="m in minis"
            :key="m.id"
            class="rounded-xl border h-[132px] overflow-hidden"
            :data-testid="`dashboard-card-${m.id}`"
            :title="m.title"
          >
            <component :is="m.comp" />
          </div>
          <div
            v-if="minis.length === 0"
            class="col-span-3 py-8 text-center text-sm text-muted-foreground"
          >
            No widgets
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
