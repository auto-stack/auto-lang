<!-- DesktopSurface component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Button } from '@/components/ui/button'
import { Popover } from '@/components/ui/popover'

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
  } else {if (drag_id.value == id) {drag_id.value = '';
  } else {let di: number = 0 - 1;
  let ti: number = 0 - 1;
  let i: number = 0;
  while (i < __desktop_cell_ids.value.length) {if (__desktop_cell_ids.value[i] == drag_id.value) {di = i;
  }if (__desktop_cell_ids.value[i] == id) {ti = i;
  }i = i + 1;
  }
  if (di >= 0) {if (ti >= 0) {let moves: string = '';
  moves = moves + drag_id.value + '=' + __desktop_cell_cs.value[di] + ':' + __desktop_cell_rs.value[di] + ',';
  moves = moves + id + '=' + __desktop_cell_cs.value[ti] + ':' + __desktop_cell_rs.value[ti] + ',';
  let cur: string = (localStorage.getItem('shell.desktop.positions') ?? '');
  localStorage.setItem('shell.desktop.positions', cur + moves);
  __desktop_cmd.value = 'refresh_desktop_icons';
  }drag_id.value = '';
  }}}

  emit('IconPress', id)
}

function MenuClose(e: any): void {
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
      <div class="w-full h-full" @click="BlankPress" @contextmenu.prevent="BlankMenu">
        <div class="flex flex-col w-full h-full">
          <div class="grid grid-cols-8 gap-2 w-[696px]">
            <div v-for="(e, __for_idx) in __desktop_cells" :key="__for_idx">
              <template v-if="e.spacer == '1'">
                <div class="w-20 h-20" />
              </template>
              <template v-else>
                <div @click="IconPress(e.id)" @dblclick="ActivateApp(e.id)">
                  <div :class="(drag_id == e.id ? 'w-20 h-20 items-center justify-center gap-1 bg-white/20 opacity-50' : 'w-20 h-20 items-center justify-center gap-1 hover:bg-white/10')" class="flex flex-col" @contextmenu.prevent="IconMenu(e.id)">
                    <Popover class="p-1 border rounded bg-card" @dismiss="MenuClose(e)" :key="'Popover-1-' + (((e as any)?.id ?? e))">
                      <div :style="'h-10 w-10 items-center justify-center rounded-xl bg-[' + e.color + ']'" class="flex flex-col">
                        <Circle class="w-5 h-5 w-5 h-5 text-white" />
                      </div>
                      <div class="flex flex-col w-44 gap-1">
                        <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-foreground hover:bg-primary/10" @click="MenuOpen(e)" :key="'Button-2-' + (((e as any)?.id ?? e))">打开</Button>
                        <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-muted-foreground hover:bg-primary/10" @click="MenuRemove(e)" :key="'Button-3-' + (((e as any)?.id ?? e))">从桌面移除</Button>
                        <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-muted-foreground hover:bg-primary/10" @click="MenuWallpaper(e)" :key="'Button-4-' + (((e as any)?.id ?? e))">更换壁纸…</Button>
                      </div>
                    </Popover>
                    <span class="text-xs text-foreground truncate w-full text-center">{{ e.label }}</span>
                  </div>
                </div>
              </template>
            </div>
          </div>
        </div>
      </div>
      <Popover class="p-1 border rounded bg-card" @dismiss="BlankClose" :key="'Popover-5'">
        <div class="flex flex-col w-44 gap-1">
          <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-foreground hover:bg-primary/10" @click="MenuWallpaperBlank" :key="'Button-6'">更换壁纸…</Button>
          <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-foreground hover:bg-primary/10" @click="OpenSettingsBlank" :key="'Button-7'">显示设置</Button>
        </div>
      </Popover>
    </div>

</template>

<style>
/* Component styles */

</style>
