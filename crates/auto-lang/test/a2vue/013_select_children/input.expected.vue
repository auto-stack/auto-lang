<!-- SelectChildren component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref } from 'vue'

const current = ref<string>('zhipu')
const providers = ref<any[]>([])

const emit = defineEmits<{
  Pick: []
}>()

function Pick(): void {

  emit('Pick')
}


</script>

<template>
    <div class="flex flex-col gap-4">
      <select class="border rounded px-2 py-1" :value="current" @change="Pick">
        <option :value="'zhipu'">Zhipu</option>
        <option :value="'deepseek'">DeepSeek</option>
      </select>
      <select class="border rounded px-2 py-1" :value="current" @change="Pick">
        <option :value="o.value" v-for="o in providers" :key="(((o as any)?.id ?? o))">{{ o.label }}</option>
      </select>
    </div>

</template>

<style>
/* Component styles */

</style>
