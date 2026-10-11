// User-provided preparation captures; original hashes: docs/reports/p758-autojade-capture-catalog.json.
export function autoJadeKnowledgeShots(zh: boolean) {
  const entries: [string, string, string, number, number][] = [
    ['p2-backlinks-dark.png', 'Backlinks', '反链', 1440, 900],
    ['p3-wikilink-dark.png', 'Page links', '双链', 1440, 900],
    ['p5-quick-open-dark.png', 'Quick open', '快速打开', 1440, 900],
    ['p6-dangling-create-dark.png', 'Create a missing page', '缺失页面建页', 1440, 900],
    ['p7-filetree-dark.png', 'Workspace tree', '文件树', 1933, 1245],
  ]
  return entries.map(([file, en, cn, width, height]) => ({
    src: `/apps/auto-jade/${file}`, label: zh ? cn : en, alt: `AutoJade · ${zh ? cn : en}`,
    caption: zh ? `AutoJade · ${cn} · 深色 · 产品手册示例库 · 用户提供 2026-10-10 · 发布准备态` : `AutoJade · ${en} · dark · product manual sample workspace · provided 2026-10-10 · release preparation`,
    width, height,
  }))
}
