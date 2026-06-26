<script setup lang="ts">
import { blocks, kindOrder } from '../blocks'

const grouped = kindOrder
  .map((kind) => ({ kind, items: blocks.filter((b) => b.kind === kind) }))
  .filter((g) => g.items.length > 0)
</script>

<template>
  <div>
    <header class="home-header">
      <h2 class="home-title">AutoUI Blocks</h2>
      <p class="home-lead">
        Each block is a <em>spec</em> (natural-language + structured contract) plus
        one or more <em>reference implementations</em> and a <em>gotchas</em> list —
        assembled from widgets. This gallery browses the catalog as-authored.
      </p>
    </header>
    <section v-for="g in grouped" :key="g.kind" class="home-group">
      <h3 class="home-group-label">{{ g.kind }}</h3>
      <div class="home-grid">
        <RouterLink
          v-for="b in g.items"
          :key="`/${b.kind}/${b.name}`"
          :to="`/${b.kind}/${b.name}`"
          class="home-card"
        >
          <div class="home-card-name">{{ b.kind }}/{{ b.name }}</div>
          <div class="home-card-blurb">{{ Object.keys(b.references).length }} reference(s)</div>
        </RouterLink>
      </div>
    </section>
  </div>
</template>
