<template>
  <div class="variable-inspector">
    <div v-if="variables.length === 0" class="empty-state">
      <Database :size="24" />
      <p>No variables yet</p>
      <span>Run a cell to see variables</span>
    </div>
    <div v-else class="variable-list">
      <div
        v-for="v in variables"
        :key="v.name"
        class="variable-item"
        :class="v.kind"
      >
        <component :is="kindIcon(v.kind)" :size="14" class="var-icon" />
        <span class="var-name">{{ v.name }}</span>
        <span class="var-kind">{{ v.kind }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Database, Box, FunctionSquare } from 'lucide-vue-next'
import type { VariableInfo } from '@/types/cell'

defineProps<{
  variables: VariableInfo[]
}>()

function kindIcon(kind: string) {
  switch (kind) {
    case 'function': return FunctionSquare
    default: return Box
  }
}
</script>

<style scoped>
.variable-inspector {
  height: 100%;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem 1rem;
  color: #6c7086;
  text-align: center;
  gap: 0.5rem;
}

.empty-state p {
  margin: 0;
  font-size: 0.9rem;
  color: #cdd6f4;
}

.empty-state span {
  font-size: 0.8rem;
}

.variable-list {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.variable-item {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.4rem 0.5rem;
  border-radius: 4px;
  font-size: 0.85rem;
  font-family: 'JetBrains Mono', monospace;
  transition: background 0.1s;
}

.variable-item:hover {
  background: #313244;
}

.var-icon {
  color: #6c7086;
  flex-shrink: 0;
}

.var-name {
  flex: 1;
  color: #cdd6f4;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.var-kind {
  font-size: 0.7rem;
  color: #6c7086;
  text-transform: lowercase;
  background: #313244;
  padding: 0.1rem 0.3rem;
  border-radius: 3px;
}

.variable-item.function .var-icon {
  color: #f9e2af;
}
</style>
