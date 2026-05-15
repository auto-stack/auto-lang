import { defineConfig, type Plugin } from 'vitepress'
import { shared } from './config/shared'
import { en } from './config/en'
import { zh } from './config/zh'

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
          const spaDirs = ['/ui/gallery', '/ui/blocks', '/ui/charts', '/ui/a2ui']
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
  locales: {
    root: { label: 'English', ...en },
    zh: { label: '简体中文', ...zh },
  },
  vite: {
    plugins: [spaRewrite()],
    resolve: {
      alias: {
        '@': __dirname,
      },
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
