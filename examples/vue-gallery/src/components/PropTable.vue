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
  <section>
    <h3 class="props-title">Props</h3>
    <table class="props-table">
      <thead>
        <tr>
          <th>Prop</th>
          <th>Type</th>
          <th>Default</th>
          <th>Description</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="r in rows" :key="r.prop">
          <td>{{ r.prop }}</td>
          <td>{{ r.type }}</td>
          <td>{{ r.default ?? '—' }}</td>
          <td>{{ r.desc }}</td>
        </tr>
      </tbody>
    </table>
  </section>
</template>
