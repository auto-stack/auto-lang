<template>
  <div class="showcase-section" :class="{ reverse: reverse }">
    <div class="showcase-content">
      <div v-if="badge" class="showcase-badge">{{ badge }}</div>
      <h2 class="showcase-title">{{ title }}</h2>
      <p v-if="description" class="showcase-description">{{ description }}</p>
      <div class="showcase-body">
        <slot />
      </div>
    </div>
    <div v-if="$slots.visual" class="showcase-visual">
      <slot name="visual" />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  title: string
  description?: string
  badge?: string
  reverse?: boolean
}>()
</script>

<style scoped>
.showcase-section {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 3rem;
  align-items: center;
  padding: 3rem 0;
}

.showcase-section.reverse {
  direction: rtl;
}

.showcase-section.reverse > * {
  direction: ltr;
}

.showcase-content {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.showcase-badge {
  display: inline-flex;
  align-self: flex-start;
  padding: 0.375rem 0.875rem;
  border-radius: 9999px;
  background: rgba(99, 102, 241, 0.1);
  border: 1px solid rgba(99, 102, 241, 0.2);
  color: #6366f1;
  font-size: 0.8rem;
  font-weight: 600;
}

.showcase-title {
  font-size: 1.75rem;
  font-weight: 700;
  line-height: 1.2;
  color: hsl(var(--foreground));
  margin: 0;
}

.showcase-description {
  font-size: 1.05rem;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
  margin: 0;
}

.showcase-body {
  font-size: 0.95rem;
  line-height: 1.7;
  color: hsl(var(--muted-foreground));
}

.showcase-body :deep(ul) {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.showcase-body :deep(li) {
  position: relative;
  padding-left: 1.5rem;
  color: hsl(var(--foreground));
}

.showcase-body :deep(li)::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0.5rem;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: linear-gradient(135deg, #6366f1, #a855f7);
}

.showcase-visual {
  display: flex;
  align-items: center;
  justify-content: center;
}

.showcase-visual :deep(pre) {
  margin: 0;
  width: 100%;
}

@media (max-width: 768px) {
  .showcase-section {
    grid-template-columns: 1fr;
    gap: 2rem;
  }
}
</style>
