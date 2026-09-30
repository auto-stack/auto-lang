<script setup lang="ts">
// PLAN-715 T-04：应用专题共享骨架。
// 章节：简介/状态/入口(hero slot) → 实图(evidence) → 核心用例(showcase) →
// 开始使用(start) → 已有边界/来源(notes)。
// apps.md 目录页与 automusk/autodown/autoui 专题经此布局共享排版；
// 页面差异经插槽传入，不在此发明能力/下载入口。AutoShell 专题保持其
// AutoShellLanding 自有布局（仅接共享视觉令牌），不经本组件。
import EvidenceImage from './EvidenceImage.vue'
import { computed, useSlots } from 'vue'
import { useRoute } from 'vitepress'

const props = withDefaults(defineProps<{
  /** 应用名（无障碍命名/返回链接文案共用） */
  appName: string
  badge?: string
  status?: 'beta' | 'alpha' | 'stable'
  /** 实图（单张主图，lazy；首页 hero 已有图时可省） */
  image?: string
  imageAlt?: string
  imageCaption?: string
  imageWidth?: number
  imageHeight?: number
  /** 展开完整图/来源的文案 */
  zoomLabel?: string
  closeLabel?: string
  originalLabel?: string
  /** 返回目录与发布页入口（缺省按 locale 推导） */
  backHref?: string
  backLabel?: string
}>(), {
  status: 'beta',
  zoomLabel: 'View full size',
  closeLabel: 'Close',
  originalLabel: 'Open original',
})

const route = useRoute()
const zh = computed(() => route.path.startsWith('/zh'))
const slots = useSlots()

const backTarget = computed(() => props.backHref ?? (zh.value ? '/zh/apps' : '/apps'))
const backLabel = computed(() => props.backLabel ?? (zh.value ? '全部应用' : 'All applications'))
const statusLabel = computed(() =>
  props.status === 'beta' ? 'Beta' : props.status === 'alpha' ? 'Alpha' : (zh.value ? '可用' : 'Available'))
</script>

<template>
  <div class="app-landing">
    <section class="app-hero" :aria-labelledby="'app-title-' + appName">
      <a class="app-back" :href="backTarget">← {{ backLabel }}</a>
      <div class="app-hero-head">
        <div v-if="badge" class="app-badge">{{ badge }}</div>
        <span v-if="status" class="app-status" :class="status">{{ statusLabel }}</span>
      </div>
      <h1 :id="'app-title-' + appName" class="app-title"><slot name="title">{{ appName }}</slot></h1>
      <p class="app-intro"><slot name="intro" /></p>
      <div v-if="slots.actions" class="app-actions"><slot name="actions" /></div>
    </section>

    <section v-if="image || slots.evidence" class="app-evidence" :aria-label="zh ? '应用界面' : 'Application interface'">
      <slot name="evidence">
        <EvidenceImage
          :src="image!"
          :alt="imageAlt ?? appName"
          :caption="imageCaption ?? ''"
          :zoom-label="zoomLabel"
          :close-label="closeLabel"
          :original-label="originalLabel"
          :width="imageWidth"
          :height="imageHeight"
        />
      </slot>
    </section>

    <section v-if="slots.showcase" class="app-showcase"><slot name="showcase" /></section>
    <section v-if="slots.start" class="app-start" :aria-label="zh ? '开始使用' : 'Get started'"><slot name="start" /></section>
    <section v-if="slots.notes" class="app-notes" :aria-label="zh ? '边界与来源' : 'Boundaries and sources'"><slot name="notes" /></section>
  </div>
</template>

<style scoped>
.app-landing {
  max-width: var(--site-max-width, 1200px);
  margin: 0 auto;
  padding: 0 2rem;
}

.app-hero {
  padding: 3.5rem 0 2rem;
  max-width: var(--site-text-width, 720px);
}

.app-back {
  display: inline-block;
  margin-bottom: 1.25rem;
  color: hsl(var(--muted-foreground));
  font-size: 0.85rem;
  text-decoration: none;
}

.app-back:hover {
  color: hsl(var(--foreground));
  text-decoration: underline;
}

.app-back:focus-visible {
  outline: 3px solid var(--vp-c-brand-1);
  outline-offset: 2px;
}

.app-hero-head {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin-bottom: 0.75rem;
}

.app-badge {
  display: inline-flex;
  padding: 0.3rem 0.75rem;
  border-radius: 9999px;
  background: color-mix(in srgb, var(--page-accent-1, #6366f1) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--page-accent-1, #6366f1) 28%, transparent);
  color: var(--page-accent-1, #6366f1);
  font-size: 0.78rem;
  font-weight: 600;
}

.app-status {
  padding: 0.2rem 0.65rem;
  border-radius: 9999px;
  font-size: 0.72rem;
  font-weight: 700;
}

.app-status.beta { background: rgba(59, 130, 246, 0.15); color: #3b82f6; }
.app-status.alpha { background: rgba(139, 92, 246, 0.15); color: #8b5cf6; }
.app-status.stable { background: rgba(34, 197, 94, 0.15); color: #22c55e; }

.app-title {
  margin: 0 0 1rem;
  font-size: clamp(2rem, 4vw, 2.8rem);
  font-weight: 800;
  line-height: 1.15;
  letter-spacing: -0.02em;
  color: hsl(var(--foreground));
}

.app-intro {
  margin: 0 0 1.5rem;
  font-size: 1.1rem;
  line-height: 1.8;
  color: hsl(var(--muted-foreground));
}

.app-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
}

.app-evidence {
  padding: 1.5rem 0 2.5rem;
}

.app-showcase {
  padding: 1rem 0 2.5rem;
  border-top: 1px solid hsl(var(--border));
}

.app-start {
  padding: 2.5rem 0;
  border-top: 1px solid hsl(var(--border));
}

.app-notes {
  padding: 2.5rem 0 3.5rem;
  border-top: 1px solid hsl(var(--border));
}

@media (max-width: 640px) {
  .app-landing {
    padding: 0 1.25rem;
  }

  .app-hero {
    padding: 2.5rem 0 1.5rem;
  }
}
</style>
