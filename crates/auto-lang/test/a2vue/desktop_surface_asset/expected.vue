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
const blank_menu = ref<string>('')
const __desktop_cursor_x = ref<number>(0)
const __desktop_cursor_y = ref<number>(0)
const __wp_picker = ref<string>('')
const __wp_preview = ref<string>('')
const __wp_dir = ref<string>('')
const __wp_current = ref<string>('')
const __wp_items = ref<any[]>([])
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
  __desktop_cmd.value = 'wallpaper_pick';

  emit('MenuWallpaper')
}

function MenuWallpaperBlank(): void {
  blank_menu.value = '';
  __desktop_cmd.value = 'wallpaper_pick';
}

function OpenSettingsBlank(): void {
  blank_menu.value = '';
  __desktop_cmd.value = 'open_settings';
}

function PickerApply(path: any): void {
  __desktop_cmd.value = 'set_wallpaper\t' + path;

  emit('PickerApply', path)
}

function PickerBack(): void {
  __desktop_cmd.value = 'wallpaper_preview\t';

  emit('PickerBack')
}

function PickerBrowse(): void {
  __desktop_cmd.value = 'wallpaper_browse_dir';

  emit('PickerBrowse')
}

function PickerDismiss(): void {
  __desktop_cmd.value = 'wallpaper_close';

  emit('PickerDismiss')
}

function PickerNav(dir: any): void {
  __desktop_cmd.value = 'wallpaper_nav\t' + dir;

  emit('PickerNav', dir)
}

function PickerPreview(path: any): void {
  __desktop_cmd.value = 'wallpaper_preview\t' + path;

  emit('PickerPreview', path)
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
          <div class="grid grid-cols-8 gap-2 w-[696px]">
            <div v-for="(e, __for_idx) in __desktop_cells" :key="__for_idx">
              <template v-if="e.spacer == '1'">
                <div class="w-20 h-20" />
              </template>
              <template v-else>
                <div @click="IconPress(e.id)" @dblclick="ActivateApp(e.id)">
                  <div :class="(drag_id == e.id ? 'w-20 h-20 items-center justify-center gap-1 bg-white/20 opacity-50' : 'w-20 h-20 items-center justify-center gap-1 hover:bg-white/10')" class="flex flex-col" @contextmenu.prevent="IconMenu(e.id)">
<div v-if="menu_id == e.id" class="fixed inset-0 z-40" @click="MenuClose"></div>
<div v-if="menu_id == e.id" class="fixed z-50 p-1 border rounded bg-card" :style="{ left: '8px', top: '8px' }">
                      <div :style="'h-10 w-10 items-center justify-center rounded-xl bg-[' + e.color + ']'" class="flex flex-col">
                        <Circle class="w-5 h-5 text-white" />
                      </div>
                      <div class="flex flex-col w-44 gap-1">
                        <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-foreground hover:bg-primary/10" @click="MenuOpen(e)" :key="'Button-1-' + (((e as any)?.id ?? e))">打开</Button>
                        <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-muted-foreground hover:bg-primary/10" @click="MenuRemove(e)" :key="'Button-2-' + (((e as any)?.id ?? e))">从桌面移除</Button>
                        <Button variant="ghost" class="w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-muted-foreground hover:bg-primary/10" @click="MenuWallpaper(e)" :key="'Button-3-' + (((e as any)?.id ?? e))">更换壁纸…</Button>
                      </div>
</div>
                    <span class="text-xs text-foreground truncate w-full text-center">{{ e.label }}</span>
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
<div v-if="__wp_picker == '1'" class="fixed inset-0 z-40" @click="PickerDismiss"></div>
<div v-if="__wp_picker == '1'" class="fixed z-50 p-4 border rounded-xl bg-card w-[880px] gap-3" :style="{ left: 40 + 'px', top: 80 + 'px' }">
        <div class="flex flex-row w-full items-center gap-2">
          <Image class="w-4 h-4 text-muted-foreground" />
          <span class="text-xs text-muted-foreground flex-1 truncate">{{ __wp_dir }}</span>
          <Button variant="ghost" class="h-7 px-3 text-xs rounded-lg bg-muted text-muted-foreground hover:bg-primary/10" @click="PickerBrowse" :key="'Button-6'">浏览…</Button>
          <Button variant="ghost" class="h-7 w-7 px-0 rounded-lg text-muted-foreground hover:bg-primary/10" @click="PickerDismiss" :key="'Button-7'">
            <X class="h-4 w-4" />          </Button>
        </div>
        <template v-if="__wp_preview == ''">
          <div class="grid grid-cols-4 gap-3">
            <div class="flex flex-col gap-1" v-for="e in __wp_items" :key="(((e as any)?.id ?? e))">
              <div @click="PickerApply(e.path)">
                <div :class="(e.path == __wp_current ? 'w-[196px] h-[110px] rounded-lg border-2 border-primary' : 'w-[196px] h-[110px] rounded-lg border-2 border-transparent')" class="flex flex-col">
                  <img :src="e.path" :alt="e.name" class="w-full h-full rounded-lg object-cover" />
                </div>
              </div>
              <div class="flex flex-row w-full items-center gap-1">
                <span class="text-[10px] text-muted-foreground flex-1 truncate">{{ e.name }}</span>
                <Button variant="ghost" class="h-5 px-2 text-[10px] rounded-md bg-transparent text-muted-foreground hover:bg-primary/10" @click="PickerPreview(e.path)" :key="'Button-8-' + (((e as any)?.id ?? e))">预览</Button>
              </div>
            </div>
          </div>
        </template>
        <template v-else>
          <div class="flex flex-col w-full gap-2 items-center">
            <img :src="__wp_preview" alt="preview" class="w-full h-[440px] rounded-lg bg-black/40 object-contain" />
            <div class="flex flex-row items-center gap-2">
              <Button variant="ghost" class="h-8 w-8 px-0 rounded-lg hover:bg-primary/10" @click="PickerNav('prev')" :key="'Button-9'">
                <ChevronLeft class="h-4 w-4" />              </Button>
              <Button variant="ghost" class="h-7 px-3 text-xs rounded-lg bg-muted text-muted-foreground hover:bg-primary/10" @click="PickerBack" :key="'Button-10'">返回</Button>
              <Button variant="ghost" class="h-8 w-8 px-0 rounded-lg hover:bg-primary/10" @click="PickerNav('next')" :key="'Button-11'">
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
