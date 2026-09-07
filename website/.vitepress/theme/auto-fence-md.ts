import type MarkdownIt from 'markdown-it'

/**
 * lang=auto 裸围栏 → <AutoFence> 包裹（Plan 582 T15，Playground 设计 §7）。
 *
 * 默认槽位保留原 shiki 高亮围栏（收起态还原原始外观）；code 经 encodeURIComponent
 * 编码传入组件 prop（HTML 属性安全）。带属性变体（```auto,ignore 等）不包裹——
 * 明示不可独立运行的示例维持普通代码块。与 build-playground-notes.mjs 的裸围栏
 * 采集规则保持同口径。
 */
export function autoFencePlugin(md: MarkdownIt) {
  const defaultFence = md.renderer.rules.fence!
  md.renderer.rules.fence = (tokens, idx, options, env, self) => {
    const info = (tokens[idx].info || '').trim()
    if (info === 'auto') {
      const encoded = encodeURIComponent(tokens[idx].content)
      return `<AutoFence code="${encoded}">${defaultFence(tokens, idx, options, env, self)}</AutoFence>`
    }
    return defaultFence(tokens, idx, options, env, self)
  }
}
