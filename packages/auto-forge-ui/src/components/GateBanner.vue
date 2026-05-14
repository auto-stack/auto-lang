<template>
  <div v-if="visible" class="gate-banner" :class="{ expanded: showDiff }">
    <div class="gate-banner-row">
      <div class="gate-banner-info">
        <span class="gate-banner-icon">🔒</span>
        <span class="gate-banner-text">
          <strong>{{ gate.profession }}</strong> is waiting for approval: {{ gate.title }}
        </span>
        <span class="gate-banner-waiting">{{ formatElapsed(gate.since) }}</span>
      </div>
      <div class="gate-banner-actions">
        <button class="banner-btn approve" @click="$emit('approve', gate.gateId)">
          <Check :size="12" />
        </button>
        <button class="banner-btn reject" @click="$emit('reject', gate.gateId)">
          <X :size="12" />
        </button>
        <button
          v-if="gate.sectionId"
          class="banner-btn view"
          @click="showDiff = !showDiff"
        >
          <Eye :size="12" />
        </button>
        <button class="banner-btn open-chat" @click="$emit('open-in-chat', gate.gateId)">
          <MessageSquare :size="12" />
        </button>
      </div>
    </div>
    <div v-if="showDiff && gate.sectionId" class="gate-banner-diff">
      <div class="diff-placeholder">Diff view for {{ gate.sectionId }} would render here</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Check, X, Eye, MessageSquare } from 'lucide-vue-next'
import type { PendingGate } from '@/composables/useGateInbox'

interface Props {
  gate: PendingGate
}

defineProps<Props>()

defineEmits<{
  (e: 'approve', gateId: string): void
  (e: 'reject', gateId: string): void
  (e: 'open-in-chat', gateId: string): void
}>()

const visible = ref(true)
const showDiff = ref(false)

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
.gate-banner {
  position: sticky;
  top: 0;
  z-index: 10;
  background: hsl(38 90% 50% / 0.08);
  border-bottom: 1px solid hsl(38 90% 50% / 0.2);
  padding: 0.5rem 1rem;
}

.gate-banner-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.gate-banner-info {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  min-width: 0;
  flex: 1;
}

.gate-banner-icon {
  font-size: 0.95rem;
  flex-shrink: 0;
}

.gate-banner-text {
  font-size: 0.8rem;
  color: var(--af-fg);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.gate-banner-waiting {
  font-size: 0.7rem;
  color: var(--af-muted);
  flex-shrink: 0;
}

.gate-banner-actions {
  display: flex;
  align-items: center;
  gap: 0.2rem;
  flex-shrink: 0;
}

.banner-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  border-radius: 5px;
  background: transparent;
  color: var(--af-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.banner-btn:hover {
  background: hsl(var(--muted-foreground) / 0.08);
}

.banner-btn.approve:hover {
  background: hsl(142 70% 45% / 0.15);
  color: hsl(142 70% 35%);
}

.banner-btn.reject:hover {
  background: hsl(0 70% 45% / 0.1);
  color: hsl(0 70% 45%);
}

.banner-btn.view:hover,
.banner-btn.open-chat:hover {
  color: var(--af-fg);
}

.gate-banner-diff {
  margin-top: 0.5rem;
  padding: 0.5rem;
  background: var(--af-bg);
  border: 1px solid var(--af-border);
  border-radius: 6px;
  font-size: 0.75rem;
  color: var(--af-muted);
}
</style>
