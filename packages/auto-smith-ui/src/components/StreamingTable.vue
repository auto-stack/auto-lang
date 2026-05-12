<template>
  <div class="streaming-table" :class="{ final }">
    <table>
      <thead>
        <tr>
          <th v-for="col in safeColumns" :key="col">{{ col }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, idx) in safeRows" :key="idx">
          <td v-for="col in safeColumns" :key="col">{{ row[col] ?? '' }}</td>
        </tr>
        <tr v-if="!final" class="loading-row">
          <td :colspan="Math.max(1, safeColumns.length)">
            <span class="loading-dots">Loading</span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    columns?: string[]
    rows?: Record<string, any>[]
    final?: boolean
  }>(),
  {
    columns: () => [],
    rows: () => [],
    final: false,
  }
)

const safeColumns = computed(() => props.columns ?? [])
const safeRows = computed(() => props.rows ?? [])
</script>

<style scoped>
.streaming-table {
  margin: 0.5rem 0;
  overflow-x: auto;
}

.streaming-table table {
  border-collapse: collapse;
  width: 100%;
  font-size: 0.85rem;
}

.streaming-table th,
.streaming-table td {
  border: 1px solid var(--af-border);
  padding: 0.4rem 0.6rem;
  text-align: left;
}

.streaming-table th {
  background: var(--af-card);
  font-weight: 600;
  color: var(--af-fg);
}

.streaming-table td {
  color: var(--af-fg);
}

.streaming-table tr:nth-child(even) {
  background: hsl(var(--muted) / 0.5);
}

.streaming-table .loading-row td {
  color: var(--af-muted);
  font-style: italic;
  text-align: center;
}

.loading-dots::after {
  content: '';
  animation: dots 1.4s infinite both;
}

@keyframes dots {
  0%, 80%, 100% { content: ''; }
  40% { content: '.'; }
  60% { content: '..'; }
}
</style>
