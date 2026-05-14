<template>
  <div class="status-transition">
    <select
      :value="status"
      class="status-select"
      :class="status"
      @change="onChange($event)"
    >
      <option
        v-for="s in allowedStatuses"
        :key="s"
        :value="s"
        :disabled="!canTransition(status, s)"
      >
        {{ formatStatus(s) }}
      </option>
    </select>
  </div>
</template>

<script setup lang="ts">
import type { Status, SectionType } from '@/types/specs'

const props = defineProps<{
  status: Status
  sectionType: SectionType
}>()

const emit = defineEmits<{
  change: [status: Status]
}>()

const ALL_STATUSES: Status[] = [
  'empty', 'proposed', 'draft', 'under_review', 'approved',
  'in_progress', 'in_implementation', 'implemented', 'verified',
  'done', 'archived', 'rejected', 'backlog', 'ready', 'in_review',
  'blocked', 'superseded', 'outdated', 'stable', 'deprecated',
  'published', 'analysed', 'obsolete', 'drift'
]

// Per-section allowed statuses (mirrors backend SectionConfig)
const SECTION_STATUSES: Record<SectionType, Status[]> = {
  goals: ['empty', 'proposed', 'draft', 'under_review', 'approved', 'in_progress', 'implemented', 'verified', 'done', 'archived', 'rejected'],
  architecture: ['empty', 'draft', 'under_review', 'approved', 'superseded', 'outdated'],
  designs: ['empty', 'draft', 'under_review', 'approved', 'superseded', 'outdated'],
  plans: ['empty', 'draft', 'approved', 'in_progress', 'done', 'obsolete'],
  tests: ['empty', 'draft', 'implemented', 'done', 'verified', 'blocked'],
  reviews: ['empty', 'draft', 'published'],
  reports: ['empty', 'draft', 'published'],

}

// Allowed transitions (from → to)
const TRANSITIONS: Record<string, Status[]> = {
  empty: ['proposed', 'draft'],
  proposed: ['draft'],
  draft: ['under_review'],
  under_review: ['approved', 'rejected'],
  approved: ['in_progress', 'superseded', 'outdated', 'stable'],
  in_progress: ['implemented', 'done', 'archived', 'obsolete', 'blocked'],
  in_implementation: ['implemented'],
  implemented: ['done', 'verified', 'blocked'],
  done: ['verified', 'archived'],
  verified: ['done', 'archived'],
  archived: [],
  rejected: ['draft'],
  backlog: ['ready'],
  ready: ['in_progress'],
  in_review: ['done'],
  blocked: ['in_progress', 'implemented'],
  superseded: [],
  outdated: [],
  stable: ['deprecated'],
  deprecated: [],
  published: [],
  analysed: ['approved'],
  obsolete: [],
  drift: ['draft', 'under_review'],
}

const allowedStatuses = computed(() => {
  return SECTION_STATUSES[props.sectionType] || ALL_STATUSES
})

function canTransition(from: Status, to: Status): boolean {
  if (from === to) return true
  const targets = TRANSITIONS[from] || []
  return targets.includes(to)
}

function formatStatus(s: Status): string {
  return s.replace(/_/g, ' ')
}

function onChange(e: Event) {
  const target = e.target as HTMLSelectElement
  emit('change', target.value as Status)
}
</script>

<script lang="ts">
import { computed } from 'vue'
</script>

<style scoped>
.status-select {
  appearance: none;
  -webkit-appearance: none;
  padding: 0.2rem 1.5rem 0.2rem 0.6rem;
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.02em;
  border-radius: 999px;
  border: 1px solid transparent;
  cursor: pointer;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 0.4rem center;
  background-size: 0.75rem;
}

.status-select:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* Status colours */
.status-select.empty { color: #94a3b8; background-color: hsl(215 16% 62% / 0.12); }
.status-select.proposed { color: #a78bfa; background-color: hsl(258 90% 66% / 0.12); }
.status-select.draft { color: #94a3b8; background-color: hsl(215 16% 62% / 0.12); }
.status-select.under_review { color: #f59e0b; background-color: hsl(38 92% 50% / 0.12); }
.status-select.approved { color: #3b82f6; background-color: hsl(217 91% 60% / 0.12); }
.status-select.in_progress { color: #f59e0b; background-color: hsl(38 92% 50% / 0.12); }
.status-select.in_implementation { color: #8b5cf6; background-color: hsl(258 90% 66% / 0.12); }
.status-select.implemented { color: #10b981; background-color: hsl(160 84% 39% / 0.12); }
.status-select.verified { color: #22c55e; background-color: hsl(142 71% 45% / 0.12); }
.status-select.done { color: #10b981; background-color: hsl(160 84% 39% / 0.12); }
.status-select.archived { color: #9ca3af; background-color: hsl(220 9% 46% / 0.12); }
.status-select.rejected { color: #ef4444; background-color: hsl(0 72% 51% / 0.12); }
.status-select.backlog { color: #94a3b8; background-color: hsl(215 16% 62% / 0.12); }
.status-select.ready { color: #3b82f6; background-color: hsl(217 91% 60% / 0.12); }
.status-select.in_review { color: #f59e0b; background-color: hsl(38 92% 50% / 0.12); }
.status-select.blocked { color: #ef4444; background-color: hsl(0 72% 51% / 0.12); }
.status-select.superseded { color: #9ca3af; background-color: hsl(220 9% 46% / 0.12); }
.status-select.outdated { color: #9ca3af; background-color: hsl(220 9% 46% / 0.12); }
.status-select.stable { color: #10b981; background-color: hsl(160 84% 39% / 0.12); }
.status-select.deprecated { color: #f59e0b; background-color: hsl(38 92% 50% / 0.12); }
.status-select.published { color: #3b82f6; background-color: hsl(217 91% 60% / 0.12); }
.status-select.analysed { color: #8b5cf6; background-color: hsl(258 90% 66% / 0.12); }
.status-select.obsolete { color: #9ca3af; background-color: hsl(220 9% 46% / 0.12); }
.status-select.drift { color: #ef4444; background-color: hsl(0 72% 51% / 0.12); }
</style>
