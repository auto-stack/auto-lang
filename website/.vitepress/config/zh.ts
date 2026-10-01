import { defineConfig, type DefaultTheme } from 'vitepress'
import { sidebarDocsZh } from './sidebar-docs-zh'
import { sidebarBooksZh, sidebarBooksZhByBook } from './sidebar-books-zh'
import { editLinkPattern } from './author-source'
import { NAV_TOP_LEAVES, NAV_GROUPS, NAV_HIGHLIGHT } from '../theme/data/navigation'

// PLAN-718 T-03：编辑链接 pattern 来自生成的自包含函数（见 author-source.ts 头注）。
export const zh = defineConfig({
  lang: 'zh-CN',
  description: 'Auto — 全栈应用平台。一门语言编写脚本、后端、UI、AI Agent 与操作系统组件。',

  themeConfig: {
    logo: '/auto.svg',
    nav: nav(),
    sidebar: {
      '/zh/docs/': { base: '/zh/docs/', items: sidebarDocsZh },
      ...Object.fromEntries(
        Object.entries(sidebarBooksZhByBook).map(
          ([id, items]) => [`/zh/books/${id}/`, { base: `/zh/books/${id}/`, items } satisfies DefaultTheme.Sidebar],
        ),
      ),
      '/zh/books/': { base: '/zh/books/', items: sidebarBooksZh },
    },

    // PLAN-715 T-02：local search 中文 UI 文案（索引由 shared 的 provider 声明建立）。
    search: {
      provider: 'local',
      options: {
        translations: {
          search: {
            placeholder: '搜索文档…',
          },
          button: {
            buttonText: '搜索文档',
            buttonAriaLabel: '搜索文档',
          },
          modal: {
            noResultsText: '没有找到结果',
            resetButtonTitle: '清空条件',
            footer: {
              selectText: '选择',
              navigateText: '切换',
              closeText: '关闭',
            },
          },
        },
      },
    },

    // PLAN-718 T-03：原生阅读控件双语。
    outline: { level: [2, 3], label: '此页内容' },
    sidebarMenuLabel: '目录',
    docFooter: { prev: '上一页', next: '下一页' },
    returnToTopLabel: '回到顶部',

    // PLAN-718 T-04：复制按钮标题由围栏渲染插件按 env.relativePath 本地化
    // （locale 级 markdown.codeCopyButtonTitle 不参与 SSG 渲染，VitePress 限制）。

    editLink: {
      pattern: editLinkPattern,
      text: '编辑此页',
    },

    footer: {
      message: '基于 MIT 许可发布。',
      copyright: 'Copyright © 2024-present Auto Language Contributors',
    },
  },
})

// PLAN-715 T-02：默认主题 nav 与 UnifiedNavbar 同源自 navigation.ts（EN 链接为
// 共享 SPA 时中英同链）；可见导航实际由 UnifiedNavbar 承载。
function nav(): DefaultTheme.NavItem[] {
  const items: DefaultTheme.NavItem[] = [
    ...NAV_TOP_LEAVES.map((i) => ({ text: i.labelZh, link: i.zh })),
    ...NAV_GROUPS.map((g) => ({
      text: g.labelZh,
      items: g.items.map((i) => ({ text: i.labelZh, link: i.zh })),
    })),
    { text: NAV_HIGHLIGHT.labelZh, link: NAV_HIGHLIGHT.zh },
  ]
  return items
}
