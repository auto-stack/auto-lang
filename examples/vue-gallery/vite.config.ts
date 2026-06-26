import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// Consume @auto-ui/widgets as a real package (file: dep). Its exports map
// serves registry/* and styles.css; pnpm places the package's runtime deps
// (clsx, tailwind-merge, class-variance-authority, reka-ui) where they resolve.
export default defineConfig({
  plugins: [vue()],
})
