<script setup lang="ts">
// Plan 405 vue-ref ArticleDetail: article body + comments list (read-only).
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { articleStore } from '../api/articles'
const route = useRoute()
const router = useRouter()
const slug = computed(() => String(route.params.slug))
const article = computed(() => articleStore.bySlug(slug.value))
const comments = computed(() => articleStore.commentsFor(slug.value))
</script>

<template>
  <div v-if="article">
    <div class="bg-gray-50 py-6">
      <div class="max-w-2xl mx-auto px-4">
        <h1 class="text-3xl font-semibold text-gray-800 mb-3">{{ article.title }}</h1>
        <div class="flex items-center gap-2">
          <img :src="`https://i.pravatar.cc/40?u=${article.author}`" class="w-8 h-8 rounded-full" alt="" />
          <div class="text-sm">
            <div class="text-brand font-medium">{{ article.author }}</div>
            <div class="text-xs text-gray-400">{{ article.createdAt }}</div>
          </div>
          <button class="ml-auto px-3 py-1 text-xs rounded border border-gray-300 hover:bg-gray-100">+ Follow {{ article.author }}</button>
          <button class="px-3 py-1 text-xs rounded border border-brand text-brand hover:bg-brand hover:text-white">♥ Favorite Article ({{ article.favoritesCount }})</button>
        </div>
      </div>
    </div>
    <div class="max-w-2xl mx-auto px-4 py-6">
      <p class="text-gray-700 leading-relaxed whitespace-pre-line">{{ article.body }}</p>
      <div class="flex gap-1 mt-6">
        <span v-for="t in article.tagList.split(',')" :key="t" class="tag-pill">{{ t.trim() }}</span>
      </div>
      <hr class="my-6" />
      <button class="text-sm text-brand hover:underline" @click="router.push('/')">← Back to feed</button>

      <div class="mt-6">
        <div class="text-sm font-medium text-gray-500 mb-2">Comments ({{ comments.length }})</div>
        <div v-if="comments.length === 0" class="text-gray-400 text-sm py-4">No comments yet.</div>
        <div v-for="c in comments" :key="c.id" class="border border-gray-200 rounded p-3 mb-2">
          <div class="flex items-center gap-2 mb-1">
            <img :src="`https://i.pravatar.cc/30?u=${c.author}`" class="w-6 h-6 rounded-full" alt="" />
            <span class="text-sm text-brand font-medium">{{ c.author }}</span>
            <span class="text-xs text-gray-400 ml-auto">{{ c.createdAt }}</span>
          </div>
          <p class="text-sm text-gray-700">{{ c.body }}</p>
        </div>
      </div>
    </div>
  </div>
  <div v-else class="max-w-2xl mx-auto px-4 py-12 text-center text-gray-400">
    Article not found.
    <button class="block mx-auto mt-2 text-brand hover:underline" @click="router.push('/')">← Home</button>
  </div>
</template>
