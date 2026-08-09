<script setup lang="ts">
// Plan 405 vue-ref ArticleCard: one article preview in the feed.
import { useRouter } from 'vue-router'
import type { Article } from '../api/mock'
defineProps<{ article: Article }>()
const router = useRouter()
</script>

<template>
  <div class="py-4 border-b border-gray-100">
    <div class="flex items-center gap-2 mb-1">
      <img :src="`https://i.pravatar.cc/40?u=${article.author}`" class="w-8 h-8 rounded-full" alt="" />
      <div class="text-sm leading-tight">
        <div class="font-medium text-brand">{{ article.author }}</div>
        <div class="text-xs text-gray-400">{{ article.createdAt }}</div>
      </div>
      <button class="ml-auto px-2 py-0.5 text-xs rounded-full border border-brand text-brand hover:bg-brand hover:text-white">
        ♥ {{ article.favoritesCount }}
      </button>
    </div>
    <a href="#" class="no-underline" @click.prevent="router.push(`/article/${article.slug}`)">
      <h2 class="text-xl font-semibold text-gray-800">{{ article.title }}</h2>
      <p class="text-gray-500 text-sm mt-1">{{ article.description }}</p>
    </a>
    <div class="flex items-center mt-2">
      <button class="text-xs text-gray-400 hover:underline" @click="router.push(`/article/${article.slug}`)">Read more...</button>
      <div class="ml-auto flex gap-1">
        <span v-for="t in article.tagList.split(',')" :key="t" class="tag-pill">{{ t.trim() }}</span>
      </div>
    </div>
  </div>
</template>
