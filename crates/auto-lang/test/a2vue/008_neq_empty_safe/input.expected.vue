<!-- NeqEmptySafe component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, computed } from 'vue'

const x = ref<string>('')

const hasX = computed<boolean>(() => !!(x.value))
const emptyX = computed<boolean>(() => !(x.value))


</script>

<template>
    <div class="flex flex-col gap-4">
      <span class="text-muted-foreground leading-7">test</span>
    </div>

</template>

<style>
/* Component styles */

</style>
