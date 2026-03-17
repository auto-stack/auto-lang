<!-- CounterApp component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref } from 'vue'
import { Button } from '@/components/ui/button'

const count = ref<number>(0)


</script>

<template>
  <div class="flex flex-col">
    <div class="flex flex-col gap-4 min-h-screen bg-gray-100 flex items-center justify-center">
      <div class="flex flex-col gap-4 bg-white p-8 rounded-lg shadow-lg">
        <span class="text-muted-foreground leading-7 text-2xl font-bold mb-4">Counter Demo</span>
        <span class="text-muted-foreground leading-7 text-3xl font-mono mb-4">Count: {count}</span>
        <div class="flex flex-row gap-4 flex items-center gap-4">
          <Button>-</Button>
          <Button>+</Button>
        </div>
      </div>
    </div>
  </div>

</template>

<style scoped>
/* Component styles */

</style>


