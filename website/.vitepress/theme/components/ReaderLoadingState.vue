<template>
  <div class="reader-loading" role="status">
    <span class="rl-spinner" aria-hidden="true"></span>
    <span class="rl-label">{{ label }}</span>
  </div>
</template>

<script setup lang="ts">
// PLAN-718 T-05：阅读页轻量加载反馈（runner chunk 拉取等待态）。
// 纯展示组件——SSR 安全（无浏览器 API）；失败/重试态由 AutoFence 自行渲染。
defineProps<{
  /** 已本地化的加载文案。 */
  label: string
}>()
</script>

<style scoped>
.reader-loading {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  border: 1px dashed var(--vp-c-border, #313244);
  border-radius: var(--site-radius, 0.5rem);
  color: var(--vp-c-text-2, #a6adc8);
  font-size: 0.8rem;
}

.rl-spinner {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  border: 2px solid var(--vp-c-border, #313244);
  border-top-color: var(--vp-c-brand-1, #6366f1);
  border-radius: 50%;
  animation: rl-spin 0.8s linear infinite;
}

@keyframes rl-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  .rl-spinner {
    animation: none;
  }
}
</style>
