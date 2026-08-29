<!-- DesktopDemo component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
import { Taskbar } from '@/wm/Taskbar'
import { VirtualWindow } from '@/wm/VirtualWindow'


const wins = ref<any[]>([])

const emit = defineEmits<{
  __evt_onclick_1: []
  __evt_onclick_2: []
}>()

function __evt_onclick_1(w: any): void {
  // TODO
}

function __evt_onclick_2(): void {
  // TODO
}


</script>

<template>
    <div class="flex flex-col w-full h-full">
      <VirtualWindow :key="'VirtualWindow-1-' + (((w as any)?.id ?? w))" v-for="w in wins">
        <Button @click="__evt_onclick_1(w)" :key="'Button-2-' + (((w as any)?.id ?? w))">client</Button>
      </VirtualWindow>
      <Taskbar :key="'Taskbar-3'">
        <Button @click="__evt_onclick_2" :key="'Button-4'">⊞</Button>
      </Taskbar>
    </div>

</template>

<style>
/* Component styles */

</style>
