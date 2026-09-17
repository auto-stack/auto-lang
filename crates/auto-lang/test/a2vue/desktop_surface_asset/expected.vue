<!-- DesktopSurface component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Button } from '@/components/ui/button'

import { ChevronLeft, ChevronRight, Circle, Image, X } from 'lucide-vue-next'


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
const drag_icon = ref<string>('')
const drop_c = ref<string>('')
const drop_r = ref<string>('')
const drag_moved = ref<string>('')
const __desktop_label_dark = ref<string>('')
const blank_menu = ref<string>('')
const __desktop_cursor_x = ref<number>(0)
const __desktop_cursor_y = ref<number>(0)
const sel_id = ref<string>('')
const launching = ref<string>('')
const __wm_running = ref<string>('')
const __wp_picker = ref<string>('')
const __wp_preview = ref<string>('')
const __wp_dir = ref<string>('')
const __wp_current = ref<string>('')
const __wp_items = ref<any[]>([])
const __wp_visible = ref<any[]>([])
const __wp_x = ref<number>(0)
const __wp_y = ref<number>(0)
const wp_paths = ref<any[]>([])

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
  PickerApply: [string]
  PickerPreview: [string]
  PickerBack: []
  PickerNav: [string]
  PickerBrowse: []
  PickerDismiss: []
  ResetIconsBlank: []
  SendCmd: [string]
}>()

function ActivateApp(app: any): void {
  SendCmd('activate\t' + app);



  launching.value = app;

  emit('ActivateApp', app)
}

function BlankClose(): void {
  blank_menu.value = '';
}

