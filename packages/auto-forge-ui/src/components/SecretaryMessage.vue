<template>
  <div
    class="secretary-message"
    :class="{ dismissed: isDismissed }"
    role="alert"
    aria-live="polite"
    aria-atomic="true"
    tabindex="0"
  >
    <div class="secretary-header">
      <span class="secretary-icon">📋</span>
      <div class="secretary-info">
        <div class="secretary-title">{{ gate.title }}</div>
        <div class="secretary-meta">
          <span class="secretary-profession">{{ gate.profession }}</span>
          <span class="secretary-waiting">Waiting {{ formatElapsed(gate.since) }}</span>
        </div>
      </div>
      <button class="secretary-dismiss" @click="dismiss" title="Dismiss">
        <X :size="14" />
      </button>
    </div>

    <div class="secretary-actions">
      <button class="secretary-btn approve" @click="$emit('approve', gate.gateId)">
        <Check :size="13" />
        Approve
      </button>
      <button class="secretary-btn reject" @click="$emit('reject', gate.gateId)">
        <X :size="13" />
        Reject
      </button>
      <button class="secretary-btn snooze" @click="$emit('snooze', gate.gateId)">
        <Clock :size="13" />
        Snooze
      </button>
      <button
        v-if="gate.sectionId"
        class="secretary-btn review"
        @click="$emit('review-in-specs', gate.sectionId)"
      >
        <Eye :size="13" />
        Review in Specs
      </button>
    </div>

    <div v-if="queuePosition > 0" class="secretary-queue">
      {{ queuePosition }} more gate{{ queuePosition > 1 ? 's' : '' }} queued
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Check, X, Clock, Eye } from 'lucide-vue-next'
import type { PendingGate } from '@/composables/useGateInbox'

interface Props {
  gate: PendingGate
  queuePosition: number
}

defineProps<Props>()

defineEmits<{
  (e: 'approve', gateId: string): void
  (e: 'reject', gateId: string): void
  (e: 'snooze', gateId: string): void
  (e: 'review-in-specs', sectionId: string): void
}>()

const isDismissed = ref(false)

function dismiss() {
  isDismissed.value = true
  setTimeout(() => {
    // After animation, parent should remove this component
  }, 300)
}

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
.secretary-message {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.6rem 0.8rem;
  border-radius: 10px;
  border: 1px solid hsl(var(--primary) / 0.2);
  background: hsl(var(--primary) / 0.04);
  margin: 0.5rem 0;
  transition: opacity 0.3s, transform 0.3s;
}

.secretary-message.dismissed {
  opacity: 0;
  transform: translateX(20px);
  pointer-events: none;
}

.secretary-header {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
}

.secretary-icon {
  font-size: 1.1rem;
  flex-shrink: 0;
  margin-top: 0.1rem;
}

.secretary-info {
  flex: 1;
  min-width: 0;
}

.secretary-title {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--af-fg);
  line-height: 1.3;
}

.secretary-meta {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin-top: 0.15rem;
}

.secretary-profession {
  font-size: 0.7rem;
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
  background: hsl(var(--primary) / 0.1);
  color: var(--af-primary);
  font-weight: 500;
}

.secretary-waiting {
  font-size: 0.7rem;
  color: var(--af-muted);
}

.secretary-dismiss {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--af-muted);
  cursor: pointer;
  transition: all 0.15s;
  flex-shrink: 0;
}

.secretary-dismiss:hover {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
}

.secretary-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}

.secretary-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.3rem 0.6rem;
  border: none;
  border-radius: 5px;
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s;
}

.secretary-btn.approve {
  background: hsl(142 70% 45% / 0.15);
  color: hsl(142 70% 35%);
}

.secretary-btn.reject {
  background: hsl(0 70% 45% / 0.1);
  color: hsl(0 70% 45%);
}

.secretary-btn.snooze {
  background: hsl(38 90% 50% / 0.1);
  color: hsl(38 80% 40%);
}

.secretary-btn.review {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
}

.secretary-btn:hover {
  opacity: 0.85;
}

.secretary-queue {
  font-size: 0.7rem;
  color: var(--af-muted);
  padding-top: 0.2rem;
}
</style>
