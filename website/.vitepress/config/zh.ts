import { defineConfig, type DefaultTheme } from 'vitepress'
import { sidebarDocsZh } from './sidebar-docs-zh'
import { sidebarBooksZh } from './sidebar-books-zh'

export const zh = defineConfig({
  lang: 'zh-CN',
  description: 'Auto — 现代系统与 AI 语言。多目标转译器、Actor 并发、编译期元编程。',

  themeConfig: {
    logo: '/auto.svg',
    nav: nav(),
    sidebar: {
      '/zh/docs/': { base: '/zh/docs/', items: sidebarDocsZh },
      '/zh/books/': { base: '/zh/books/', items: sidebarBooksZh },
    },

    editLink: {
      pattern: 'https://github.com/autostack/auto-lang/edit/main/docs/:path',
      text: '在 GitHub 上编辑此页',
    },

    footer: {
      message: '基于 MIT 许可发布。',
      copyright: 'Copyright © 2024-present Auto Language Contributors',
    },
  },
})

function nav(): DefaultTheme.NavItem[] {
  return [
    { text: '首页', link: '/zh/' },
    { text: 'AI', link: '/zh/ai' },
    { text: 'OS', link: '/zh/os' },
    { text: '文档', link: '/zh/docs/' },
    { text: '教程', link: '/zh/books/' },
    { text: 'Playground', link: '/zh/playground' },
    { text: 'UI 画廊', link: '/zh/ui-gallery' },
    {
      text: 'v0.2',
      items: [
        { text: '发布说明', link: '/zh/releases/v0.2' },
        { text: '参与贡献', link: '/zh/docs/' },
      ],
    },
  ]
}
