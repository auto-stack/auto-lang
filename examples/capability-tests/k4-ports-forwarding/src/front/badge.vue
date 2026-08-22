<!-- badge.vue — k4 hand-written local component (default export).
     Forwarded to callers through the port module, never imported directly. -->
<script setup lang="ts">
defineProps<{ text: string }>()
defineEmits<{ select: [string] }>()
</script>

<template>
  <button
    class="rounded-md border px-3 py-1 text-sm"
    @click="$emit('select', text)"
  >
    {{ text }}
  </button>
</template>
