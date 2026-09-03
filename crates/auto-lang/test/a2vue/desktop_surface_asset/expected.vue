<!-- DesktopSurface component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Button } from '@/components/ui/button'


const __desktop_cmd = ref<string>('')
const __desktop_bg = ref<string>('')
const __desktop_icons = ref<any[]>([])
const __desktop_hidden = ref<string>('')
const menu_id = ref<string>('')
const blank_menu = ref<string>('')

const emit = defineEmits<{
  Init: []
  ActivateApp: [string]
  IconMenu: [string]
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

function MenuOpen(): void {
  if (menu_id.value != '') {__desktop_cmd.value = 'activate\t' + menu_id.value;
  menu_id.value = '';
  }

  emit('MenuOpen')
}

function MenuRemove(): void {
  if (menu_id.value != '') {if (__desktop_hidden.value == '') {__desktop_hidden.value = menu_id.value;
  } else {__desktop_hidden.value = __desktop_hidden.value + ',' + menu_id.value;
  }localStorage.setItem('shell.desktop.hidden', __desktop_hidden.value);
  menu_id.value = '';
  }

  emit('MenuRemove')
}

function MenuWallpaper(): void {
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
        <template v-if="blank_menu != ''">
          <div class="flex flex-col w-44 gap-1 p-1 bg-card/80 border rounded-xl shadow-xl">
            <Button class="w-full h-8 px-0 text-xs justify-start text-muted-foreground rounded-lg hover:bg-primary/10" @click="MenuWallpaperBlank" :key="'Button-1'">更换壁纸…</Button>
            <Button class="w-full h-8 px-0 text-xs justify-start text-muted-foreground rounded-lg hover:bg-primary/10" @click="OpenSettingsBlank" :key="'Button-2'">显示设置</Button>
          </div>
        </template>
        <template v-if="menu_id != ''">
          <div class="flex flex-col w-44 gap-1 p-1 bg-card/80 border rounded-xl shadow-xl">
            <Button class="w-full h-8 px-0 text-xs justify-start rounded-lg hover:bg-primary/10" @click="MenuOpen" :key="'Button-3'">打开</Button>
            <Button class="w-full h-8 px-0 text-xs justify-start text-muted-foreground rounded-lg hover:bg-primary/10" @click="MenuRemove" :key="'Button-4'">从桌面移除</Button>
            <Button class="w-full h-8 px-0 text-xs justify-start text-muted-foreground rounded-lg hover:bg-primary/10" @click="MenuWallpaper" :key="'Button-5'">更换壁纸…</Button>
          </div>
        </template>
        <div class="grid grid-cols-8 gap-2 w-full">
          <div @dblclick="ActivateApp(e.id)" v-for="e in __desktop_icons" :key="(((e as any)?.id ?? e))">
            <div class="flex flex-col w-20 h-20 items-center justify-center gap-1">
              <Button :style="'h-10 w-10 px-0 text-xl text-white rounded-xl bg-[' + e.color + ']'" @contextmenu.prevent="IconMenu(e.id)" :key="'Button-6-' + (((e as any)?.id ?? e))" />
              <span class="text-xs text-foreground truncate w-full text-center">{{ e.label }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

</template>

<style>
/* Component styles */

</style>
