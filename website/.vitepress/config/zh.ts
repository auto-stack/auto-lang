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
    {
      text: 'UI',
      items: [
        { text: '概览', link: '/zh/ui/' },
        { text: '组件', link: '/ui/gallery/index.html', target: '_self' },
        { text: '区块', link: '/ui/blocks/index.html', target: '_self' },
        { text: '图表', link: '/ui/charts/index.html', target: '_self' },
        { text: '桌面端', link: '/zh/ui-desktop' },
        { text: 'Android', link: '/zh/ui-android' },
        { text: '鸿蒙', link: '/zh/ui-harmony' },
      ],
    },
    {
      text: 'v0.3',
      items: [
        { text: 'v0.3 发布说明', link: '/zh/docs/releases/v0.3' },
        { text: 'v0.2 发布说明', link: '/zh/docs/releases/v0.2' },
        { text: 'v0.1 发布说明', link: '/zh/docs/releases/v0.1' },
        { text: '参与贡献', link: '/zh/docs/' },
      ],
    },
  ]
}
