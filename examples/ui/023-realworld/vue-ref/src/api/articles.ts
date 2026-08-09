import { reactive, computed } from 'vue'
import { seedArticles, seedComments, seedTags } from './mock'
import type { Article, Comment } from './mock'

// Plan 405 vue-ref articles store. Mock filtering against seedArticles.
export const articleStore = reactive({
  all: [...seedArticles] as Article[],
  activeTag: '' as string,
  loading: false,

  get tags() { return seedTags },

  get filtered(): Article[] {
    if (!this.activeTag) return this.all
    return this.all.filter(a => a.tagList.split(',').map(t => t.trim()).includes(this.activeTag))
  },

  bySlug(slug: string): Article | undefined {
    return this.all.find(a => a.slug === slug)
  },

  commentsFor(slug: string): Comment[] {
    return seedComments[slug] ?? []
  },

  setTag(tag: string) {
    this.activeTag = this.activeTag === tag ? '' : tag
  },
})
