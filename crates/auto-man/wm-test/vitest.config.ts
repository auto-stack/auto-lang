// Plan 516: WM 资产测试位配置。被测物 = 生成器资产 ../assets/wm/*（源
// 级测试——生成物勿测副本）。wm 资产里 './remote-renderer/index.ts' 在
// 生成项目由 wm_assets.rs 物化（packages/drawlist-renderer/src 的编译期
// 拷贝）；此处经 resolveId 直指渲染器包源——同一份代码，零拷贝漂移。
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

const RENDERER = fileURLToPath(
  new URL('../../../packages/drawlist-renderer/src/index.ts', import.meta.url),
)

export default defineConfig({
  plugins: [
    vue(),
    {
      name: 'wm-remote-renderer-src',
      enforce: 'pre',
      resolveId(source) {
        if (source === './remote-renderer/index.ts') return RENDERER
        return null
      },
    },
  ],
  test: {
    environment: 'jsdom',
    globals: true,
  },
})
