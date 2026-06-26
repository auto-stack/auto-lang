<script setup lang="ts">
// Showcase shell: title + optional description + preview slot + optional code.
defineProps<{
  title: string
  description?: string
  code?: string
}>()
</script>

<template>
  <section class="demo-block">
    <h3 class="demo-title">{{ title }}</h3>
    <p v-if="description" class="demo-desc">{{ description }}</p>
    <div class="demo-preview">
      <slot />
    </div>
    <pre v-if="code" class="demo-code"><code>{{ code }}</code></pre>
  </section>
</template>
