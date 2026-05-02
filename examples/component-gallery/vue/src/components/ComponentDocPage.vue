<script setup lang="ts">
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'

const props = defineProps<{
  title: string
  description: string
  installCommand: string
}>()
</script>

<template>
  <div class="flex flex-col gap-8">
    <!-- Header -->
    <div class="flex flex-col gap-2">
      <h1 class="text-3xl md:text-4xl font-bold tracking-tight">{{ title }}</h1>
      <p class="text-muted-foreground text-base md:text-lg">{{ description }}</p>
    </div>

    <!-- Installation -->
    <div class="flex flex-col gap-3">
      <h2 class="text-xl md:text-2xl font-semibold tracking-tight">Installation</h2>
      <installation-block :command="installCommand" />
    </div>

    <!-- Demo slots -->
    <slot />

    <!-- Properties -->
    <div class="flex flex-col gap-3">
      <h2 class="text-xl md:text-2xl font-semibold tracking-tight">Properties</h2>
      <slot name="properties" />
    </div>
  </div>
</template>
