<script setup lang="ts">
import { computed } from 'vue'
import { useData } from 'vitepress'
import ScreenshotGallery from './ScreenshotGallery.vue'
import { desktopShots, desktopSteps } from '../data/desktop-showcase'

const props = withDefaults(defineProps<{ preview?: boolean; embedded?: boolean }>(), { preview: false, embedded: false })
const { lang } = useData()
const zh = computed(() => lang.value.startsWith('zh'))
const shots = computed(() => props.preview ? desktopShots(zh.value).slice(0, 2) : desktopShots(zh.value))
const steps = computed(() => desktopSteps(zh.value))
</script>

<template>
  <section id="desktop-showcase" class="desktop-showcase" :class="{ embedded }">
    <template v-if="!embedded">
      <h2>{{ zh ? '虚拟桌面：从桌面到多窗口' : 'Virtual desktop: from the desktop to multiple windows' }}</h2>
      <p class="desktop-intro">{{ zh ? '先从没有展开应用窗口的桌面看起，再打开应用、摆放窗口。下面记录的是 AutoOS 虚拟桌面的实际画面。' : 'Begin with the desktop, with no expanded app windows, then open apps and arrange their windows. These are actual captures of the AutoOS virtual desktop.' }}</p>
    </template>
    <ol v-if="!preview" class="desktop-steps">
      <li v-for="(step, i) in steps" :key="step.title">
        <span class="step-number" aria-hidden="true">{{ String(i + 1).padStart(2, '0') }}</span>
        <div><h3>{{ step.title }}</h3><p>{{ step.desc }}</p></div>
      </li>
    </ol>
    <ScreenshotGallery :shots="shots" :group-label="zh ? '虚拟桌面实际截图' : 'Virtual desktop captures'" :zoom-label="zh ? '放大查看' : 'View full size'" :close-label="zh ? '关闭' : 'Close'" :original-label="zh ? '打开原图' : 'Open original'" />
    <p class="capture-source">{{ zh ? 'AutoOS 虚拟桌面实际截图 · 2026-10-01。保留完整原图；应用中的内容为捕获时的演示状态。' : 'Actual AutoOS virtual desktop captures · 2026-10-01. Full original images are preserved; app content shows the demonstration state at capture time.' }}</p>
    <a v-if="preview" class="desktop-more" :href="(zh ? '/zh' : '') + '/autoos/#desktop-showcase'">{{ zh ? '继续查看 Launcher、多窗口和工作场景 →' : 'Continue to Launcher, multiple windows and the workspace →' }}</a>
  </section>
</template>

<style scoped>
.desktop-showcase { max-width: var(--site-max-width, 1200px); margin: 0 auto; padding: 2.5rem 1.5rem; min-width: 0; }
.desktop-showcase.embedded { padding: 0; }
h2 { font-size: clamp(1.5rem, 3vw, 2rem); line-height: 1.3; font-weight: 700; margin-bottom: 1rem; }
.desktop-intro { max-width: var(--site-text-width, 720px); line-height: 1.75; color: hsl(var(--muted-foreground)); margin-bottom: 1.5rem; }
.desktop-steps { list-style: none; padding: 0; margin: 0 0 2rem; display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 1.5rem 2rem; }
.desktop-steps li { display: flex; gap: 1rem; min-width: 0; }
.step-number { font-size: 0.875rem; font-weight: 700; color: var(--vp-c-brand-1); padding-top: 0.15rem; }
h3 { font-weight: 650; font-size: 1.05rem; margin: 0 0 0.4rem; }
.desktop-steps p { line-height: 1.7; color: hsl(var(--muted-foreground)); margin: 0; }
.capture-source { font-size: 0.8rem; line-height: 1.6; color: hsl(var(--muted-foreground)); margin-top: 1rem; }
.desktop-more { display: inline-block; padding: 0.75rem 0; min-height: 44px; color: var(--vp-c-brand-1); font-weight: 600; }
.desktop-more:focus-visible { outline: 3px solid var(--vp-c-brand-1); outline-offset: 3px; }
@media (max-width: 640px) { .desktop-steps { grid-template-columns: 1fr; gap: 1.25rem; } }
</style>
