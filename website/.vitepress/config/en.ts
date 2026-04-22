import { defineConfig, type DefaultTheme } from 'vitepress'
import { sidebarDocsEn } from './sidebar-docs-en'
import { sidebarBooksEn } from './sidebar-books-en'

export const en = defineConfig({
  lang: 'en-US',
  description: 'Auto — A modern systems and AI language. Multi-target transpiler, actor concurrency, comptime metaprogramming.',

  themeConfig: {
    logo: '/auto.svg',
    nav: nav(),
    sidebar: {
      '/docs/': { base: '/docs/', items: sidebarDocsEn },
      '/books/': { base: '/books/', items: sidebarBooksEn },
    },

    editLink: {
      pattern: 'https://github.com/autostack/auto-lang/edit/main/docs/:path',
      text: 'Edit this page on GitHub',
    },

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2024-present Auto Language Contributors',
    },
  },
})

function nav(): DefaultTheme.NavItem[] {
  return [
    { text: 'Home', link: '/' },
    { text: 'Docs', link: '/docs/' },
    { text: 'Books', link: '/books/' },
    { text: 'Playground', link: '/playground' },
    {
      text: 'v0.2',
      items: [
        { text: 'Release Notes', link: '/releases/v0.2' },
        { text: 'Contributing', link: '/docs/' },
      ],
    },
  ]
}
