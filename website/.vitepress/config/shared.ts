import { defineConfig } from 'vitepress'

export const shared = defineConfig({
  title: 'Auto Language',
  description: 'Auto — A modern systems and AI language',
  lastUpdated: true,
  cleanUrls: true,
  metaChunk: false,
  ignoreDeadLinks: true,
  appearance: 'dark',

  themeConfig: {
    outline: {
      level: [2, 3],
      label: 'On this Page',
    },

    // PLAN-715 T-02：站内搜索 = VitePress local provider（minisearch 离线索引，
    // 按 locale 分库，覆盖专题正文/docs/books；接线与版本约束见
    // docs/reports/p715-website-ui-baseline.md §3）。按钮/弹窗 UI 由主题层
    // SiteSearch.vue + VPNavBarSearch(Teleport 到 body) 提供。
    search: {
      provider: 'local',
      options: {
        translations: {
          search: { placeholder: 'Search docs…' },
        },
      },
    },
  },

  markdown: {
    codeCopyButtonTitle: 'Copy Code',
    theme: {
      light: 'github-light',
      dark: 'github-dark',
    },
  },

  head: [
    ['link', { rel: 'icon', href: '/auto.svg', type: 'image/svg+xml' }],
    ['link', { rel: 'icon', href: '/auto.png', type: 'image/png' }],
    ['link', { rel: 'apple-touch-icon', href: '/auto.png' }],
    ['meta', { name: 'theme-color', content: '#6366f1' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:locale', content: 'en' }],
    ['meta', { property: 'og:title', content: 'Auto Language' }],
    ['meta', { property: 'og:site_name', content: 'Auto Language' }],
    ['meta', { property: 'og:image', content: '/auto.png' }],
  ],
})
