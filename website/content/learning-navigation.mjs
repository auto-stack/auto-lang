/**
 * PLAN-718 T-02：学习导航作者数据（单一来源）。
 *
 * 消费方：
 *  - website/scripts/prepare-content.js —— 校验每个 href 都落在真实生成页上，
 *    并在 docs/books hub 根页输出可索引 Markdown 摘要 + <LearningHub> 组件调用。
 *  - website/.vitepress/theme/components/LearningHub.vue —— SSR 渲染入口卡片。
 *
 * 规则（PLAN-718 §5.1）：
 *  - 只描述标题、受众、说明、真实内容路径与顺序；不复制正文、不编造难度/时长。
 *  - 八本书的说明沿用 generateBooksIndex 既有文案（"现有书的说明"），选择依据
 *    由各书自身定位（"与 Rust 对比学习"等）推导，不虚构先修要求。
 *  - href 一律为不带 locale 前缀的真实站点路径（EN 视角）；zh 变体由 localeHref()
 *    按前缀规则生成（docs/books 双语 1:1，手工专题页镜像，与 navigation.ts 同序）。
 */

/**
 * @typedef {{ en: string, zh: string }} Bi
 * @typedef {{ key: string, title: Bi, desc: Bi, href: string }} HubCard
 * @typedef {{ key: string, title: Bi, items: Array<{ label: Bi, desc?: Bi, href: string }> }} TaskGroup
 * @typedef {{ id: string, featured?: boolean, background: 'new'|'rust'|'typescript'|'c'|'python'|'types',
 *             title: Bi, blurb: Bi, audience: Bi, href: string, entry: string }} BookEntry
 */

/** locale 前缀规则：zh 变体 = '/zh' + path（'/' 例外）。与 theme/data/navigation.ts switchLocaleHref 同口径。 */
export function localeHref(path, zh) {
  if (!zh) return path
  if (path === '/') return '/zh/'
  return '/zh' + path
}

// ---------------------------------------------------------------- docs hub

/** @type {{ intro: Bi, primary: [HubCard, HubCard], cards: HubCard[], tasks: TaskGroup[] }} */
export const DOCS_HUB = {
  intro: {
    en: 'Start with a guided tour or the syntax reference, then pick a task below. Internal design documents stay available through the sidebar and search.',
    zh: '从引导巡礼或语法参考开始，再按下方任务选择。内部设计文档仍可经侧栏与搜索找到。',
  },
  // 主要行动：开始学习 + 查语法，高于内部设计列表。
  primary: [
    {
      key: 'tour',
      title: { en: 'Start learning: the Auto Tour', zh: '开始学习：Auto 巡礼' },
      desc: {
        en: 'A hands-on guided tour — hello world, types, functions, control flow and more.',
        zh: '动手式引导教程——Hello World、类型、函数、控制流等。',
      },
      href: '/docs/tour/ch01-hello',
    },
    {
      key: 'syntax',
      title: { en: 'Look up the syntax', zh: '查阅语法' },
      desc: {
        en: 'The Auto syntax reference — grammar at a glance.',
        zh: 'Auto 语法参考——一页速查语法。',
      },
      href: '/docs/syntax',
    },
  ],
  cards: [
    {
      key: 'spec',
      title: { en: 'Language specification', zh: '语言规范' },
      desc: {
        en: 'The full language specification, batch by batch.',
        zh: '按批次推进的完整语言规范。',
      },
      href: '/docs/language/specification',
    },
    {
      key: 'features',
      title: { en: 'Feature deep dives', zh: '特性深入' },
      desc: {
        en: 'Actor concurrency, memory safety, comptime metaprogramming and more.',
        zh: 'Actor 并发、内存安全、编译期元编程等。',
      },
      href: '/docs/features/actor-concurrency',
    },
    {
      key: 'v05',
      title: { en: 'What’s new in v0.5', zh: 'v0.5 新特性' },
      desc: {
        en: 'The current release, with screenshots and demos.',
        zh: '当前版本发布说明，含截图与演示。',
      },
      href: '/v05/',
    },
  ],
  tasks: [
    {
      key: 'tutorials',
      title: { en: 'Work through a tutorial', zh: '跟着教程做' },
      items: [
        { label: { en: 'Autogen tutorial', zh: 'Autogen 教程' }, href: '/docs/tutorials/autogen-tutorial' },
        { label: { en: 'Atom API guide', zh: 'Atom API 指南' }, href: '/docs/tutorials/atom-api-guide' },
        { label: { en: 'for loop guide', zh: 'for 循环指南' }, href: '/docs/tutorials/for-loop-guide' },
        { label: { en: 'Debugger tutorial', zh: '调试器教程' }, href: '/docs/tutorials/debugger-tutorial' },
      ],
    },
    {
      key: 'guides',
      title: { en: 'Solve a concrete task', zh: '解决具体任务' },
      items: [
        { label: { en: 'Choose a mode', zh: '选择运行模式' }, href: '/docs/guides/mode-selection-guide' },
        { label: { en: 'Migrate from another language', zh: '从其他语言迁移' }, href: '/docs/guides/migration-guide' },
        { label: { en: 'Use FFI', zh: '使用 FFI' }, href: '/docs/guides/ffi-usage-guide' },
        { label: { en: 'Cache builds with Autocache', zh: '用 Autocache 缓存构建' }, href: '/docs/guides/autocache-guide' },
      ],
    },
    {
      key: 'ui',
      title: { en: 'Build UIs with Auto', zh: '用 Auto 构建 UI' },
      items: [
        { label: { en: 'Desktop UI overview', zh: '桌面 UI 总览' }, href: '/ui-desktop' },
        { label: { en: 'UI gallery (shared demo)', zh: 'UI 画廊（共享演示）' }, href: '/ui/gallery/index.html' },
      ],
    },
    {
      key: 'examples',
      title: { en: 'Read example projects', zh: '阅读示例项目' },
      items: [
        { label: { en: 'Mixed mode project', zh: '混合模式项目' }, href: '/docs/examples/mixed-mode-project' },
        { label: { en: 'Streaming component protocol', zh: '流式组件协议' }, href: '/docs/examples/streaming-component-protocol' },
      ],
    },
  ],
}

