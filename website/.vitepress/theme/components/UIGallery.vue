<template>
  <div class="ui-gallery">
    <!-- Hero -->
    <div class="gallery-hero">
      <h1 class="gallery-title">{{ t.title }}</h1>
      <p class="gallery-desc">{{ t.desc }}</p>
    </div>

    <!-- Backend Selector -->
    <div class="backend-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        class="tab-btn"
        :class="{ active: activeTab === tab.key }"
        @click="activeTab = tab.key"
      >
        <span class="tab-icon">{{ tab.icon }}</span>
        <span class="tab-label">{{ tab.label }}</span>
        <span v-if="tab.badge" class="tab-badge">{{ tab.badge }}</span>
      </button>
    </div>

    <!-- Web Panel -->
    <div v-if="activeTab === 'web'" class="panel">
      <div class="panel-header">
        <div class="panel-info">
          <h3>{{ t.webTitle }}</h3>
          <p>{{ t.webDesc }}</p>
        </div>
        <a :href="webUrl" target="_blank" class="open-link">
          {{ t.openNew }} ↗
        </a>
      </div>
      <div class="iframe-wrapper">
        <iframe
          :src="webUrl"
          :title="t.webTitle"
          frameborder="0"
          loading="lazy"
          allow="clipboard-write"
        />
      </div>
    </div>

    <!-- Desktop Panel -->
    <div v-else-if="activeTab === 'desktop'" class="panel placeholder-panel">
      <div class="placeholder-content">
        <div class="placeholder-icon">🖥️</div>
        <h3>{{ t.desktopTitle }}</h3>
        <p>{{ t.desktopDesc }}</p>
        <div class="tech-stack">
          <span class="tech-tag">Tauri</span>
          <span class="tech-tag">Winit</span>
          <span class="tech-tag">LVGL</span>
        </div>
      </div>
    </div>

    <!-- Mobile Panel -->
    <div v-else-if="activeTab === 'mobile'" class="panel placeholder-panel">
      <div class="placeholder-content">
        <div class="placeholder-icon">📱</div>
        <h3>{{ t.mobileTitle }}</h3>
        <p>{{ t.mobileDesc }}</p>
        <div class="tech-stack">
          <span class="tech-tag">ArkTS</span>
          <span class="tech-tag">Jetpack Compose</span>
          <span class="tech-tag">SwiftUI</span>
        </div>
      </div>
    </div>

    <!-- How it works -->
    <div class="how-it-works">
      <h2>{{ t.howTitle }}</h2>
      <div class="steps">
        <div class="step">
          <div class="step-num">1</div>
          <h4>{{ t.step1Title }}</h4>
          <p>{{ t.step1Desc }}</p>
        </div>
        <div class="step-arrow">→</div>
        <div class="step">
          <div class="step-num">2</div>
          <h4>{{ t.step2Title }}</h4>
          <p>{{ t.step2Desc }}</p>
        </div>
        <div class="step-arrow">→</div>
        <div class="step">
          <div class="step-num">3</div>
          <h4>{{ t.step3Title }}</h4>
          <p>{{ t.step3Desc }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps<{
  locale?: 'en' | 'zh'
}>()

const isZh = computed(() => props.locale === 'zh')

const t = computed(() =>
  isZh.value
    ? {
        title: 'UI 画廊',
        desc: '同一个 Auto 视图声明，编译到 Web、桌面和移动端。体验真正的跨平台 UI 开发。',
        webTitle: 'Web 版本',
        webDesc: '基于 Vue 3 + shadcn-vue，由 auto build --backend vue 自动生成。',
        openNew: '新窗口打开',
        desktopTitle: '桌面端',
        desktopDesc: '支持 Tauri、Winit 和 LVGL 后端。正在开发中，敬请期待。',
        mobileTitle: '移动端',
        mobileDesc: '支持 ArkTS、Jetpack Compose 和 SwiftUI 后端。正在开发中，敬请期待。',
        howTitle: '工作原理',
        step1Title: '编写视图',
        step1Desc: '使用 Auto 的 view 块声明 UI 结构和样式',
        step2Title: 'AURA 提取',
        step2Desc: '编译器将视图提取为与平台无关的中间表示',
        step3Title: '后端生成',
        step3Desc: '根据目标平台生成 Vue、Compose 或 ArkTS 代码',
      }
    : {
        title: 'UI Gallery',
        desc: 'One Auto view declaration compiles to Web, Desktop, and Mobile. Experience truly cross-platform UI development.',
        webTitle: 'Web (Vue)',
        webDesc: 'Powered by Vue 3 + shadcn-vue, auto-generated via auto build --backend vue.',
        openNew: 'Open in new tab',
        desktopTitle: 'Desktop',
        desktopDesc: 'Targeting Tauri, Winit, and LVGL backends. Coming soon.',
        mobileTitle: 'Mobile',
        mobileDesc: 'Targeting ArkTS, Jetpack Compose, and SwiftUI backends. Coming soon.',
        howTitle: 'How it works',
        step1Title: 'Write Views',
        step1Desc: 'Declare UI structure and styling in Auto\'s view block',
        step2Title: 'AURA Extract',
        step2Desc: 'Compiler extracts views into platform-agnostic IR',
        step3Title: 'Backend Gen',
        step3Desc: 'Generate Vue, Compose, or ArkTS code per target platform',
      }
)

const tabs = computed(() =>
  isZh.value
    ? [
        { key: 'web', label: 'Web', icon: '🌐', badge: 'Live' },
        { key: 'desktop', label: '桌面端', icon: '🖥️', badge: null },
        { key: 'mobile', label: '移动端', icon: '📱', badge: null },
      ]
    : [
        { key: 'web', label: 'Web', icon: '🌐', badge: 'Live' },
        { key: 'desktop', label: 'Desktop', icon: '🖥️', badge: null },
        { key: 'mobile', label: 'Mobile', icon: '📱', badge: null },
      ]
)

const activeTab = ref('web')
const webUrl = '/ui-gallery/web/index.html'
</script>

<style scoped>
.ui-gallery {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 1rem 4rem;
}

.gallery-hero {
  text-align: center;
  padding: 2rem 0 2.5rem;
}

.gallery-title {
  font-size: 2.5rem;
  font-weight: 700;
  background: linear-gradient(120deg, #6366f1 30%, #a855f7 70%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin-bottom: 0.75rem;
}

.gallery-desc {
  font-size: 1.125rem;
  color: hsl(var(--muted-foreground));
  max-width: 600px;
  margin: 0 auto;
  line-height: 1.6;
}

/* Tabs */
.backend-tabs {
  display: flex;
  gap: 0.5rem;
  justify-content: center;
  margin-bottom: 1.5rem;
  flex-wrap: wrap;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 1.25rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  color: hsl(var(--foreground));
  font-size: 0.9375rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tab-btn:hover {
  border-color: hsl(var(--primary) / 0.4);
  background: hsl(var(--accent));
}

.tab-btn.active {
  border-color: hsl(var(--primary));
  background: hsl(var(--primary) / 0.1);
  color: hsl(var(--primary));
}

.tab-icon {
  font-size: 1.125rem;
}

.tab-badge {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  padding: 0.125rem 0.5rem;
  border-radius: 9999px;
  background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%);
  color: white;
}

/* Panel */
.panel {
  border: 1px solid hsl(var(--border));
  border-radius: calc(var(--radius) + 4px);
  background: hsl(var(--card));
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid hsl(var(--border));
  gap: 1rem;
  flex-wrap: wrap;
}

.panel-info h3 {
  font-size: 1rem;
  font-weight: 600;
  margin: 0 0 0.25rem;
}

.panel-info p {
  font-size: 0.875rem;
  color: hsl(var(--muted-foreground));
  margin: 0;
}

.open-link {
  font-size: 0.875rem;
  font-weight: 500;
  color: hsl(var(--primary));
  text-decoration: none;
  padding: 0.375rem 0.75rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--primary) / 0.3);
  transition: all 0.2s ease;
  white-space: nowrap;
}

