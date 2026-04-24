<template>
  <div class="stat-card">
    <div class="stat-value" :style="valueStyle">{{ value }}</div>
    <div class="stat-label">{{ label }}</div>
    <div v-if="description" class="stat-desc">{{ description }}</div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  value: string
  label: string
  description?: string
  color?: string
}>()

const valueStyle = computed(() => {
  const c = props.color || '#6366f1'
  return {
    background: `linear-gradient(120deg, ${c} 30%, ${lighten(c, 20)} 70%)`,
    '-webkit-background-clip': 'text',
    '-webkit-text-fill-color': 'transparent',
    'background-clip': 'text',
  }
})

function lighten(hex: string, amount: number): string {
  const num = parseInt(hex.replace('#', ''), 16)
  const r = Math.min(255, (num >> 16) + amount)
  const g = Math.min(255, ((num >> 8) & 0x00ff) + amount)
  const b = Math.min(255, (num & 0x0000ff) + amount)
  return `rgb(${r}, ${g}, ${b})`
}
</script>

<style scoped>
.stat-card {
  padding: 2rem 1.5rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  text-align: center;
  transition: all 0.2s ease;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.08);
  border-color: rgba(99, 102, 241, 0.3);
}

.dark .stat-card:hover {
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.3);
}

.stat-value {
  font-size: 2.5rem;
  font-weight: 800;
  line-height: 1.1;
  margin-bottom: 0.5rem;
  letter-spacing: -0.02em;
}

.stat-label {
  font-size: 1rem;
  font-weight: 600;
  color: hsl(var(--foreground));
  margin-bottom: 0.25rem;
}

.stat-desc {
  font-size: 0.875rem;
  color: hsl(var(--muted-foreground));
  line-height: 1.5;
}
</style>
