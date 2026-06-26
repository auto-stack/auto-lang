<script setup lang="ts">
// Phase 1 smoke test: consume @auto-ui/widgets directly via its exports map.
import { Button } from '@auto-ui/widgets/registry/button'
</script>

<template>
  <main style="padding: 2rem; display: flex; gap: 1rem; align-items: center">
    <Button variant="default">It works</Button>
    <Button variant="destructive">Destructive</Button>
    <Button variant="outline">Outline</Button>
  </main>
</template>
