<script setup lang="ts">
// Plan 405 vue-ref Home: Global Feed + Popular Tags sidebar.
import { articleStore } from '../api/articles'
import ArticleCard from '../components/ArticleCard.vue'
</script>

<template>
  <div class="max-w-3xl mx-auto px-4 py-6 flex gap-6">
    <div class="flex-1">
      <div class="flex gap-4 border-b border-brand pb-1 mb-2">
        <button class="text-brand font-medium" :class="{ 'text-gray-400': articleStore.activeTag }">Global Feed</button>
        <button v-if="articleStore.activeTag" class="text-brand font-medium"># {{ articleStore.activeTag }}</button>
      </div>
      <div v-if="articleStore.filtered.length === 0" class="py-8 text-center text-gray-400">No articles are here... yet.</div>
      <ArticleCard v-for="a in articleStore.filtered" :key="a.slug" :article="a" />
    </div>
    <div class="w-48 hidden md:block">
      <div class="bg-gray-50 rounded p-3">
        <div class="text-sm font-medium text-gray-500 mb-2">Popular Tags</div>
        <div class="flex flex-wrap gap-1">
          <button v-for="t in articleStore.tags" :key="t"
            class="tag-pill" :class="{ 'bg-brand text-white border-brand': articleStore.activeTag === t }"
            @click="articleStore.setTag(t)">{{ t }}</button>
        </div>
      </div>
    </div>
  </div>
</template>
