<!-- Counter component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref } from 'vue'

const count = ref<number>(0)

const emit = defineEmits<{
  Inc: []
}>()

function Inc(): void {
  count.value = count.value + 1;

  emit('Inc')
}


</script>

<template>
    <div class="flex flex-col gap-4 p-4">
      <span class="text-muted-foreground leading-7">Count: {{ count }}</span>
      <button class="px-4 py-2 rounded" @click="Inc">+</button>
    </div>

</template>

<style>
/* Component styles */

</style>
