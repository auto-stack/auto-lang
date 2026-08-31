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
  <!-- Plan 503：stella 风刷新——56px 底栏、rounded-xl 按钮格、激活窗
       accent-light 底 + 2px 竖条、hover accent-light（无 scale/无描边）。 -->
  <footer class="h-14 w-full flex items-center gap-2 px-2 bg-card border-t shrink-0">
    <button
      class="h-10 w-10 px-0 text-sm rounded-xl hover:bg-primary/10"
      title="Summon launcher (Ctrl+Space)"
      @click="emit('summon')"
    >
      ⊞
    </button>
    <template v-for="w in sorted" :key="w.wid">
      <span v-if="w.focused" class="w-0.5 h-5 rounded-full bg-primary" />
      <button
        class="h-9 px-3 text-xs rounded-xl max-w-48 truncate"
        :class="w.focused ? 'bg-primary/15 text-primary' : 'hover:bg-primary/10'"
        :title="w.title"
        @click="focus(w.wid)"
      >
        {{ w.title }}
      </button>
      <button
        class="h-8 w-7 px-0 text-xs text-muted-foreground rounded-xl hover:bg-primary/10"
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
      class="h-8 w-9 px-0 text-xs rounded-xl hover:bg-primary/10"
      :class="wm.layoutMode === l.mode ? 'text-primary bg-primary/15' : ''"
      :title="`layout: ${l.label}`"
      @click="setLayout(l.mode)"
    >
      {{ l.icon }}
    </button>
    <button
      class="h-8 w-9 px-0 text-xs rounded-xl hover:bg-primary/10"
      title="cycle focus (Alt+Tab)"
      @click="onAltTab"
    >
      ⇄
    </button>
  </footer>
</template>
