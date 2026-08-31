<!-- ThumbDemo component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref } from 'vue'
import { WindowThumbnail } from '@/wm/WindowThumbnail'


const wins = ref<any[]>([])


</script>

<template>
    <div class="flex flex-col w-full h-full">
      <WindowThumbnail class="w-24 h-14 border rounded" :fallback_icon="'app-window'" :wid="w.wid" :key="'WindowThumbnail-1-' + (((w as any)?.id ?? w))"  v-for="w in wins"/>
    </div>

</template>

<style>
/* Component styles */

</style>
