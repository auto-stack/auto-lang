<!-- NestedIfStyle component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, defineModel } from 'vue'

const sid = defineModel<string>("sid", { default: '' })


</script>

<template>
    <div class="flex flex-col gap-4">
      <span :class="(sid != '' ? ((sid == 'a' ? 'item active' : 'item')) : 'item')">test</span>
    </div>

</template>

<style>
/* Component styles */

</style>