function BlankDrop(): void {
  if (drag_id.value != '') {SendCmd('desktop_icon_drop_at\t' + drag_id.value);
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
  sel_id.value = '';

  emit('BlankPress')
}

function IconMenu(id: any): void {
  menu_id.value = id;

  emit('IconMenu', id)
}

function IconPress(id: any): void {
  sel_id.value = id;
  if (drag_id.value == '') {drag_id.value = id;
  SendCmd('desktop_icon_drag_start\t' + id);
  }

  emit('IconPress', id)
}

function MenuClose(): void {
  menu_id.value = '';
}

function MenuOpen(e: any): void {
  if (menu_id.value != '') {SendCmd('activate\t' + menu_id.value);
  menu_id.value = '';
  }

  emit('MenuOpen')
}

function MenuRemove(e: any): void {
  if (menu_id.value != '') {let hay: string = ',' + __desktop_hidden.value + ',';
  let needle: string = ',' + menu_id.value + ',';
  if (hay.includes(needle) == false) {if (__desktop_hidden.value == '') {__desktop_hidden.value = menu_id.value;
  } else {__desktop_hidden.value = __desktop_hidden.value + ',' + menu_id.value;
  }localStorage.setItem('shell.desktop.hidden', __desktop_hidden.value);
  }menu_id.value = '';
  }

  emit('MenuRemove')
}

function MenuWallpaper(e: any): void {
  menu_id.value = '';
  SendCmd('wallpaper_pick');

  emit('MenuWallpaper')
}

function MenuWallpaperBlank(): void {
  blank_menu.value = '';
  SendCmd('wallpaper_pick');
}

function OpenSettingsBlank(): void {
  blank_menu.value = '';
  SendCmd('open_settings');
}

function PickerApply(path: any): void {
  SendCmd('set_wallpaper\t' + path);

  emit('PickerApply', path)
}

function PickerBack(): void {
  SendCmd('wallpaper_preview\t');

  emit('PickerBack')
}

function PickerBrowse(): void {
  SendCmd('wallpaper_browse_dir');

  emit('PickerBrowse')
}

function PickerDismiss(): void {
  SendCmd('wallpaper_close');

  emit('PickerDismiss')
}

function PickerNav(dir: any): void {
  SendCmd('wallpaper_nav\t' + dir);

  emit('PickerNav', dir)
}

function PickerPreview(path: any): void {
  SendCmd('wallpaper_preview\t' + path);

  emit('PickerPreview', path)
}

function ResetIconsBlank(): void {
  blank_menu.value = '';
  __desktop_hidden.value = '';
  localStorage.setItem('shell.desktop.hidden', '');
  SendCmd('refresh_desktop_icons');

  emit('ResetIconsBlank')
}

function RunningSync(): void {
  if (launching.value != '') {if (__wm_running.value.includes(',' + launching.value + ',')) {launching.value = '';
  }}
}

function SendCmd(rec: any): void {
  if (__desktop_cmd.value != '') {__desktop_cmd.value = __desktop_cmd.value + '\n';
  }
  __desktop_cmd.value = __desktop_cmd.value + rec;
}

onMounted(() => {
  menu_id.value = '';
  blank_menu.value = '';


  if (launching.value != '') {if (__wm_running.value.includes(',' + launching.value + ',') == false) {launching.value = '';
  }}
})


</script>

<template>
    <div :class="'w-full h-full p-3' + __desktop_bg" class="flex flex-col w-full h-full p-3">
      <div class="w-full h-full" @click="BlankPress" @contextmenu.prevent="BlankMenu" @mouseup="BlankDrop">
        <div class="flex flex-col w-full h-full">
          <div class="grid grid-cols-8 gap-2 w-[696px]">
            <div v-for="(e, __for_idx) in __desktop_cells" :key="__for_idx">
              <template v-if="e.spacer == '1'">
                <div class="w-20 h-[72px]" />
              </template>
              <template v-else>
                <div @click="IconPress(e.id)" @dblclick="ActivateApp(e.id)">
                  <div :class="(drag_id == e.id ? 'w-20 h-[72px] items-center justify-center gap-1 bg-white/20 opacity-50' : ((drag_id != '' && drag_id != e.id && e.c == drop_c && e.r == drop_r ? 'w-20 h-[72px] items-center justify-center gap-1 bg-primary/20' : ((launching == e.id ? 'w-20 h-[72px] items-center justify-center gap-1 rounded-lg bg-white/10 opacity-50' : ((sel_id == e.id ? 'w-20 h-[72px] items-center justify-center gap-1 rounded-lg bg-white/10' : 'w-20 h-[72px] items-center justify-center gap-1 hover:bg-white/10')))))))" class="flex flex-col" @contextmenu.prevent="IconMenu(e.id)">
<div v-if="menu_id == e.id" class="fixed inset-0 z-40" @click="MenuClose"></div>
<div v-if="menu_id == e.id" class="fixed z-50 p-1 border rounded bg-card" :style="{ left: '8px', top: '8px' }">
                      <template v-if="e.full == '1'">
                        <div class="flex flex-col w-12 h-12">
                          <Circle class="w-full h-full" />
                        </div>
                      </template>
                      <template v-else>
                        <div :style="'h-10 w-10 items-center justify-center rounded-xl bg-[' + e.color + ']'" class="flex flex-col">
                          <Circle class="w-5 h-5 text-white" />
                        </div>
                      </template>
                      <div class="flex flex-col w-44 gap-1">
                        <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-foreground hover:bg-primary/10" @click="MenuOpen(e)" :key="'Button-1-' + (((e as any)?.id ?? e))">打开</Button>
                        <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-muted-foreground hover:bg-primary/10" @click="MenuRemove(e)" :key="'Button-2-' + (((e as any)?.id ?? e))">从桌面移除</Button>
                        <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-muted-foreground hover:bg-primary/10" @click="MenuWallpaper(e)" :key="'Button-3-' + (((e as any)?.id ?? e))">更换壁纸…</Button>
                      </div>
</div>
                    <span :class="(__desktop_bg == '' ? 'text-xs text-white truncate w-full text-center rounded-md bg-black/30' : ((__desktop_label_dark == '1' ? 'text-xs text-white truncate w-full text-center' : 'text-xs text-foreground truncate w-full text-center')))">{{ e.label }}</span>
                    <template v-if="launching == e.id">
                      <div class="absolute top-0.5 right-1 w-1.5 h-1.5 rounded-full bg-muted-foreground" />
                    </template>
                  </div>
                </div>
              </template>
            </div>
          </div>
<div v-if="drag_moved == '1'" class="fixed z-50 p-0 bg-transparent" :style="{ left: __desktop_cursor_x + 'px', top: __desktop_cursor_y + 'px' }">
            <div class="flex flex-col w-12 h-12 opacity-60">
              <Circle class="w-full h-full" />
            </div>
</div>
        </div>
      </div>
<div v-if="blank_menu != ''" class="fixed inset-0 z-40" @click="BlankClose"></div>
<div v-if="blank_menu != ''" class="fixed z-50 p-1 border rounded bg-card" :style="{ left: __desktop_cursor_x + 'px', top: __desktop_cursor_y + 'px' }">
        <div class="flex flex-col w-44 gap-1">
          <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-foreground hover:bg-primary/10" @click="MenuWallpaperBlank" :key="'Button-4'">更换壁纸…</Button>
          <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-foreground hover:bg-primary/10" @click="OpenSettingsBlank" :key="'Button-5'">显示设置</Button>
          <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-muted-foreground hover:bg-primary/10" @click="ResetIconsBlank" :key="'Button-6'">恢复默认图标</Button>
        </div>
</div>
<div v-if="__wp_picker == '1'" class="fixed inset-0 z-40" @click="PickerDismiss"></div>
<div v-if="__wp_picker == '1'" class="fixed z-50 p-4 border rounded-xl bg-card w-[720px] gap-3" :style="{ left: __wp_x + 'px', top: __wp_y + 'px' }">
        <div class="flex flex-row w-full items-center gap-2">
          <Image class="w-4 h-4 text-muted-foreground" />
          <span class="text-xs text-muted-foreground flex-1 truncate">{{ __wp_dir }}</span>
          <Button variant="ghost" class="h-7 px-3 text-xs rounded-lg bg-muted text-muted-foreground hover:bg-primary/10" @click="PickerBrowse" :key="'Button-7'">浏览…</Button>
          <Button variant="ghost" class="h-7 w-7 px-0 rounded-lg text-muted-foreground hover:bg-primary/10" @click="PickerDismiss" :key="'Button-8'">
            <X class="h-4 w-4" />          </Button>
        </div>
        <template v-if="__wp_preview == ''">
          <div class="flex flex-row w-full items-center gap-2">
            <Button variant="ghost" class="h-8 w-8 px-0 rounded-lg hover:bg-primary/10" @click="PickerNav('prev')" :key="'Button-9'">
              <ChevronLeft class="h-4 w-4" />            </Button>
            <div class="flex flex-col gap-1" v-for="e in __wp_visible" :key="(((e as any)?.id ?? e))">
              <div @click="PickerApply(e.path)">
                <div :class="(e.path == __wp_current ? 'w-[120px] h-[68px] rounded-lg border-2 border-primary' : 'w-[120px] h-[68px] rounded-lg border-2 border-transparent')" class="flex flex-col">
                  <img :src="e.path" :alt="e.name" class="w-full h-full rounded-lg object-cover" />
                </div>
              </div>
              <div class="flex flex-row w-[120px] items-center justify-between">
                <span class="text-[10px] text-muted-foreground truncate">{{ e.name }}</span>
                <Button variant="ghost" class="h-5 px-1 text-[10px] rounded-md bg-transparent text-muted-foreground hover:bg-primary/10" @click="PickerPreview(e.path)" :key="'Button-10-' + (((e as any)?.id ?? e))">预览</Button>
              </div>
            </div>
            <Button variant="ghost" class="h-8 w-8 px-0 rounded-lg hover:bg-primary/10" @click="PickerNav('next')" :key="'Button-11'">
              <ChevronRight class="h-4 w-4" />            </Button>
          </div>
        </template>
        <template v-else>
          <div class="flex flex-col w-full gap-2 items-center">
            <img :src="__wp_preview" alt="preview" class="w-full h-[440px] rounded-lg bg-black/40 object-contain" />
            <div class="flex flex-row items-center gap-2">
              <Button variant="ghost" class="h-8 w-8 px-0 rounded-lg hover:bg-primary/10" @click="PickerNav('prev')" :key="'Button-12'">
                <ChevronLeft class="h-4 w-4" />              </Button>
              <Button variant="ghost" class="h-7 px-3 text-xs rounded-lg bg-muted text-muted-foreground hover:bg-primary/10" @click="PickerBack" :key="'Button-13'">返回</Button>
              <Button variant="ghost" class="h-8 w-8 px-0 rounded-lg hover:bg-primary/10" @click="PickerNav('next')" :key="'Button-14'">
                <ChevronRight class="h-4 w-4" />              </Button>
            </div>
          </div>
        </template>
</div>
    </div>

</template>

<style>
/* Component styles */

</style>
