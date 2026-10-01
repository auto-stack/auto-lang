/**
 * PLAN-718 T-05：runner 入口 shim（首次加载）。
 *
 * 动态导入失败的 URL 会被 HTML module map 缓存（同 URL 重试秒拒；query 变体在
 * build 解析时被去重抹平、?url 对 .ts 是资产拷贝——均实测不可用）。因此准备真实
 * 的多入口 shim：不同文件/内容 = 不同模块 id = 不同入口 chunk = 不同 module-map
 * 键；重型依赖 chunk 跨 shim 共享（__runnerAttempt 区分防止 rollup 合并同构 facade）。
 */
export { SnippetRunner } from 'auto-playground-vue'
export const __runnerAttempt = 0
