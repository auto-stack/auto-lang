import { defineConfig } from 'vitepress'
import { shared } from './config/shared'
import { en } from './config/en'
import { zh } from './config/zh'
import path from 'path'

export default defineConfig({
  ...shared,
  locales: {
    root: { label: 'English', ...en },
    zh: { label: '简体中文', ...zh },
  },
  vite: {
    resolve: {
      alias: {
        '@': __dirname,
        'auto-playground-vue': path.resolve(__dirname, '../../packages/auto-playground-vue/src/index.ts'),
      },
    },
    optimizeDeps: {
      include: [
        'highlight.js',
        '@codemirror/language',
        '@codemirror/state',
        '@codemirror/view',
        '@codemirror/commands',
        '@codemirror/theme-one-dark',
      ],
    },
  },
})
