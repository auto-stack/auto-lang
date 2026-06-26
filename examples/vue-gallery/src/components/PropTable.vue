<script setup lang="ts">
interface Row {
  prop: string
  type: string
  default?: string
  desc: string
}
defineProps<{ rows: Row[] }>()
</script>

<template>
  <section class="mb-8">
    <h3 class="text-base font-semibold mb-2">Props</h3>
    <table class="w-full text-sm border border-border rounded">
      <thead class="bg-muted/50 text-left">
        <tr>
          <th class="px-3 py-2 font-medium">Prop</th>
          <th class="px-3 py-2 font-medium">Type</th>
          <th class="px-3 py-2 font-medium">Default</th>
          <th class="px-3 py-2 font-medium">Description</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="r in rows" :key="r.prop" class="border-t border-border align-top">
          <td class="px-3 py-2 font-mono">{{ r.prop }}</td>
          <td class="px-3 py-2 font-mono text-muted-foreground">{{ r.type }}</td>
          <td class="px-3 py-2 font-mono text-muted-foreground">{{ r.default ?? '—' }}</td>
          <td class="px-3 py-2">{{ r.desc }}</td>
        </tr>
      </tbody>
    </table>
  </section>
</template>
