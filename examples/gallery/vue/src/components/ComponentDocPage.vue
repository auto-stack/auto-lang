<script setup lang="ts">
defineProps<{
  title: string
  description: string
  installCommand: string
}>()
</script>

<template>
  <div class="flex flex-col">
    <!-- Hero Header -->
    <div class="relative -mx-4 -mt-4 px-4 py-8 md:py-12 mb-8 md:-mx-6 md:-mt-6 md:px-6 lg:-mx-8 lg:-mt-8 lg:px-8 lg:py-16">
      <!-- Gradient background using theme primary color -->
      <div class="absolute inset-0 bg-gradient-to-br from-primary/[0.08] via-transparent to-primary/[0.04] border-b border-primary/10" />
      <!-- Decorative blob -->
      <div class="absolute top-0 right-0 w-64 h-64 md:w-96 md:h-96 bg-primary/[0.06] rounded-full blur-3xl -translate-y-1/2 translate-x-1/4" />

      <div class="relative flex flex-col gap-3 md:gap-4 max-w-3xl">
        <!-- Theme-colored accent bar + title -->
        <div class="flex items-center gap-3 md:gap-4">
          <div class="hidden md:block w-1.5 h-10 rounded-full bg-primary shrink-0" />
          <h1 class="text-3xl md:text-5xl font-bold tracking-tight">{{ title }}</h1>
        </div>
        <p class="text-muted-foreground text-base md:text-xl leading-relaxed md:pl-6">{{ description }}</p>
      </div>
    </div>

    <!-- Installation -->
    <div class="flex flex-col gap-3 mb-10">
      <h2 class="text-xl md:text-2xl font-semibold tracking-tight flex items-center gap-2">
        <span class="w-1 h-5 rounded-full bg-primary/60 inline-block" />
        Installation
      </h2>
      <installation-block :command="installCommand" />
    </div>

    <!-- Demo slots -->
    <slot />

    <!-- Properties -->
    <div class="flex flex-col gap-3 mt-10">
      <h2 class="text-xl md:text-2xl font-semibold tracking-tight flex items-center gap-2">
        <span class="w-1 h-5 rounded-full bg-primary/60 inline-block" />
        Properties
      </h2>
      <slot name="properties" />
    </div>
  </div>
</template>
