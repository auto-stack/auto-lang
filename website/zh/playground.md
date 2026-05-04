---
layout: page
sidebar: false
---

<script setup>
import { ref } from 'vue'
const isLoading = ref(true)
</script>

<div class="playground-page">
  <div class="playground-header">
    <h1>Playground</h1>
    <p>在浏览器中编写、运行和转译 Auto 代码。需要运行 playground 后端服务器。</p>
  </div>

  <div class="iframe-wrapper" style="position: relative;">
    <div v-if="isLoading" class="iframe-loading">
      <span class="loading-spinner"></span>
      <p>正在加载 Playground...</p>
    </div>
    <iframe
      src="/playground/index.html"
      title="Auto Playground"
      frameborder="0"
      loading="lazy"
      @load="isLoading = false"
    />
  </div>

  <div class="playground-note">
    <p>
      <strong>需要后端服务器？</strong>
      Playground 需要运行 Auto playground 服务器。你可以在本地启动：
    </p>
    <pre><code>cargo run -p auto-playground</code></pre>
    <p>
      或者将后端与网站一起部署，并配置 API 端点。
    </p>
  </div>
</div>

<style scoped>
.playground-page {
  max-width: 1400px;
  margin: 0 auto;
  padding: 0 1rem 3rem;
}

.playground-header {
  text-align: center;
  padding: 1.5rem 0 1rem;
}

.playground-header h1 {
  font-size: 2rem;
  font-weight: 700;
  background: linear-gradient(120deg, #6366f1 30%, #a855f7 70%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin-bottom: 0.5rem;
}

.playground-header p {
  color: hsl(var(--muted-foreground));
  margin: 0;
}

.iframe-wrapper {
  border: 1px solid hsl(var(--border));
  border-radius: calc(var(--radius) + 4px);
  overflow: hidden;
  background: hsl(var(--card));
  height: 800px;
  position: relative;
}

.iframe-wrapper iframe {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  border: none;
}

.iframe-loading {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  background: hsl(var(--card));
  z-index: 1;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid hsl(var(--border));
  border-top-color: hsl(var(--primary));
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  display: inline-block;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.playground-note {
  margin-top: 1.5rem;
  padding: 1rem 1.25rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--muted) / 0.5);
}

.playground-note p {
  margin: 0 0 0.75rem;
  color: hsl(var(--muted-foreground));
  font-size: 0.875rem;
}

.playground-note p:last-child {
  margin-bottom: 0;
}

.playground-note pre {
  margin: 0.5rem 0;
  background: hsl(var(--background));
  border: 1px solid hsl(var(--border));
  border-radius: var(--radius);
  padding: 0.75rem 1rem;
}

.playground-note code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8125rem;
}

@media (max-width: 768px) {
  .iframe-wrapper {
    height: 600px;
  }
}
</style>
