import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// Plan 405 vue-ref: reference prototype for 023-realworld.
// Mirrors the auto codegen stack (Vue3 + Vite + Tailwind) so the .at port
// stays a 1:1 translation. Runs on port 3123 to avoid clashing with the auto
// project's 3023. Mock data only — no backend.
export default defineConfig({
  plugins: [vue()],
  resolve: { alias: { '@': resolve(__dirname, 'src') } },
  server: {
    port: Number(process.env.VUE_REF_PORT || 3123),
    open: false,
  },
})
