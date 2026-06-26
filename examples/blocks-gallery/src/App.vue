<script setup lang="ts">
import { computed } from 'vue'
import { blocks, kindOrder } from './blocks'

const grouped = computed(() =>
  kindOrder
    .map((kind) => ({ kind, items: blocks.filter((b) => b.kind === kind) }))
    .filter((g) => g.items.length > 0),
)
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <h1 class="sidebar-title">AutoUI Blocks</h1>
      <p class="sidebar-subtitle">Skill-tier catalog</p>
      <nav v-for="g in grouped" :key="g.kind" class="nav-group">
        <div class="nav-group-label">{{ g.kind }}</div>
        <RouterLink
          v-for="b in g.items"
          :key="`/${b.kind}/${b.name}`"
          :to="`/${b.kind}/${b.name}`"
          class="nav-link"
        >
          {{ b.name }}
        </RouterLink>
      </nav>
    </aside>
    <main class="content">
      <RouterView />
    </main>
  </div>
</template>
