<template>
  <!-- PLAN-754: 宿主壳已迁入组件包 PlaygroundIDEApp（双端同形，website /playground 共用）。 -->
  <PlaygroundIDEApp />
</template>

<script setup lang="ts">
import { PlaygroundIDEApp } from 'auto-playground-vue';
</script>
