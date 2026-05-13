<template>
  <div class="plan-detail">
    <div v-for="(phase, idx) in phases" :key="idx" class="phase-card">
      <div class="phase-header">
        <div class="phase-title">
          <span class="phase-num">P{{ phase.number }}</span>
          <span class="phase-name">{{ phase.title }}</span>
          <span v-if="phase.version" class="phase-version">{{ phase.version }}</span>
        </div>
        <div class="phase-progress">
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: phase.progress + '%' }" />
          </div>
          <span class="progress-text">{{ phase.completed }}/{{ phase.tasks.length }}</span>
        </div>
      </div>
      <ul class="task-list">
        <li
          v-for="(task, tidx) in phase.tasks"
          :key="tidx"
          :class="{ done: task.done }"
        >
          <span class="task-check">{{ task.done ? '✓' : '○' }}</span>
          <span class="task-text">{{ task.text }}</span>
        </li>
      </ul>
    </div>
    <MarkdownContent v-if="remainingContent" :content="remainingContent" @link-click="$emit('linkClick', $event)" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import MarkdownContent from '@/components/MarkdownContent.vue'

const props = defineProps<{
  content: string
}>()

const emit = defineEmits<{
  linkClick: [id: string]
}>()

interface Phase {
  number: number
  title: string
  version: string
  tasks: { text: string; done: boolean }[]
  completed: number
  progress: number
}

const phases = computed<Phase[]>(() => {
  const result: Phase[] = []
  const lines = props.content.split('\n')
  let current: Phase | null = null

  for (const raw of lines) {
    const line = raw.trimEnd()
    const headingMatch = line.match(/^##\s+Phase\s+(\d+):\s+(.+?)(?:\s+\(([^)]+)\))?\s*$/i)
    if (headingMatch) {
      if (current) result.push(current)
      current = {
        number: parseInt(headingMatch[1]),
        title: headingMatch[2].trim(),
        version: headingMatch[3] || '',
        tasks: [],
        completed: 0,
        progress: 0,
      }
      continue
    }
    if (current) {
      const taskMatch = line.match(/^-\s+\[([ xX])\]\s+(.+)$/)
      if (taskMatch) {
        const done = taskMatch[1].toLowerCase() === 'x'
        current.tasks.push({ text: taskMatch[2].trim(), done })
        if (done) current.completed++
      }
    }
  }
  if (current) {
    current.progress = current.tasks.length ? Math.round((current.completed / current.tasks.length) * 100) : 0
    result.push(current)
  }
  // Calculate progress for all
  result.forEach(p => {
    p.progress = p.tasks.length ? Math.round((p.completed / p.tasks.length) * 100) : 0
  })
  return result
})

const remainingContent = computed(() => {
  // Strip out phase headings and tasks that we've parsed, keep everything else
  const lines = props.content.split('\n')
  const kept: string[] = []
  let inPhase = false
  for (const raw of lines) {
    const line = raw.trimEnd()
    if (line.match(/^##\s+Phase\s+\d+/i)) {
      inPhase = true
      continue
    }
    if (inPhase && line.match(/^##\s+/)) {
      inPhase = false
    }
    if (inPhase && line.match(/^-\s+\[[ xX]\]/)) {
      continue
    }
    if (line.trim()) kept.push(line)
  }
  return kept.join('\n')
})
</script>

<style scoped>
.plan-detail {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.phase-card {
  border: 1px solid var(--af-border);
  border-radius: 10px;
  padding: 0.9rem 1rem;
  background: hsl(var(--muted-foreground) / 0.02);
}

.phase-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.6rem;
}

.phase-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.phase-num {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 6px;
  background: hsl(var(--primary) / 0.1);
  color: hsl(var(--primary));
  font-size: 0.7rem;
  font-weight: 700;
}

.phase-name {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--af-fg);
}

.phase-version {
  font-size: 0.7rem;
  color: var(--af-muted);
  background: hsl(var(--muted-foreground) / 0.08);
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
}

.phase-progress {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.progress-bar {
  width: 80px;
  height: 6px;
  border-radius: 3px;
  background: hsl(var(--muted-foreground) / 0.1);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 3px;
  background: hsl(var(--primary));
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 0.7rem;
  color: var(--af-muted);
  font-variant-numeric: tabular-nums;
}

.task-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.task-list li {
  display: flex;
  align-items: flex-start;
  gap: 0.4rem;
  font-size: 0.82rem;
  color: var(--af-fg);
  line-height: 1.4;
}

.task-list li.done {
  opacity: 0.55;
}

.task-list li.done .task-text {
  text-decoration: line-through;
}

.task-check {
  font-size: 0.75rem;
  color: hsl(var(--primary));
  min-width: 1rem;
  text-align: center;
  margin-top: 0.05rem;
}

.task-list li.done .task-check {
  color: hsl(142 71% 45%);
}

.task-text {
  flex: 1;
}
</style>
