import type MarkdownIt from 'markdown-it'

/**
 * lang=auto 裸围栏 → <AutoFence> 包裹（Plan 582 T15，Playground 设计 §7）。
 *
 * 默认槽位保留原 shiki 高亮围栏（收起态还原原始外观）；code 经 encodeURIComponent
 * 编码传入组件 prop（HTML 属性安全）。带属性变体（```auto,ignore 等）不包裹——
 * 明示不可独立运行的示例维持普通代码块。与 build-playground-notes.mjs 的裸围栏
 * 采集规则保持同口径。
 *
 * PLAN-718 T-04：原生复制按钮标题按页面 locale 本地化。VitePress 的
 * codeCopyButtonTitle 由全局 markdown 渲染器一次性解析（locale 级 markdown
 * 配置不参与 SSG 渲染，实测 ZH 页仍输出 'Copy Code'）——此处围栏渲染时按
 * env.relativePath（SSG 注入，见 dist node chunk: env={relativePath,...}）前缀
 * 判定 zh 并改写标题；HTML 在构建期烘焙，客户端不再重渲染 markdown。
 */
export function autoFencePlugin(md: MarkdownIt) {
  const defaultFence = md.renderer.rules.fence!
  md.renderer.rules.fence = (tokens, idx, options, env, self) => {
    const info = (tokens[idx].info || '').trim()
    const html = defaultFence(tokens, idx, options, env, self)
    const relPath = String((env as { relativePath?: string }).relativePath || '')
    const title = relPath.startsWith('zh/') ? '复制代码' : 'Copy Code'
    const localized = html.replace(
      /<button title="Copy Code" class="copy">/,
      `<button title="${title}" class="copy">`,
    )
    if (info === 'auto') {
      const encoded = encodeURIComponent(tokens[idx].content)
      return `<AutoFence code="${encoded}">${localized}</AutoFence>`
    }
    return localized
  }
}
