<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vitepress'
const props = defineProps<{ target: string }>()
const router = useRouter()
onMounted(() => router.go(props.target))
</script>

<template><p><a :href="target">{{ target.startsWith('/zh/') ? '在应用概览中查看 →' : 'View in the applications overview →' }}</a></p></template>
