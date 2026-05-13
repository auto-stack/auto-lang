<template>
  <div class="order-view">
    <div class="order-header">
      <h2>Agents</h2>
      <div class="order-stats">
        <span class="stat">Phase: {{ sessionPhase }}</span>
        <span class="stat">Status: {{ sessionStatus }}</span>
      </div>
    </div>
    <div class="order-body">
      <!-- Phase Pipeline -->
      <div class="pipeline-flow">
        <div
          v-for="(phase, idx) in phases"
          :key="phase.key"
          class="pipeline-node"
          :class="phaseStatus(phase.key)"
        >
          <div class="node-icon">{{ phase.icon }}</div>
          <div class="node-name">{{ phase.label }}</div>
          <div v-if="phaseTime(phase.key)" class="node-meta">{{ phaseTime(phase.key) }}s</div>
          <div v-if="idx < phases.length - 1" class="node-arrow">→</div>
        </div>
      </div>

      <!-- Todo Progress (when in Execution) -->
      <div v-if="sessionPhase === 'execution'" class="progress-panel">
        <div class="progress-header">
          <span class="progress-title">Executing Todos</span>
          <span class="progress-count">{{ completedTodos }} / {{ totalTodos }}</span>
        </div>
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: todoProgressPercent + '%' }"></div>
        </div>
      </div>

      <!-- Phase History -->
      <div class="history-panel">
        <div class="history-title">Phase History</div>
        <div v-if="phaseHistory.length === 0" class="history-empty">No phase transitions yet</div>
        <div
          v-for="entry in phaseHistory"
          :key="entry.phase + entry.entered_at"
          class="history-row"
        >
          <span class="history-phase">{{ entry.phase }}</span>
          <span class="history-time">{{ formatTime(entry.entered_at) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useForge } from '@/composables/useForge'

const { sessionPhase, sessionStatus, session } = useForge()

const phases = [
  { key: 'intake', label: 'Intake', icon: '📥' },
  { key: 'spec_draft', label: 'SpecDraft', icon: '📝' },
  { key: 'spec_review', label: 'SpecReview', icon: '🔍' },
  { key: 'execution', label: 'Execution', icon: '⚒️' },
  { key: 'verification', label: 'Verification', icon: '✅' },
]

const phaseHistory = computed(() => session.value?.phase_history ?? [])

function phaseStatus(phaseKey: string): string {
  const current = sessionPhase.value
  const order = ['intake', 'spec_draft', 'spec_review', 'execution', 'verification']
  const currentIdx = order.indexOf(current)
  const phaseIdx = order.indexOf(phaseKey)

  if (phaseIdx < currentIdx) return 'completed'
  if (phaseIdx === currentIdx) return 'active'
  return 'pending'
}

function phaseTime(phaseKey: string): string | null {
  const entry = phaseHistory.value.find((e) => e.phase === phaseKey)
  if (!entry) return null
  // Show duration or entry time
  return Math.round(entry.entered_at).toString()
}

// Mock todo progress until backend tracks real todo counts
const totalTodos = computed(() => 7)
const completedTodos = computed(() => {
  const idx = session.value?.current_todo_index
  return idx !== undefined && idx !== null ? idx + 1 : 0
})
const todoProgressPercent = computed(() => {
  if (totalTodos.value === 0) return 0
  return Math.round((completedTodos.value / totalTodos.value) * 100)
})

function formatTime(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}
</script>

<style scoped>
.order-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.order-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.6rem 1.25rem;
  flex-shrink: 0;
}

.order-header h2 {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--af-fg);
}

.order-stats {
  display: flex;
  gap: 1rem;
  font-size: 0.8rem;
}

.stat {
  color: var(--af-muted);
  text-transform: capitalize;
}

.order-body {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.pipeline-flow {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 1rem;
  border: 1px solid var(--af-border);
  border-radius: 8px;
  overflow-x: auto;
}

.pipeline-node {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
  padding: 0.6rem 0.8rem;
  border-radius: 8px;
  min-width: 80px;
  border: 1px solid transparent;
  transition: all 0.2s;
}

.pipeline-node.completed {
  border-color: hsl(var(--af-success) / 0.25);
}

.pipeline-node.active {
  border-color: hsl(var(--af-order) / 0.3);
  background: hsl(var(--af-order) / 0.04);
}

.pipeline-node.pending {
  border-color: transparent;
  opacity: 0.5;
}

.node-icon {
  font-size: 1.1rem;
}

.node-name {
  font-size: 0.7rem;
  font-weight: 500;
  color: var(--af-fg);
}

.node-meta {
  font-size: 0.65rem;
  color: var(--af-muted);
}

.node-arrow {
  font-size: 1.1rem;
  color: var(--af-border);
  margin: 0 0.2rem;
}

/* Progress Panel */
.progress-panel {
  border: 1px solid var(--af-border);
  border-radius: 8px;
  padding: 0.75rem 1rem;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.progress-title {
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--af-fg);
}

.progress-count {
  font-size: 0.75rem;
  color: var(--af-muted);
}

.progress-bar {
  height: 6px;
  background: hsl(var(--muted-foreground) / 0.1);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: hsl(var(--af-success));
  border-radius: 3px;
  transition: width 0.3s ease;
}

/* History Panel */
.history-panel {
  border: 1px solid var(--af-border);
  border-radius: 8px;
  padding: 0.75rem 1rem;
}

.history-title {
  font-size: 0.8rem;
  font-weight: 500;
  margin-bottom: 0.5rem;
  color: var(--af-fg);
}

.history-empty {
  font-size: 0.8rem;
  color: var(--af-muted);
  text-align: center;
  padding: 0.5rem 0;
}

.history-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.35rem 0;
  border-bottom: 1px solid var(--af-border);
}

.history-row:last-child {
  border-bottom: none;
}

.history-phase {
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--af-fg);
  text-transform: capitalize;
}

.history-time {
  font-size: 0.75rem;
  color: var(--af-muted);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}
</style>
