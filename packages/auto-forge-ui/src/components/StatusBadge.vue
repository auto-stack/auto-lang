<template>
  <span class="status-badge" :class="[status, size]">
    {{ status.replace('_', ' ') }}
  </span>
</template>

<script setup lang="ts">
defineProps<{
  status: string
  size?: 'sm' | 'md'
}>()
</script>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  padding: 0.15rem 0.5rem;
  border-radius: 999px;
  font-size: 0.65rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.02em;
  white-space: nowrap;
}
.status-badge.sm {
  padding: 0.1rem 0.4rem;
  font-size: 0.6rem;
}
.status-badge.md {
  padding: 0.25rem 0.6rem;
  font-size: 0.75rem;
}

.status-badge.empty {
  color: #94a3b8;
  background: hsl(215 16% 62% / 0.12);
}
.status-badge.proposed {
  color: #a78bfa;
  background: hsl(258 90% 66% / 0.12);
}
.status-badge.draft {
  color: #94a3b8;
  background: hsl(215 16% 62% / 0.12);
}
.status-badge.under_review {
  color: #f59e0b;
  background: hsl(38 92% 50% / 0.12);
}
.status-badge.approved {
  color: #3b82f6;
  background: hsl(217 91% 60% / 0.12);
}
.status-badge.in_progress {
  color: #f59e0b;
  background: hsl(38 92% 50% / 0.12);
}
.status-badge.in_implementation {
  color: #8b5cf6;
  background: hsl(258 90% 66% / 0.12);
}
.status-badge.implemented {
  color: #10b981;
  background: hsl(160 84% 39% / 0.12);
}
.status-badge.verified {
  color: #22c55e;
  background: hsl(142 71% 45% / 0.12);
}
.status-badge.done {
  color: #10b981;
  background: hsl(160 84% 39% / 0.12);
}
.status-badge.archived {
  color: #9ca3af;
  background: hsl(220 9% 46% / 0.12);
}
.status-badge.rejected {
  color: #ef4444;
  background: hsl(0 84% 60% / 0.12);
}
.status-badge.backlog {
  color: #94a3b8;
  background: hsl(215 16% 62% / 0.12);
}
.status-badge.ready {
  color: #3b82f6;
  background: hsl(217 91% 60% / 0.12);
}
.status-badge.in_review {
  color: #f97316;
  background: hsl(24 95% 53% / 0.12);
}
.status-badge.blocked {
  color: #ef4444;
  background: hsl(0 84% 60% / 0.12);
}
.status-badge.superseded,
.status-badge.outdated,
.status-badge.deprecated,
.status-badge.obsolete {
  color: #9ca3af;
  background: hsl(220 9% 46% / 0.12);
  text-decoration: line-through;
}
.status-badge.stable {
  color: #3b82f6;
  background: hsl(217 91% 60% / 0.12);
}
.status-badge.published {
  color: #8b5cf6;
  background: hsl(258 90% 66% / 0.12);
}
.status-badge.analysed {
  color: #8b5cf6;
  background: hsl(258 90% 66% / 0.12);
}

/* Relay statuses */
.status-badge.idle,
.status-badge.completed {
  color: #10b981;
  background: hsl(160 84% 39% / 0.12);
}
.status-badge.running,
.status-badge.in_progress {
  color: #3b82f6;
  background: hsl(217 91% 60% / 0.12);
}
.status-badge.failed,
.status-badge.error {
  color: #ef4444;
  background: hsl(0 84% 60% / 0.12);
}
.status-badge.waitingforhuman,
.status-badge.waiting_approval {
  color: #f59e0b;
  background: hsl(38 92% 50% / 0.12);
}
.status-badge.paused {
  color: #94a3b8;
  background: hsl(215 16% 62% / 0.12);
}
</style>
