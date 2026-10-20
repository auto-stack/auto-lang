<!-- IconChild component - Auto-generated from Auto language -->
<script setup lang="ts">
import { Plus } from 'lucide-vue-next'


const emit = defineEmits<{
  Add: []
}>()

function Add(): void {

  emit('Add')
}


</script>

<template>
    <div class="flex flex-col gap-4">
      <button class="sidebar-btn" @click="Add">
        <Plus :size="14" :key="'Plus-1'" />
        <span class="text-muted-foreground leading-7">新建</span>
      </button>
    </div>

</template>

<style>
/* Component styles */

</style>
