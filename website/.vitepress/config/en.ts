import { defineConfig, type DefaultTheme } from 'vitepress'
import { sidebarDocsEn } from './sidebar-docs-en'
import { sidebarBooksEn } from './sidebar-books-en'
import { NAV_TOP_LEAVES, NAV_GROUPS, NAV_HIGHLIGHT } from '../theme/data/navigation'

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

// PLAN-715 T-02：默认主题 nav 与 UnifiedNavbar 同源自 navigation.ts，消除分叉。
// （默认 VPNav 被 CSS 隐藏，实际可见导航是 UnifiedNavbar；此处保持一致供
// VitePress 内部消费与将来的回退。）发布说明深链由 /docs/ 侧栏承载。
function nav(): DefaultTheme.NavItem[] {
  const items: DefaultTheme.NavItem[] = [
    ...NAV_TOP_LEAVES.map((i) => ({ text: i.labelEn, link: i.en })),
    ...NAV_GROUPS.map((g) => ({
      text: g.labelEn,
      items: g.items.map((i) => ({ text: i.labelEn, link: i.en })),
    })),
    { text: NAV_HIGHLIGHT.labelEn, link: NAV_HIGHLIGHT.en },
  ]
  return items
}
