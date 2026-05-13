<template>
  <span class="spec-link" :class="{ unknown: !known }" @click="onClick">
    {{ id }}
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useSpecs } from '@/composables/useSpecs'

const props = defineProps<{
  id: string
}>()

const emit = defineEmits<{
  jump: [id: string]
}>()

const { findItemById } = useSpecs()

const known = computed(() => !!findItemById(props.id))

function onClick() {
  emit('jump', props.id)
}
</script>

<style scoped>
.spec-link {
  display: inline-flex;
  align-items: center;
  padding: 0.05rem 0.3rem;
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.8em;
  font-weight: 600;
  cursor: pointer;
  background: hsl(var(--primary) / 0.08);
  color: hsl(var(--primary));
  transition: background 0.15s;
}
.spec-link:hover {
  background: hsl(var(--primary) / 0.15);
}
.spec-link.unknown {
  opacity: 0.5;
  cursor: not-allowed;
  text-decoration: line-through;
}
</style>