// ---------------------------------------------------------------- books hub

/**
 * 八本书。background 为读者背景分组；featured 为主书（教程根页优先展示）。
 * entry 为每本书第一章（已验证存在，zh 侧有 .cn.md 或按既定 EN 回退）。
 * @type {BookEntry[]}
 */
export const BOOKS = [
  {
    id: 'tapl',
    featured: true,
    background: 'new',
    title: { en: 'The Auto Programming Language', zh: 'Auto 编程语言（主书）' },
    blurb: {
      en: 'The main Auto tutorial — a comprehensive introduction to the language.',
      zh: 'Auto 主书——全面的语言介绍。',
    },
    audience: {
      en: 'New to Auto? Start here.',
      zh: '刚接触 Auto？从这里开始。',
    },
    href: '/books/tapl/',
    entry: '/books/tapl/ch00-introduction',
  },
  {
    id: 'rust',
    background: 'rust',
    title: { en: 'Auto vs Rust', zh: 'Auto 版 Rust Book' },
    blurb: {
      en: 'Learn Auto by comparing it with Rust.',
      zh: '通过与 Rust 比较来学习 Auto。',
    },
    audience: {
      en: 'Coming from Rust.',
      zh: '有 Rust 背景。',
    },
    href: '/books/rust/',
    entry: '/books/rust/ch00-introduction',
  },
  {
    id: 'typescript',
    background: 'typescript',
    title: { en: 'Auto vs TypeScript', zh: 'Auto 版 TypeScript Handbook' },
    blurb: {
      en: 'A handbook for TypeScript developers learning Auto.',
      zh: '面向 TypeScript 开发者的 Auto 手册。',
    },
    audience: {
      en: 'Coming from TypeScript.',
      zh: '有 TypeScript 背景。',
    },
    href: '/books/typescript/',
    entry: '/books/typescript/ch00-introduction',
  },
  {
    id: 'typescript-deepdive',
    background: 'types',
    title: { en: 'Auto vs TypeScript DeepDive', zh: 'Auto 版 TypeScript DeepDive' },
    blurb: {
      en: 'Deep dive into Auto’s type system compared to TypeScript.',
      zh: '深入比较 Auto 和 TypeScript 的类型系统。',
    },
    audience: {
      en: 'Want the type-system details.',
      zh: '想深入类型系统细节。',
    },
    href: '/books/typescript-deepdive/',
    entry: '/books/typescript-deepdive/ch00-why-types',
  },
  {
    id: 'little-c',
    background: 'c',
    title: { en: 'Auto vs The Little Book of C', zh: 'Auto 版 The Little Book of C' },
    blurb: {
      en: 'A gentle introduction to Auto through C concepts.',
      zh: '通过 C 语言概念温和地介绍 Auto。',
    },
    audience: {
      en: 'Coming from C, or want a gentle pace.',
      zh: '有 C 背景，或想要平缓节奏。',
    },
    href: '/books/little-c/',
    entry: '/books/little-c/ch00-getting-started',
  },
  {
    id: 'modern-c',
    background: 'c',
    title: { en: 'Auto vs Modern C', zh: 'Auto 版 Modern C' },
    blurb: {
      en: 'Modern systems programming with Auto and C.',
      zh: '使用 Auto 和 C 进行现代系统编程。',
    },
    audience: {
      en: 'Doing systems programming.',
      zh: '面向系统编程场景。',
    },
    href: '/books/modern-c/',
    entry: '/books/modern-c/ch00-introduction',
  },
  {
    id: 'byte-of-python',
    background: 'python',
    title: { en: 'A Byte of Auto', zh: 'Auto 版 A Byte of Python' },
    blurb: {
      en: 'A beginner-friendly tutorial inspired by “A Byte of Python”.',
      zh: '受《A Byte of Python》启发的初学者友好教程。',
    },
    audience: {
      en: 'First language, or coming from Python.',
      zh: '第一门语言，或有 Python 背景。',
    },
    href: '/books/byte-of-python/',
    entry: '/books/byte-of-python/ch00-preface',
  },
  {
    id: 'think-python',
    background: 'python',
    title: { en: 'Think Auto', zh: 'Auto 版 Think Python' },
    blurb: {
      en: 'Computational thinking with Auto, based on “Think Python”.',
      zh: '基于《Think Python》的 Auto 计算思维。',
    },
    audience: {
      en: 'Learning to think like a programmer.',
      zh: '想建立编程思维方式。',
    },
    href: '/books/think-python/',
    entry: '/books/think-python/ch00-preface',
  },
]
