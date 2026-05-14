<template>
  <div class="gate-panel">
    <div class="gate-panel-header">
      <div class="gate-panel-icon">{{ professionIcon || '🔒' }}</div>
      <div class="gate-panel-info">
        <div class="gate-panel-title">{{ gate.title }}</div>
        <div class="gate-panel-meta">
          <span class="gate-panel-profession">{{ gate.profession }}</span>
          <span class="gate-panel-waiting">Waiting {{ formatElapsed(gate.since) }}</span>
        </div>
      </div>
    </div>
    <div class="gate-panel-actions">
      <button class="panel-btn approve" @click="$emit('approve', runId)">
        <Check :size="14" />
        Approve
      </button>
      <button class="panel-btn reject" @click="$emit('reject', runId)">
        <X :size="14" />
        Reject
      </button>
      <button
        v-if="gate.sectionId"
        class="panel-btn review"
        @click="$emit('review-in-specs', gate.sectionId)"
      >
        <Scroll :size="14" />
        Review in Specs
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Check, X, Scroll } from 'lucide-vue-next'
import type { PendingGate } from '@/composables/useGateInbox'

interface Props {
  runId: string
  gate: PendingGate
  professionIcon?: string
}

withDefaults(defineProps<Props>(), {
  professionIcon: undefined,
})

defineEmits<{
  (e: 'approve', runId: string): void
  (e: 'reject', runId: string): void
  (e: 'review-in-specs', sectionId: string): void
}>()

function formatElapsed(since: number): string {
  const mins = Math.floor((Date.now() - since) / 60000)
  if (mins < 1) return 'just now'
  if (mins < 60) return `${mins}m`
  const hrs = Math.floor(mins / 60)
  if (hrs < 24) return `${hrs}h`
  return `${Math.floor(hrs / 24)}d`
}
</script>

<style scoped>
.gate-panel {
  padding: 0.75rem 1rem;
  border: 1px solid hsl(38 90% 50% / 0.3);
  border-radius: 8px;
  background: hsl(38 90% 50% / 0.04);
  margin-bottom: 1rem;
}

.gate-panel-header {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.gate-panel-icon {
  font-size: 1.2rem;
  flex-shrink: 0;
}

.gate-panel-info {
  flex: 1;
  min-width: 0;
}

.gate-panel-title {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--af-fg);
}

.gate-panel-meta {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin-top: 0.15rem;
}

.gate-panel-profession {
  font-size: 0.7rem;
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
  background: hsl(var(--primary) / 0.1);
  color: var(--af-primary);
  font-weight: 500;
}

.gate-panel-waiting {
  font-size: 0.7rem;
  color: var(--af-muted);
}

.gate-panel-actions {
  display: flex;
  gap: 0.4rem;
}

.panel-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.35rem 0.7rem;
  border: none;
  border-radius: 5px;
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s;
}

.panel-btn.approve {
  background: hsl(142 70% 45% / 0.15);
  color: hsl(142 70% 35%);
}

.panel-btn.reject {
  background: hsl(0 70% 45% / 0.1);
  color: hsl(0 70% 45%);
}

.panel-btn.review {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
}

.panel-btn:hover {
  opacity: 0.85;
}
</style>
