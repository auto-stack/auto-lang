<script setup lang="ts">
import { widgetGroups } from '../widgets'
</script>

<template>
  <div>
    <header class="home-header">
      <h2 class="home-title">@auto-ui/widgets — vue-gallery</h2>
      <p class="home-lead">
        This app consumes the <code>@auto-ui/widgets</code> package directly
        (registry subpath + precompiled <code>styles.css</code>). Every page
        renders one widget with its variants and states — a live visual regression
        and the package's dogfood consumer.
      </p>
    </header>
    <section v-for="group in widgetGroups" :key="group.label" class="home-group">
      <h3 class="home-group-label">{{ group.label }}</h3>
      <div class="home-grid">
        <RouterLink
          v-for="w in group.widgets"
          :key="w.route"
          :to="w.route"
          class="home-card"
        >
          <div class="home-card-name">{{ w.name }}</div>
          <div class="home-card-blurb">{{ w.blurb }}</div>
        </RouterLink>
      </div>
    </section>
  </div>
</template>
