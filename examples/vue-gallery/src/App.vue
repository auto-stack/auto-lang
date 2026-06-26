<script setup lang="ts">
import { widgetGroups } from './widgets'
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <h1 class="sidebar-title">@auto-ui/widgets</h1>
      <p class="sidebar-subtitle">vue-gallery dogfood</p>
      <nav v-for="group in widgetGroups" :key="group.label" class="nav-group">
        <div class="nav-group-label">{{ group.label }}</div>
        <RouterLink
          v-for="w in group.widgets"
          :key="w.route"
          :to="w.route"
          class="nav-link"
        >
          {{ w.name }}
        </RouterLink>
      </nav>
    </aside>
    <main class="content">
      <RouterView />
    </main>
  </div>
</template>
