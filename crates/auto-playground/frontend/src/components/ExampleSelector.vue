<template>
  <select class="example-selector" @change="onSelect" v-model="selected">
    <option value="">Load Example...</option>
    <option v-for="ex in examples" :key="ex.name" :value="ex.source">
      {{ ex.name }}
    </option>
  </select>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { Example } from '../types';

const API_BASE = 'http://localhost:3030/api';

const emit = defineEmits<{
  select: [code: string];
}>();

const examples = ref<Example[]>([]);
const selected = ref('');

onMounted(async () => {
  try {
    const res = await fetch(`${API_BASE}/examples`);
    const data = await res.json();
    examples.value = data.examples || [];
  } catch { /* ignore */ }
});

function onSelect() {
  if (selected.value) {
    emit('select', selected.value);
    selected.value = '';
  }
}
</script>

<style scoped>
.example-selector {
  background: #2d2d2d;
  color: #ccc;
  border: 1px solid #555;
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 13px;
  cursor: pointer;
}
</style>
