<!-- ShadcnColDynamic component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, defineModel, computed } from 'vue'

const isActive = defineModel<boolean>("isActive", { default: false })

const rowClass = computed<any>(() => (isActive.value ? 'row-active' : 'row-idle'))


</script>

<template>
    <div :class="rowClass" class="flex flex-col">
      <span>test</span>
    </div>

</template>

<style>
/* Component styles */

</style>
