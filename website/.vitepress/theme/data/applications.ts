// Product descriptions, rather than framework demos, define the Apps overview.
export function applicationCopy(zh: boolean) {
  return zh ? {
    title: 'Auto 应用',
    lead: '编辑代码与文档、处理命令、推进开发任务、组织个人知识。四个主应用围绕这些工作展开；系统应用与 Demo 则展示桌面里的日常工具和具体交互。',
    mainTitle: '四个主应用',
    mainLead: '每个应用有自己的使用场景，也会复用 Auto 语言、AutoUI 和生态中的公共能力。它们的功能与完成度分别说明。',
    details: '了解这个应用',
    apps: [
      { key: 'autoedit', name: 'AutoEdit', role: '代码与文本', summary: '轻量文本工作台，用于浏览、编辑和比较文件。以原生桌面体验为主要方向，把代码审阅和小步修改放在前面。', uses: ['项目文件与多标签编辑', '查找、替换与跨文件搜索', '文件、目录与缓冲区差异比较'] },
      { key: 'autoshell', name: 'AutoShell', role: '命令与脚本', summary: '将日常 Shell、按字段处理的数据管道和 AutoScript 放在同一个会话中。既可交互使用，也能从命令行调用。', uses: ['连续的命令会话', '文件与 JSON 的结构化查询', '多行脚本和命令行自动化'] },
      { key: 'automusk', name: 'AutoMusk', role: 'AI 辅助开发', summary: '围绕 Plan 与 Spec 组织开发任务的 Coding Agent。让需求、执行过程、验证结果和项目知识保持可追溯。', uses: ['项目对话与工具调用', '规划、执行、复审与知识沉淀', '模型、角色与技能配置'] },
      { key: 'jadeedit', name: 'JadeEdit', role: '文档与知识', summary: '面向 AutoDown 文档和本地知识库的编辑器。将写作、页面链接、搜索与目录组织连起来，逐步形成个人知识工作台。', uses: ['文档编辑与多页面浏览', '链接、反链、标签与检索', '页面组织与本地草稿恢复'] },
    ],
    relatedTitle: '应用与生态',
    related: [
      { href: '/os', name: 'AutoOS', text: '了解应用所在的桌面、系统服务，以及 LaOS 与 OS over OS 的关系。' },
      { href: '/ui', name: 'AutoUI', text: '了解这些界面如何由 Auto 源码构建，以及组件与跨端运行方式。' },
      { href: '/ai', name: 'AutoAI', text: '了解应用、Agent 和模型服务之间的公共 AI 架构。' },
    ],
    resources: '相关资料',
    resourceLead: 'AutoDown 是文档与编辑器的基础；AutoUI 示例页提供界面开发资料。它们与四个主应用分别介绍。',
    resourceLinks: [['/apps/autodown/', 'AutoDown 与 Jade Garden'], ['/apps/autoui/', 'AutoUI 示例资料']],
    date: '介绍依据当前项目进展整理 · 2026-10-01',
  } : {
    title: 'Auto applications',
    lead: 'Edit code and documents, work with commands, carry out development tasks, and organize personal knowledge. Four main applications address these activities; system apps and demos introduce everyday desktop tools and their interactions.',
    mainTitle: 'Four main applications',
    mainLead: 'Each application has its own use cases while sharing Auto, AutoUI, and common ecosystem capabilities. Features and readiness are described individually.',
    details: 'Explore this application',
    apps: [
      { key: 'autoedit', name: 'AutoEdit', role: 'Code and text', summary: 'A lightweight text workspace for browsing, editing, and comparing files. Native desktop use, code review, and small focused edits guide its development.', uses: ['Project files and multiple editing tabs', 'Find, replace, and search across files', 'File, directory, and buffer comparisons'] },
      { key: 'autoshell', name: 'AutoShell', role: 'Commands and scripts', summary: 'Everyday shell commands, field-based data pipelines, and AutoScript in one session. Use it interactively or call it from the command line.', uses: ['A persistent command session', 'Structured file and JSON queries', 'Multiline scripts and CLI automation'] },
      { key: 'automusk', name: 'AutoMusk', role: 'AI-assisted development', summary: 'A coding agent that organizes development around Plans and Specs, keeping requirements, execution, verification, and project knowledge traceable.', uses: ['Project conversations and tool calls', 'Planning, execution, review, and consolidation', 'Model, role, and skill configuration'] },
      { key: 'jadeedit', name: 'JadeEdit', role: 'Documents and knowledge', summary: 'An editor for AutoDown documents and local knowledge bases. Writing, page links, search, and directory organization form a developing personal knowledge workspace.', uses: ['Document editing and multiple pages', 'Links, backlinks, tags, and search', 'Page organization and local draft recovery'] },
    ],
    relatedTitle: 'Applications in the ecosystem',
    related: [
      { href: '/os', name: 'AutoOS', text: 'The desktop and system services, and the relationship between LaOS and OS over OS.' },
      { href: '/ui', name: 'AutoUI', text: 'How Auto sources build interfaces, with components and different rendering paths.' },
      { href: '/ai', name: 'AutoAI', text: 'The shared AI architecture connecting applications, agents, and model services.' },
    ],
    resources: 'Related resources',
    resourceLead: 'AutoDown supplies document and editor foundations. AutoUI examples offer interface development material. Both have their own introductions alongside the four main applications.',
    resourceLinks: [['/apps/autodown/', 'AutoDown and Jade Garden'], ['/apps/autoui/', 'AutoUI example resources']],
    date: 'Descriptions reflect current project progress · 2026-10-01',
  }
}
