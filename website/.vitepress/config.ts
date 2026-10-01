import path from 'path'
import { fileURLToPath } from 'url'
const __root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../')
import { defineConfig, type Plugin } from 'vitepress'
import { shared } from './config/shared'
import { en } from './config/en'
import { zh } from './config/zh'
import { autoFencePlugin } from './theme/auto-fence-md'

// SPA pages (Gallery, Charts, A2UI, Blocks) are served from public/ui/*/index.html.
// When the browser navigates to the directory URL, VitePress's SPA fallback
// returns the VitePress shell HTML. This middleware serves the SPA's index.html instead.
function spaRewrite(): Plugin {
  return {
    name: 'spa-rewrite',
    enforce: 'pre',
    configureServer(server) {
      server.middlewares.use((req: any, res: any, next: any) => {
        if (req.url) {
          // Only rewrite exact directory URLs, NOT sub-resources (assets/*.js, assets/*.css)
          const urlPath = req.url.split('?')[0]
          const spaDirs = ['/ui/gallery', '/ui/blocks', '/ui/charts', '/ui/a2ui', '/ui/demos']
          for (const dir of spaDirs) {
            if (urlPath === dir || urlPath === dir + '/') {
              req.url = dir + '/index.html'
              break
            }
          }
        }
        next()
      })
    },
  }
}

export default defineConfig({
  ...shared,
  markdown: {
    // ```auto 裸围栏后处理 → <AutoFence>（▶ Run 原地展开/收起还原；Plan 582 T15）。
    config(md) {
      md.use(autoFencePlugin)
    },
  },
  locales: {
    root: { label: 'English', ...en },
    zh: { label: '简体中文', ...zh },
  },
  vite: {
    plugins: [spaRewrite()],
    resolve: {
      alias: {
        '@': __dirname,
        // PLAN-718 T-05：lang/auto 子路径别名必须排在包别名前（前缀匹配）——
        // CodeView/ScriptShipView 只需 autoLanguage（StreamLanguage 小模块），
        // 经包 index 会拖入全包（NotesExplorer/PlaygroundCard 等）。
        'auto-playground-vue/lang/auto': path.resolve(__root, 'packages/auto-playground-vue/src/lang/auto.ts'),
        'auto-playground-vue': path.resolve(__root, 'packages/auto-playground-vue/src/index.ts'),
      },
      // PLAN-718 T-05：auto-playground-vue 有自己的 node_modules（@codemirror/*、
      // highlight.js、vue）——不去重会把两份 @codemirror/state 打进同一页面，
      // instanceof 失效（"Unrecognized extension value" 实测）。强制从 website
      // 根解析单一实例。
      dedupe: [
        'vue',
        'lucide-vue-next',
        'highlight.js',
        '@codemirror/state',
        '@codemirror/view',
        '@codemirror/language',
        '@codemirror/commands',
        '@codemirror/theme-one-dark',
        '@lezer/highlight',
      ],
    },
    server: {
      proxy: {
        '/api': {
          target: 'http://localhost:3030',
          changeOrigin: true,
        },
      },
    },
  },
})
