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
    { text: 'AI', link: '/ai' },
    { text: 'OS', link: '/os' },
    { text: 'Docs', link: '/docs/' },
    { text: 'Tutorials', link: '/books/' },
    { text: 'Playground', link: '/playground' },
    {
      text: 'UI',
      items: [
        { text: 'Overview', link: '/ui/' },
        { text: 'Components', link: '/ui/gallery/index.html', target: '_self' },
        { text: 'Blocks', link: '/ui/blocks/index.html', target: '_self' },
        { text: 'Charts', link: '/ui/charts/index.html', target: '_self' },
        { text: 'Desktop', link: '/ui-desktop' },
        { text: 'Android', link: '/ui-android' },
        { text: 'Harmony', link: '/ui-harmony' },
      ],
    },
    {
      text: 'v0.3',
      items: [
        { text: 'v0.3 Release Notes', link: '/docs/releases/v0.3' },
        { text: 'v0.2 Release Notes', link: '/docs/releases/v0.2' },
        { text: 'v0.1 Release Notes', link: '/docs/releases/v0.1' },
        { text: 'Contributing', link: '/docs/' },
      ],
    },
  ]
}
