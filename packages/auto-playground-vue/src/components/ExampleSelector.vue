<template>
  <select class="example-selector" @change="onSelect" v-model="selected">
    <option value="">Load Example...</option>
    <optgroup label="Single-file">
      <option
        v-for="ex in singleExamples"
        :key="ex.name"
        :value="JSON.stringify(ex)"
      >
        {{ ex.name }}
      </option>
    </optgroup>
    <optgroup label="Projects">
      <option
        v-for="ex in projectExamples"
        :key="ex.name"
        :value="JSON.stringify(ex)"
      >
        {{ ex.name }}
      </option>
    </optgroup>
  </select>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import type { Example } from '../types';

const props = withDefaults(defineProps<{
  apiBase?: string
}>(), {
  apiBase: '/api'
})

const emit = defineEmits<{
  select: [payload: { source: string; project_dir?: string }];
}>();

const examples = ref<Example[]>([]);
const selected = ref('');

const singleExamples = computed(() =>
  examples.value.filter((ex) => ex.example_type === 'single')
);
const projectExamples = computed(() =>
  examples.value.filter((ex) => ex.example_type === 'project')
);

onMounted(async () => {
  try {
    const res = await fetch(`${props.apiBase}/examples`);
    const data = await res.json();
    examples.value = data.examples || [];
  } catch { /* ignore */ }
});

function onSelect() {
  if (!selected.value) return;
  try {
    const ex: Example = JSON.parse(selected.value);
    emit('select', {
      source: ex.source,
      project_dir: ex.project_dir,
    });
  } catch { /* ignore */ }
  selected.value = '';
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
