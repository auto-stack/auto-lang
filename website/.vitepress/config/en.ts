import { defineConfig, type DefaultTheme } from 'vitepress'
import { sidebarDocsEn } from './sidebar-docs-en'
import { sidebarBooksEn } from './sidebar-books-en'

export const en = defineConfig({
  lang: 'en-US',
  description: 'Auto — A full-stack application platform. One language for scripts, backends, UIs, AI agents, and OS components.',

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
    { text: 'Language', link: '/docs/language' },
    { text: 'Rust', link: '/rust' },
    { text: 'Python', link: '/python' },
    { text: 'UI', link: '/ui' },
    { text: 'AI', link: '/ai' },
    { text: 'OS', link: '/os' },
    { text: 'Apps', link: '/apps' },
    { text: 'Docs', link: '/docs/' },
    { text: 'Playground', link: '/playground' },
    {
      text: 'v0.5',
      items: [
        { text: 'v0.5 Release Notes', link: '/docs/releases/v0.5' },
        { text: 'v0.4 Release Notes', link: '/docs/releases/v0.4' },
        { text: 'v0.3 Release Notes', link: '/docs/releases/v0.3' },
        { text: 'Contributing', link: '/docs/' },
      ],
    },
  ]
}
