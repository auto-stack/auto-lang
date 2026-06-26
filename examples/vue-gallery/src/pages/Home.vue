<script setup lang="ts">
import { widgetGroups } from '../widgets'
</script>

<template>
  <div>
    <header class="mb-8">
      <h2 class="text-2xl font-bold tracking-tight">@auto-ui/widgets — vue-gallery</h2>
      <p class="text-muted-foreground mt-1">
        This app consumes the <code class="font-mono">@auto-ui/widgets</code> package
        directly (registry subpath + precompiled <code class="font-mono">styles.css</code>).
        Every page renders one widget with its variants and states — a live visual
        regression and the package's dogfood consumer.
      </p>
    </header>
    <section v-for="group in widgetGroups" :key="group.label" class="mb-8">
      <h3 class="text-sm uppercase tracking-wide text-muted-foreground mb-2">{{ group.label }}</h3>
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
        <RouterLink
          v-for="w in group.widgets"
          :key="w.route"
          :to="w.route"
          class="block rounded-lg border border-border bg-card p-4 hover:bg-accent hover:text-accent-foreground transition-colors"
        >
          <div class="font-mono font-semibold">{{ w.name }}</div>
          <div class="text-sm text-muted-foreground mt-1">{{ w.blurb }}</div>
        </RouterLink>
      </div>
    </section>
  </div>
</template>
