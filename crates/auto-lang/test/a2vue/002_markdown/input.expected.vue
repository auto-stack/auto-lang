<!-- DocView component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, defineModel } from 'vue'
import { MarkdownRender } from 'markstream-vue'


const body = defineModel<string>("body", { default: '' })


</script>

<template>
    <div class="flex flex-col doc-wrap">
      <MarkdownRender :content="body" class="doc-content" :key="'MarkdownRender-1'" />
    </div>

</template>

<style>
/* Component styles */

</style>
