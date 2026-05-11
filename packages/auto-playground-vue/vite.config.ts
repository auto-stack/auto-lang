import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'AutoPlaygroundVue',
      fileName: 'auto-playground-vue',
    },
    rollupOptions: {
      external: [
        'vue',
        '@codemirror/commands',
        '@codemirror/language',
        '@codemirror/state',
        '@codemirror/theme-one-dark',
        '@codemirror/view',
        'highlight.js',
      ],
      output: {
        globals: {
          vue: 'Vue',
        },
      },
    },
  },
})
