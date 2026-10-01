import { defineConfig, type DefaultTheme } from 'vitepress'
import { sidebarDocsEn } from './sidebar-docs-en'
import { sidebarBooksEn, sidebarBooksEnByBook } from './sidebar-books-en'
import { editLinkPattern } from './author-source'
import { NAV_TOP_LEAVES, NAV_GROUPS, NAV_HIGHLIGHT } from '../theme/data/navigation'

// PLAN-718 T-03：编辑链接 pattern 来自生成的自包含函数（见 author-source.ts 头注）。
export const en = defineConfig({
  lang: 'en-US',
  description: 'Auto — A full-stack application platform. One language for scripts, backends, UIs, AI agents, and OS components.',

  themeConfig: {
    logo: '/auto.svg',
    nav: nav(),
    sidebar: {
      '/docs/': { base: '/docs/', items: sidebarDocsEn },
      // PLAN-718 T-03：每书一棵侧栏（最长前缀命中），prev/next 与目录只覆盖
      // 当前书，消除跨书章节跳转；'/books/' 仅 hub 落这里。
      ...Object.fromEntries(
        Object.entries(sidebarBooksEnByBook).map(
          ([id, items]) => [`/books/${id}/`, { base: `/books/${id}/`, items } satisfies DefaultTheme.Sidebar],
        ),
      ),
      '/books/': { base: '/books/', items: sidebarBooksEn },
    },

    sidebarMenuLabel: 'Menu',
    docFooter: { prev: 'Previous page', next: 'Next page' },
    returnToTopLabel: 'Return to top',

    editLink: {
      pattern: editLinkPattern,
      text: 'Edit this page',
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
