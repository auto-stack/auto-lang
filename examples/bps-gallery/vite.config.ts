import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
// Allow Vite to serve the real bp packages living at <repo>/blueprints via ?raw
// imports (the gallery reads spec.md / reference/*.at / gotchas.md directly).
const repoRoot = path.resolve(here, '../..')

export default defineConfig({
  plugins: [vue()],
  server: { fs: { allow: [repoRoot] } },
})
