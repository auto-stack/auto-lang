<template>
  <div class="tag-input">
    <div class="tag-list">
      <span v-for="(tag, idx) in modelValue" :key="idx" class="tag-chip">
        {{ tag }}
        <button class="tag-remove" @click="removeTag(idx)">
          <X :size="10" />
        </button>
      </span>
    </div>
    <input
      v-model="inputValue"
      class="tag-entry"
      :placeholder="placeholder"
      @keydown.enter.prevent="addTag"
      @keydown.backspace="onBackspace"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { X } from 'lucide-vue-next'

const props = defineProps<{
  modelValue: string[]
  placeholder?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string[]]
}>()

const inputValue = ref('')

function addTag() {
  const val = inputValue.value.trim()
  if (val && !props.modelValue.includes(val)) {
    emit('update:modelValue', [...props.modelValue, val])
  }
  inputValue.value = ''
}

function removeTag(idx: number) {
  const next = [...props.modelValue]
  next.splice(idx, 1)
  emit('update:modelValue', next)
}

function onBackspace() {
  if (inputValue.value === '' && props.modelValue.length > 0) {
    removeTag(props.modelValue.length - 1)
  }
}
</script>

<style scoped>
.tag-input {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.35rem;
  padding: 0.3rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--af-border);
  background: hsl(var(--background));
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
}

.tag-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.15rem;
  padding: 0.15rem 0.4rem;
  font-size: 0.75rem;
  border-radius: 4px;
  background: hsl(var(--primary) / 0.08);
  color: hsl(var(--primary));
}

.tag-remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 14px;
  height: 14px;
  border-radius: 3px;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  padding: 0;
}

.tag-remove:hover {
  background: hsl(var(--primary) / 0.15);
}

.tag-entry {
  flex: 1;
  min-width: 80px;
  border: none;
  background: transparent;
  color: var(--af-fg);
  font-size: 0.82rem;
  outline: none;
  padding: 0.15rem 0;
}
</style>
