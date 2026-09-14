<!-- DesktopSurface component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Button } from '@/components/ui/button'

import { Circle } from 'lucide-vue-next'


const __desktop_cmd = ref<string>('')
const __desktop_bg = ref<string>('')
const __desktop_icons = ref<any[]>([])
const __desktop_hidden = ref<string>('')
const __desktop_cells = ref<any[]>([])
const __desktop_cell_ids = ref<any[]>([])
const __desktop_cell_cs = ref<any[]>([])
const __desktop_cell_rs = ref<any[]>([])
const menu_id = ref<string>('')
const drag_id = ref<string>('')
const blank_menu = ref<string>('')
const __desktop_cursor_x = ref<number>(0)
const __desktop_cursor_y = ref<number>(0)

const emit = defineEmits<{
  Init: []
  ActivateApp: [string]
  IconMenu: [string]
  IconPress: [string]
  BlankPress: []
  BlankDrop: []
  MenuOpen: []
  MenuRemove: []
  MenuWallpaper: []
  BlankMenu: []
}>()

function ActivateApp(app: any): void {
  __desktop_cmd.value = 'activate\t' + app;

  emit('ActivateApp', app)
}

function BlankClose(): void {
  blank_menu.value = '';
}

function BlankDrop(): void {
  if (drag_id.value != '') {__desktop_cmd.value = 'desktop_icon_drop_at\t' + drag_id.value;
  drag_id.value = '';
  }

  emit('BlankDrop')
}

function BlankMenu(): void {
  blank_menu.value = '1';

  emit('BlankMenu')
}

function BlankPress(): void {
  menu_id.value = '';
  blank_menu.value = '';

  emit('BlankPress')
}

function IconMenu(id: any): void {
  menu_id.value = id;

  emit('IconMenu', id)
}

function IconPress(id: any): void {
  if (drag_id.value == '') {drag_id.value = id;
  __desktop_cmd.value = 'desktop_icon_drag_start\t' + id;
  }

  emit('IconPress', id)
}

function MenuClose(): void {
  menu_id.value = '';
}

function MenuOpen(e: any): void {
  if (menu_id.value != '') {__desktop_cmd.value = 'activate\t' + menu_id.value;
  menu_id.value = '';
  }

  emit('MenuOpen')
}

function MenuRemove(e: any): void {
  if (menu_id.value != '') {if (__desktop_hidden.value == '') {__desktop_hidden.value = menu_id.value;
  } else {__desktop_hidden.value = __desktop_hidden.value + ',' + menu_id.value;
  }localStorage.setItem('shell.desktop.hidden', __desktop_hidden.value);
  menu_id.value = '';
  }

  emit('MenuRemove')
}

function MenuWallpaper(e: any): void {
  menu_id.value = '';
  __desktop_cmd.value = 'open_settings';

  emit('MenuWallpaper')
}

function MenuWallpaperBlank(): void {
  blank_menu.value = '';
  __desktop_cmd.value = 'open_settings';
}

function OpenSettingsBlank(): void {
  blank_menu.value = '';
  __desktop_cmd.value = 'open_settings';
}

onMounted(() => {
  menu_id.value = '';
  blank_menu.value = '';
})


</script>

<template>
    <div :class="'w-full h-full p-3' + __desktop_bg" class="flex flex-col w-full h-full p-3">
      <div class="w-full h-full" @click="BlankPress" @contextmenu.prevent="BlankMenu" @mouseup="BlankDrop">
        <div class="flex flex-col w-full h-full">
          <div class="grid grid-cols-8 gap-2 w-[1336px]">
            <div v-for="(e, __for_idx) in __desktop_cells" :key="__for_idx">
              <template v-if="e.spacer == '1'">
                <div class="w-40 h-40" />
              </template>
              <template v-else>
                <div @click="IconPress(e.id)" @dblclick="ActivateApp(e.id)">
                  <div :class="(drag_id == e.id ? 'w-40 h-40 items-center justify-center gap-1 bg-white/20 opacity-50' : 'w-40 h-40 items-center justify-center gap-1 hover:bg-white/10')" class="flex flex-col" @contextmenu.prevent="IconMenu(e.id)">
<div v-if="menu_id == e.id" class="fixed inset-0 z-40" @click="MenuClose"></div>
<div v-if="menu_id == e.id" class="fixed z-50 p-1 border rounded bg-card" :style="{ left: '8px', top: '8px' }">
                      <template v-if="e.full == '1'">
                        <div class="flex flex-col w-40 h-40 rounded-xl border-2 border-transparent hover:border-white/50">
                          <Circle class="w-full h-full" />
                        </div>
                      </template>
                      <template v-else>
                        <div :style="'h-20 w-20 items-center justify-center rounded-xl bg-[' + e.color + ']'" class="flex flex-col">
                          <Circle class="w-10 h-10 text-white" />
                        </div>
                      </template>
                      <div class="flex flex-col w-44 gap-1">
                        <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-foreground hover:bg-primary/10" @click="MenuOpen(e)" :key="'Button-1-' + (((e as any)?.id ?? e))">打开</Button>
                        <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-muted-foreground hover:bg-primary/10" @click="MenuRemove(e)" :key="'Button-2-' + (((e as any)?.id ?? e))">从桌面移除</Button>
                        <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-muted-foreground hover:bg-primary/10" @click="MenuWallpaper(e)" :key="'Button-3-' + (((e as any)?.id ?? e))">更换壁纸…</Button>
                      </div>
</div>
                    <span class="text-sm text-foreground truncate w-full text-center">{{ e.label }}</span>
                  </div>
                </div>
              </template>
            </div>
          </div>
        </div>
      </div>
<div v-if="blank_menu != ''" class="fixed inset-0 z-40" @click="BlankClose"></div>
<div v-if="blank_menu != ''" class="fixed z-50 p-1 border rounded bg-card" :style="{ left: __desktop_cursor_x + 'px', top: __desktop_cursor_y + 'px' }">
        <div class="flex flex-col w-44 gap-1">
          <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-foreground hover:bg-primary/10" @click="MenuWallpaperBlank" :key="'Button-4'">更换壁纸…</Button>
          <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-foreground hover:bg-primary/10" @click="OpenSettingsBlank" :key="'Button-5'">显示设置</Button>
        </div>
</div>
    </div>

</template>

<style>
/* Component styles */

</style>
