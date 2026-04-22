import { defineConfig } from 'vitepress'

export const shared = defineConfig({
  title: 'Auto Language',
  description: 'Auto — A modern systems and AI language',
  lastUpdated: true,
  cleanUrls: true,
  metaChunk: true,
  ignoreDeadLinks: true,
  appearance: 'dark',

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
    ['meta', { property: 'og:image', content: '/og-image.png' }],
  ],
})
