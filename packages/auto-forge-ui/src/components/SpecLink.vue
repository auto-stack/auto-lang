<template>
  <span class="spec-link" @click.stop="$emit('jump', id)">
    {{ id }}
  </span>
</template>

<script setup lang="ts">
defineProps<{
  id: string
}>()

defineEmits<{
  jump: [id: string]
}>()
</script>

<style scoped>
.spec-link {
  display: inline;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.85em;
  font-weight: 600;
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 0.08);
  padding: 0.05em 0.35em;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.12s;
}
.spec-link:hover {
  background: hsl(var(--primary) / 0.18);
}
</style>
