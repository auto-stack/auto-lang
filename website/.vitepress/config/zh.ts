import { defineConfig, type DefaultTheme } from 'vitepress'
import { sidebarDocsZh } from './sidebar-docs-zh'
import { sidebarBooksZh } from './sidebar-books-zh'

export const zh = defineConfig({
  lang: 'zh-CN',
  description: 'Auto — 全栈应用平台。一门语言编写脚本、后端、UI、AI Agent 与操作系统组件。',

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
    { text: '语言', link: '/zh/docs/language' },
    { text: 'Rust', link: '/zh/rust' },
    { text: 'Python', link: '/zh/python' },
    { text: 'UI', link: '/zh/ui' },
    { text: 'AI', link: '/zh/ai' },
    { text: 'OS', link: '/zh/os' },
    { text: '应用', link: '/zh/apps' },
    { text: '文档', link: '/zh/docs/' },
    { text: 'Playground', link: '/zh/playground' },
    {
      text: 'v0.5',
      items: [
        { text: 'v0.5 发布说明', link: '/zh/docs/releases/v0.5' },
        { text: 'v0.4 发布说明', link: '/zh/docs/releases/v0.4' },
        { text: 'v0.3 发布说明', link: '/zh/docs/releases/v0.3' },
        { text: '参与贡献', link: '/zh/docs/' },
      ],
    },
  ]
}
