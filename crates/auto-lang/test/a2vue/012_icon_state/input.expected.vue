<!-- IconStateShowcase component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref } from 'vue'
import { Bell, Heart, Search, Zap } from 'lucide-vue-next'


const liked = ref<boolean>(false)


</script>

<template>
    <div class="flex flex-col gap-4">
      <Bell class="w-5 h-5 text-primary" :stroke-width="2.5" />
      <Bell class="w-5 h-5 text-muted-foreground" />
      <Bell class="text-red-500 w-5 h-5" :stroke-width="2.5" />
      <Heart class="w-5 h-5" :class="liked ? 'text-primary' : 'text-muted-foreground'" :stroke-width="liked ? 2.5 : undefined" />
      <Zap class="text-primary" style="width:48px;height:48px" :stroke-width="2" />
      <Search class="w-5 h-5" />
    </div>

</template>

<style>
/* Component styles */

</style>
