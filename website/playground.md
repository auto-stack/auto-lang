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
    <p>Write, run, and transpile Auto code in your browser. Requires a running playground server.</p>
  </div>

  <AutoPlayground api-url="" height="700px" />

  <div class="playground-note">
    <p>
      <strong>Need a backend?</strong>
      The playground requires a running Auto playground server. You can start one locally:
    </p>
    <pre><code>cargo run -p auto-playground</code></pre>
    <p>
      Or deploy the backend alongside this website and configure the API endpoint.
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
