<script setup lang="ts">
// Showcase shell: title + optional description + preview slot + optional code.
defineProps<{
  title: string
  description?: string
  code?: string
}>()
</script>

<template>
  <section class="mb-8">
    <h3 class="text-base font-semibold mb-1">{{ title }}</h3>
    <p v-if="description" class="text-sm text-muted-foreground mb-3">{{ description }}</p>
    <div class="rounded-md border border-border p-4 flex flex-wrap items-center gap-3 mb-2 bg-card">
      <slot />
    </div>
    <pre v-if="code" class="text-xs bg-muted/50 rounded p-2 overflow-x-auto"><code>{{ code }}</code></pre>
  </section>
</template>
