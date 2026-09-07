---
layout: page
sidebar: false
---

<div class="playground-page">
  <div class="playground-header">
    <h1>Playground</h1>
    <p>浏览仓内语料笔记——VM 金样、书页围栏、Demo 示例；运行与转译需本地后端。</p>
  </div>

  <NotesExplorer />

  <div id="backend" class="playground-note">
    <p>
      <strong>需要后端服务器？</strong>
      浏览与编辑笔记无需后端。要运行或转译代码，可在本地启动 playground 服务器：
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
</style>
