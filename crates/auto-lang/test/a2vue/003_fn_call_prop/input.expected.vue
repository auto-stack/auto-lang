<!-- FnCallProp component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, defineModel } from 'vue'
import ItemList from '@/ext/src/front/components/ItemList.vue'
import { getList } from '@/ext/src/front/helpers'


const msg = defineModel<string>("msg", { default: '' })


</script>

<template>
    <div class="flex flex-col gap-4">
      <ItemList :items="getList(msg)" :key="'ItemList-1'" />
    </div>

</template>

<style>
/* Component styles */

</style>
