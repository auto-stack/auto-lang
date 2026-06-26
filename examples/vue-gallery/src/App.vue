<script setup lang="ts">
import { widgetGroups } from './widgets'
</script>

<template>
  <div class="min-h-screen flex bg-background text-foreground">
    <aside class="w-60 shrink-0 border-r border-border p-4 overflow-y-auto">
      <h1 class="text-lg font-semibold mb-1">@auto-ui/widgets</h1>
      <p class="text-xs text-muted-foreground mb-4">vue-gallery dogfood</p>
      <nav v-for="group in widgetGroups" :key="group.label" class="mb-4">
        <div class="text-xs uppercase tracking-wide text-muted-foreground mb-1">
          {{ group.label }}
        </div>
        <RouterLink
          v-for="w in group.widgets"
          :key="w.route"
          :to="w.route"
          class="block rounded px-2 py-1 text-sm hover:bg-accent hover:text-accent-foreground"
          active-class="!bg-primary !text-primary-foreground"
        >
          {{ w.name }}
        </RouterLink>
      </nav>
    </aside>
    <main class="flex-1 p-8 overflow-y-auto">
      <RouterView />
    </main>
  </div>
</template>
