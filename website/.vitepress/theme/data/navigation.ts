// PLAN-715 T-02：全站导航单一来源。
// UnifiedNavbar（桌面/手机）与 en.ts/zh.ts 的 VitePress 默认导航都从这里取数，
// 消除 UnifiedNavbar 硬编码英文与 en/zh nav 的分叉。
//
// 规则（docs/reports/p715-website-ui-baseline.md §5）：
// - 每项带 en/zh 链接；zh 链接缺省时取 en 链接加 /zh 前缀（docs/books 与全部
//   手工专题页双语 1:1，见 prepare-content.js 与站点文件清单）。
// - shared: true 表示双语共享 SPA（public/ui/*，无中文镜像，双语同 URL），
//   导航不造 /zh/ui/gallery 之类的假路由。
// - docs/books 深链在 zh 侧可能有 EN 回落内容（.cn.md 缺失时），但 URL 恒有效，
//   语言切换按前缀规则即可，不会 404。

export interface NavLeaf {
  key: string
  en: string
  zh: string
  labelEn: string
  labelZh: string
  /** 双语共享 SPA 入口（不造中文镜像） */
  shared?: boolean
  /** 右侧强调入口（在线体验等） */
  highlight?: boolean
}

export interface NavGroup {
  key: string
  labelEn: string
  labelZh: string
  items: NavLeaf[]
}

function leaf(
  key: string,
  labelEn: string,
  labelZh: string,
  en: string,
  opts: { zh?: string; shared?: boolean; highlight?: boolean } = {},
): NavLeaf {
  return {
    key,
    labelEn,
    labelZh,
    en,
    zh: opts.zh ?? (en === '/' ? '/zh/' : '/zh' + en),
    shared: opts.shared,
    highlight: opts.highlight,
  }
}

const sharedSpa = (key: string, labelEn: string, labelZh: string, href: string): NavLeaf => ({
  key,
  labelEn,
  labelZh,
  en: href,
  zh: href,
  shared: true,
})

export const NAV_GROUPS: NavGroup[] = [
  {
    key: 'lang',
    labelEn: 'Language & Ecosystem',
    labelZh: '语言与生态',
    items: [
      leaf('language', 'Language', '语言', '/docs/language'),
      leaf('rust', 'Rust', 'Rust', '/rust'),
      leaf('python', 'Python', 'Python', '/python'),
      leaf('ai', 'AI', 'AI', '/ai'),
      leaf('os', 'OS', 'OS', '/os'),
    ],
  },
  {
    key: 'ui',
    labelEn: 'UI',
    labelZh: 'UI',
    items: [
      leaf('ui-overview', 'Overview', '总览', '/ui'),
      leaf('ui-desktop', 'Desktop', '桌面', '/ui-desktop'),
      leaf('ui-android', 'Android', 'Android', '/ui-android'),
      leaf('ui-harmony', 'Harmony', '鸿蒙', '/ui-harmony'),
      sharedSpa('ui-gallery', 'Gallery (shared)', 'Gallery（共享）', '/ui/gallery/index.html'),
      sharedSpa('ui-blocks', 'Blocks (shared)', 'Blocks（共享）', '/ui/blocks/index.html'),
      sharedSpa('ui-charts', 'Charts (shared)', 'Charts（共享）', '/ui/charts/index.html'),
      sharedSpa('ui-a2ui', 'A2UI Demo (shared)', 'A2UI 演示（共享）', '/ui/a2ui/index.html'),
      sharedSpa('ui-demos', 'Demo Apps (shared)', '应用演示（共享）', '/ui/demos/'),
    ],
  },
]

export const NAV_TOP_LEAVES: NavLeaf[] = [
  leaf('v05', 'v0.5 Release', 'v0.5 发布', '/v05'),
  leaf('apps', 'Apps', '应用', '/apps'),
  leaf('docs', 'Docs', '文档', '/docs/'),
  leaf('books', 'Tutorials', '教程', '/books/'),
]

// 右侧强调入口：在线体验。
export const NAV_HIGHLIGHT: NavLeaf = leaf('playground', 'Try Online', '在线体验', '/playground')

export const NAV_HOME: NavLeaf = leaf('home', 'Auto Language', 'Auto 语言', '/')

export interface NavItemView {
  key: string
  label: string
  href: string
  shared?: boolean
  highlight?: boolean
}

export function localize(item: { key: string; labelEn: string; labelZh: string; en: string; zh: string; shared?: boolean; highlight?: boolean }, zh: boolean): NavItemView {
  return {
    key: item.key,
    label: zh ? item.labelZh : item.labelEn,
    href: zh ? item.zh : item.en,
    shared: item.shared,
    highlight: item.highlight,
  }
}

/** 当前路由是否命中某个入口（用于当前位置反馈/aria-current）。 */
export function isRouteActive(href: string, path: string): boolean {
  const norm = normalizePath(path)
  const target = normalizePath(href)
  if (target === '/') return norm === '/'
  return norm === target || norm.startsWith(target.endsWith('/') ? target : target + '/')
}

/** 归一化：去 .html、去 index、收敛尾部斜杠（根除外）。 */
export function normalizePath(p: string): string {
  let out = p.split('?')[0].split('#')[0]
  out = out.replace(/\.html$/, '')
  out = out.replace(/\/index$/, '')
  if (out.length > 1) out = out.replace(/\/$/, '')
  return out === '' ? '/' : out
}

/**
 * 语言切换：按前缀规则换 locale，保留 fragment。
 * docs/books 双语 1:1、手工专题页全镜像（见基线报告 §5）；共享 SPA 双语同 URL。
 */
export function switchLocaleHref(path: string, toZh: boolean): string {
  const hash = path.includes('#') ? '#' + path.split('#')[1] : ''
  const clean = path.split('#')[0]
  if (toZh) {
    if (clean === '/') return '/zh/' + hash
    return '/zh' + clean + hash
  }
  if (clean === '/zh') return '/' + hash
  if (clean.startsWith('/zh/')) return clean.slice(3) + hash
  return clean + hash
}
