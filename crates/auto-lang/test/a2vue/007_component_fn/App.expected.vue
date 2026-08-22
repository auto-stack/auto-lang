<!-- App component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, defineModel } from 'vue'
import Card from '@/components/Card.vue'


const heading = defineModel<string>("heading", { default: 'demo' })


</script>

<template>
    <Card :title="heading" :key="'Card-1'" />

</template>

<style>
/* Component styles */

</style>
