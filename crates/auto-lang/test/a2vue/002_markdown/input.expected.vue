<!-- DocView component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref } from 'vue'
import { StreamingRenderer } from '@autodown/engine'


const body = ref<string>('')


</script>

<template>
    <div class="flex flex-col doc-wrap">
      <StreamingRenderer :source="body" :scroll-sync="true" class="doc-content" :key="'StreamingRenderer-1'" />
    </div>

</template>

<style>
/* Component styles */

</style>
