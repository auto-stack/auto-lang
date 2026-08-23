<!-- Hand-written Vue component consumed by `use { component: ... }`.
     Instantiated from the DSL view as `FancyBadge { label: .message, onselected: .BadgePicked }`. -->
<script setup lang="ts">
defineProps<{ label: string }>()
const emit = defineEmits<{ selected: [] }>()
</script>

<template>
  <button class="fancy-badge" @click="emit('selected')">{{ label }}</button>
</template>

<style scoped>
.fancy-badge {
  padding: 2px 10px;
  border-radius: 9999px;
  background: #eef2ff;
  color: #3730a3;
  border: 1px solid #c7d2fe;
  cursor: pointer;
  font-size: 13px;
}
</style>