.open-link:hover {
  background: hsl(var(--primary) / 0.1);
}

.iframe-wrapper {
  position: relative;
  width: 100%;
  height: 700px;
  background: hsl(var(--background));
}

.iframe-wrapper iframe {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  border: none;
}

/* Placeholder panels */
.placeholder-panel {
  min-height: 400px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.placeholder-content {
  text-align: center;
  padding: 3rem;
}

.placeholder-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
}

.placeholder-content h3 {
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.placeholder-content p {
  color: hsl(var(--muted-foreground));
  max-width: 400px;
  margin: 0 auto 1.5rem;
}

.tech-stack {
  display: flex;
  gap: 0.5rem;
  justify-content: center;
  flex-wrap: wrap;
}

.tech-tag {
  font-size: 0.8125rem;
  font-weight: 500;
  padding: 0.375rem 0.875rem;
  border-radius: 9999px;
  background: hsl(var(--muted));
  color: hsl(var(--muted-foreground));
  border: 1px solid hsl(var(--border));
}

/* How it works */
.how-it-works {
  margin-top: 4rem;
  text-align: center;
}

.how-it-works h2 {
  font-size: 1.75rem;
  font-weight: 700;
  margin-bottom: 2rem;
}

.steps {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  flex-wrap: wrap;
}

.step {
  flex: 1;
  min-width: 200px;
  max-width: 280px;
  padding: 1.5rem;
  border-radius: var(--radius);
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  text-align: center;
}

.step-num {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: linear-gradient(135deg, #6366f1 0%, #a855f7 100%);
  color: white;
  font-weight: 700;
  font-size: 0.875rem;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 1rem;
}

.step h4 {
  font-size: 1rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.step p {
  font-size: 0.875rem;
  color: hsl(var(--muted-foreground));
  line-height: 1.5;
  margin: 0;
}

.step-arrow {
  font-size: 1.5rem;
  color: hsl(var(--muted-foreground));
  font-weight: 300;
}

@media (max-width: 768px) {
  .gallery-title {
    font-size: 1.875rem;
  }

  .iframe-wrapper {
    height: 500px;
  }

  .step-arrow {
    display: none;
  }

  .steps {
    flex-direction: column;
  }

  .step {
    max-width: 100%;
    width: 100%;
  }
}
</style>
