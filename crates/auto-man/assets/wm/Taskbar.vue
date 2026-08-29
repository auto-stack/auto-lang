<script setup lang="ts">
// Plan 465 T6: taskbar DOM 叶 — 底栏 chrome：召唤 launcher 按钮 + 窗列
// （聚焦/关闭）+ 布局切换。结构 1:1 镜像 463 assets/shell.at 的 DesktopShell
// （对拍表见 docs/plans/reports/465-t1-host-blueprint.md §5）。
// iced 对应实现 = shell.at 经 DesktopBus 注入 __wm_wins/__wm_meta；vue 宿主
// 同进程直读 WmStore（无字符串总线，T1 蓝图登记差异）。
import { computed } from 'vue'
import { wm, close, focus, setLayout, cycleFocus, type LayoutModeName } from './store'

const emit = defineEmits<{ summon: [] }>()

const sorted = computed(() => [...wm.wins].sort((a, b) => b.z - a.z))

const layouts: { mode: LayoutModeName; icon: string; label: string }[] = [
  { mode: 'free', icon: '⊞', label: 'free' },
  { mode: 'grid', icon: '▦', label: 'grid' },
  { mode: 'master-stack', icon: '≣', label: 'master-stack' },
]

function onAltTab(): void {
  cycleFocus()
}

defineExpose({ onAltTab })
</script>

<template>
  <footer class="h-12 w-full flex items-center gap-1 px-2 bg-card border-t shrink-0">
    <button
      class="h-8 w-10 px-0 text-xs rounded border border-border hover:bg-accent"
      title="Summon launcher (Ctrl+Space)"
      @click="emit('summon')"
    >
      ⊞
    </button>
    <template v-for="w in sorted" :key="w.wid">
      <button
        class="h-8 px-3 text-xs rounded border max-w-48 truncate"
        :class="w.focused ? 'border-primary text-primary' : 'border-border'"
        :title="w.title"
        @click="focus(w.wid)"
      >
        {{ w.title }}
      </button>
      <button
        class="h-8 w-7 px-0 text-xs text-muted-foreground rounded border border-border hover:bg-accent"
        :aria-label="`close ${w.title}`"
        @click="close(w.wid)"
      >
        ×
      </button>
    </template>
    <span class="flex-1" />
    <button
      v-for="l in layouts"
      :key="l.mode"
      class="h-8 w-9 px-0 text-xs rounded border"
      :class="wm.layoutMode === l.mode ? 'border-primary text-primary' : 'border-border'"
      :title="`layout: ${l.label}`"
      @click="setLayout(l.mode)"
    >
      {{ l.icon }}
    </button>
    <button
      class="h-8 w-9 px-0 text-xs rounded border border-border hover:bg-accent"
      title="cycle focus (Alt+Tab)"
      @click="onAltTab"
    >
      ⇄
    </button>
  </footer>
</template>
