<template>
  <div class="feature-card">
    <div class="icon-wrapper">
      <component :is="icon" :size="24" />
    </div>
    <h3 class="title">{{ title }}</h3>
    <p class="description">{{ description }}</p>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'

defineProps<{
  icon: Component
  title: string
  description: string
}>()
</script>

<style scoped>
.feature-card {
  padding: 1.5rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  transition: all 0.2s ease;
}

.feature-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.08);
  border-color: rgba(99, 102, 241, 0.3);
}

.dark .feature-card:hover {
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.3);
}

.icon-wrapper {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  margin-bottom: 1rem;
}

.title {
  font-size: 1.1rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: hsl(var(--foreground));
}

.description {
  font-size: 0.9rem;
  line-height: 1.6;
  color: hsl(var(--muted-foreground));
  margin: 0;
}
</style>
