<template>
  <div class="output-table">
    <div v-if="error" class="table-error">{{ error }}</div>
    <table v-else-if="rows.length > 0">
      <thead>
        <tr>
          <th v-for="col in columns" :key="col">{{ col }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, i) in rows" :key="i">
          <td v-for="col in columns" :key="col">
            <template v-if="row[col] !== undefined && row[col] !== null">
              {{ formatValue(row[col]) }}
            </template>
            <span v-else class="null-value">null</span>
          </td>
        </tr>
      </tbody>
    </table>
    <div v-else class="empty-table">No table data</div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  source: string
}>()

const parsed = computed(() => {
  try {
    return JSON.parse(props.source)
  } catch {
    return null
  }
})

const error = computed(() => {
  if (parsed.value === null) return 'Invalid table data. Expected JSON array of objects.'
  if (!Array.isArray(parsed.value)) return 'Table data must be an array of objects.'
  return null
})

const rows = computed(() => {
  if (!Array.isArray(parsed.value)) return []
  return parsed.value.filter((r: any) => r !== null && typeof r === 'object')
})

const columns = computed(() => {
  const keys = new Set<string>()
  for (const row of rows.value) {
    for (const key of Object.keys(row)) {
      keys.add(key)
    }
  }
  return Array.from(keys)
})

function formatValue(v: any): string {
  if (typeof v === 'string') return v
  if (typeof v === 'number') return String(v)
  if (typeof v === 'boolean') return String(v)
  return JSON.stringify(v)
}
</script>

<style scoped>
.output-table {
  padding: 0.5rem 0.75rem;
  overflow-x: auto;
}

.output-table table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8rem;
  font-family: 'JetBrains Mono', monospace;
}

.output-table th,
.output-table td {
  padding: 0.4rem 0.6rem;
  text-align: left;
  border-bottom: 1px solid #313244;
}

.output-table th {
  color: #6c7086;
  font-weight: 600;
  background: #181825;
  position: sticky;
  top: 0;
}

.output-table td {
  color: #cdd6f4;
}

.output-table tr:hover td {
  background: #31324444;
}

.null-value {
  color: #6c7086;
  font-style: italic;
}

.table-error {
  color: #f38ba8;
  font-size: 0.8rem;
  padding: 0.5rem;
  background: #f38ba811;
  border-radius: 4px;
}

.empty-table {
  color: #6c7086;
  font-size: 0.85rem;
  padding: 1rem;
  text-align: center;
}
</style>
