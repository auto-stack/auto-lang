<template>
  <div class="agents-view">
    <div class="agents-header">
      <h2>Agents Relay</h2>
      <div class="agents-actions">
        <button class="btn-primary" @click="showStartModal = true" :disabled="loading">
          <Play :size="14" />
          Start Run
        </button>
        <button class="btn-secondary" @click="refresh">
          <RefreshCw :size="14" />
        </button>
      </div>
    </div>

    <div v-if="error" class="error-banner">{{ error }}</div>

    <div class="agents-body">
      <!-- Left: Runs list -->
      <div class="runs-sidebar">
        <div class="panel-title">Runs</div>
        <div v-if="runs.length === 0" class="empty-state">No runs yet</div>
        <div
          v-for="run in runs"
          :key="run.run_id"
          class="run-card"
          :class="{ active: currentRun?.run_id === run.run_id }"
          @click="selectRun(run.run_id)"
        >
          <div class="run-card-header">
            <span class="run-id">{{ run.run_id }}</span>
            <StatusBadge :status="run.status" />
          </div>
          <div class="run-card-meta">
            <span>{{ run.current_profession ?? '—' }}</span>
            <span>{{ formatTokens(run.cumulative_tokens) }}</span>
          </div>
          <div class="run-progress-bar">
            <div
              class="run-progress-fill"
              :style="{ width: runProgressPercent(run) + '%' }"
            />
          </div>
        </div>
      </div>

      <!-- Center: Pipeline visualization -->
      <div class="pipeline-panel">
        <div v-if="!currentRun" class="empty-state">
          Select a run or start a new one
        </div>

        <template v-else>
          <!-- Run header -->
          <div class="run-header">
            <div class="run-title">{{ currentRun.run_id }}</div>
            <div class="run-stats">
              <span class="stat-badge">
                <Coins :size="12" />
                {{ formatTokens(currentRun.cumulative_tokens) }}
              </span>
              <span class="stat-badge">
                <Zap :size="12" />
                {{ Math.round(currentRun.savings_ratio * 100) }}% saved
              </span>
            </div>
          </div>

          <!-- Budget bar -->
          <div class="budget-bar-container">
            <div class="budget-label">
              <span>Budget</span>
              <span>{{ formatTokens(currentRun.budget_limit - currentRun.budget_remaining) }} / {{ formatTokens(currentRun.budget_limit) }}</span>
            </div>
            <div class="budget-bar">
              <div
                class="budget-fill"
                :class="{ warning: budgetUsedPercent > 70, danger: budgetUsedPercent > 90 }"
                :style="{ width: budgetUsedPercent + '%' }"
              />
            </div>
          </div>

          <!-- Pipeline steps -->
          <div class="pipeline-flow">
            <template v-for="(step, idx) in currentRun.steps" :key="step.id">
              <div
                class="pipeline-step"
                :class="step.status"
                :title="`${step.profession_id} (${step.gate})`"
              >
                <div class="step-icon">{{ professionIcon(step.profession_id) }}</div>
                <div class="step-name">{{ step.profession_id }}</div>
                <div v-if="step.gate === 'human'" class="step-gate">🔒</div>
                <div v-if="step.status === 'running'" class="step-pulse" />
              </div>
              <div v-if="idx < currentRun.steps.length - 1" class="step-connector">
                <ChevronRight :size="14" />
              </div>
            </template>
          </div>

          <!-- Gate approval panel -->
          <div v-if="hasActiveGate && currentRun.waiting_for_gate" class="gate-panel">
            <div class="gate-header">
              <AlertCircle :size="16" />
              <span>Human gate at {{ currentRun.waiting_for_gate.profession_id }}</span>
            </div>
            <div class="gate-actions">
              <button class="btn-approve" @click="onApprove">
                <Check :size="14" />
                Approve
              </button>
              <button class="btn-reject" @click="onReject">
                <X :size="14" />
                Reject
              </button>
              <button class="btn-edit" @click="onEdit">
                <Edit3 :size="14" />
                Edit &amp; Approve
              </button>
            </div>
          </div>

          <!-- Step history -->
          <div class="history-panel">
            <div class="panel-title">Step History</div>
            <div v-if="currentRun.step_history.length === 0" class="empty-state">No steps completed yet</div>
            <div
              v-for="record in currentRun.step_history"
              :key="record.step_id + record.started_at"
              class="history-row"
            >
              <span class="history-profession">{{ record.profession_id }}</span>
              <span class="history-time">{{ formatTime(record.completed_at) }}</span>
            </div>
          </div>
        </template>
      </div>

      <!-- Right: Professions & Souls -->
      <div class="config-sidebar">
        <div class="panel-title">Professions</div>
        <div class="profession-list">
          <div v-for="p in professions" :key="p.id" class="profession-item">
            <div class="profession-name">{{ p.name }}</div>
            <div class="profession-phase">{{ p.phase }}</div>
          </div>
        </div>

        <div class="panel-title" style="margin-top: 1rem;">Souls</div>
        <div class="soul-list">
          <div v-for="s in souls" :key="s.id" class="soul-item">
            {{ s.name }}
          </div>
        </div>
      </div>
    </div>

    <!-- Start Run Modal -->
    <div v-if="showStartModal" class="modal-overlay" @click.self="showStartModal = false">
      <div class="modal-content">
        <h3>Start New Run</h3>
        <div class="form-group">
          <label>Flow ID</label>
          <input v-model="newFlowId" placeholder="e.g. feature-auth" />
        </div>
        <div class="form-group">
          <label>Steps</label>
          <div class="steps-builder">
            <div v-for="(step, i) in newSteps" :key="i" class="step-row">
              <input v-model="step.id" placeholder="step-id" class="step-input" />
              <select v-model="step.profession_id" class="step-select">
                <option v-for="p in professions" :key="p.id" :value="p.id">{{ p.name }}</option>
              </select>
              <select v-model="step.gate" class="step-select">
                <option value="auto">Auto</option>
                <option value="human">Human</option>
              </select>
              <button class="btn-icon" @click="removeStep(i)">
                <Trash2 :size="14" />
              </button>
            </div>
            <button class="btn-add" @click="addStep">
              <Plus :size="14" />
              Add Step
            </button>
          </div>
        </div>
        <div class="modal-actions">
          <button class="btn-secondary" @click="showStartModal = false">Cancel</button>
          <button class="btn-primary" :disabled="loading" @click="onStartRun">
            <Play :size="14" />
            Start
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import {
  Play, RefreshCw, Coins, Zap, ChevronRight,
  AlertCircle, Check, X, Edit3, Trash2, Plus,
} from 'lucide-vue-next'
import { useRelay } from '@/composables/useRelay'
import StatusBadge from '@/components/StatusBadge.vue'

const {
  runs, currentRun, professions, souls, loading, error,
  hasActiveGate, budgetUsedPercent,
  loadProfessions, loadSouls, loadRuns, loadRun, startRun,
  resolveGate, subscribeToRun,
} = useRelay()

const showStartModal = ref(false)
const newFlowId = ref('demo-flow')
const newSteps = ref<{ id: string; profession_id: string; gate: string }[]>([
  { id: 'intake', profession_id: 'assistant', gate: 'auto' },
  { id: 'discover', profession_id: 'advisor', gate: 'human' },
  { id: 'design', profession_id: 'architect', gate: 'auto' },
  { id: 'plan', profession_id: 'planner', gate: 'auto' },
  { id: 'draft-tests', profession_id: 'tester', gate: 'auto' },
  { id: 'code', profession_id: 'coder', gate: 'auto' },
  { id: 'run-tests', profession_id: 'tester', gate: 'auto' },
  { id: 'review', profession_id: 'reviewer', gate: 'auto' },
  { id: 'report', profession_id: 'documenter', gate: 'auto' },
])

let unsubscribe: (() => void) | null = null

onMounted(async () => {
  await loadProfessions()
  await loadSouls()
  await loadRuns()
})

onUnmounted(() => {
  if (unsubscribe) unsubscribe()
})

function selectRun(runId: string) {
  if (unsubscribe) unsubscribe()
  loadRun(runId)
  unsubscribe = subscribeToRun(runId)
}

async function refresh() {
  await loadRuns()
  if (currentRun.value) {
    await loadRun(currentRun.value.run_id)
  }
}

function addStep() {
  newSteps.value.push({ id: '', profession_id: 'coder', gate: 'auto' })
}

function removeStep(i: number) {
  newSteps.value.splice(i, 1)
}

async function onStartRun() {
  const runId = await startRun({
    flow_id: newFlowId.value,
    steps: newSteps.value,
  })
  if (runId) {
    showStartModal.value = false
    selectRun(runId)
  }
}

async function onApprove() {
  if (!currentRun.value) return
  await resolveGate(currentRun.value.run_id, 'approve')
}

async function onReject() {
  if (!currentRun.value) return
  await resolveGate(currentRun.value.run_id, 'reject', 'Needs revision')
}

async function onEdit() {
  if (!currentRun.value) return
  await resolveGate(currentRun.value.run_id, 'edit', 'Approved with minor edits')
}

function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`
  return `${n}`
}

function formatTime(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

function runProgressPercent(run: { current_step: number; total_steps: number }): number {
  if (run.total_steps === 0) return 0
  return Math.round((run.current_step / run.total_steps) * 100)
}

function professionIcon(id: string): string {
  const map: Record<string, string> = {
    assistant: '📥', advisor: '💡', planner: '📝', architect: '🏗️',
    coder: '💻', tester: '🧪', reviewer: '🔍', documenter: '📚',
  }
  return map[id] ?? '⚙️'
}
</script>

<style scoped>
.agents-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.agents-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.6rem 1.25rem;
  flex-shrink: 0;
  border-bottom: 1px solid var(--af-border);
}

.agents-header h2 {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--af-fg);
}

.agents-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-primary, .btn-secondary, .btn-approve, .btn-reject, .btn-edit, .btn-add, .btn-icon {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.4rem 0.7rem;
  border-radius: 6px;
  border: none;
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-primary {
  background: var(--af-primary);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary, .btn-icon {
  background: hsl(var(--muted-foreground) / 0.08);
  color: var(--af-fg);
}

.btn-secondary:hover, .btn-icon:hover {
  background: hsl(var(--muted-foreground) / 0.14);
}

.btn-approve { background: hsl(142 70% 45% / 0.15); color: hsl(142 70% 35%); }
.btn-reject { background: hsl(0 70% 45% / 0.15); color: hsl(0 70% 45%); }
.btn-edit { background: hsl(220 70% 50% / 0.15); color: hsl(220 70% 45%); }
.btn-add { background: transparent; color: var(--af-muted); border: 1px dashed var(--af-border); width: 100%; justify-content: center; }

.error-banner {
  padding: 0.5rem 1.25rem;
  background: hsl(0 70% 50% / 0.08);
  color: hsl(0 70% 45%);
  font-size: 0.8rem;
  border-bottom: 1px solid var(--af-border);
}

.agents-body {
  flex: 1;
  display: grid;
  grid-template-columns: 220px 1fr 180px;
  gap: 1px;
  background: var(--af-border);
  overflow: hidden;
}

.runs-sidebar, .pipeline-panel, .config-sidebar {
  background: var(--af-bg);
  overflow-y: auto;
  padding: 0.75rem;
}

.panel-title {
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--af-muted);
  margin-bottom: 0.5rem;
}

.empty-state {
  font-size: 0.8rem;
  color: var(--af-muted);
  text-align: center;
  padding: 1rem 0;
}

/* Run cards */
.run-card {
  padding: 0.6rem;
  border-radius: 6px;
  border: 1px solid var(--af-border);
  margin-bottom: 0.5rem;
  cursor: pointer;
  transition: all 0.15s;
}

.run-card:hover, .run-card.active {
  border-color: hsl(var(--primary) / 0.3);
  background: hsl(var(--primary) / 0.03);
}

.run-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.3rem;
}

.run-id {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--af-fg);
  font-family: 'JetBrains Mono', monospace;
}

.run-card-meta {
  display: flex;
  justify-content: space-between;
  font-size: 0.7rem;
  color: var(--af-muted);
  margin-bottom: 0.4rem;
}

.run-progress-bar {
  height: 4px;
  background: hsl(var(--muted-foreground) / 0.08);
  border-radius: 2px;
  overflow: hidden;
}

.run-progress-fill {
  height: 100%;
  background: var(--af-primary);
  border-radius: 2px;
  transition: width 0.3s ease;
}

/* Pipeline */
.run-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
}

.run-title {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--af-fg);
  font-family: 'JetBrains Mono', monospace;
}

.run-stats {
  display: flex;
  gap: 0.5rem;
}

.stat-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.7rem;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  background: hsl(var(--muted-foreground) / 0.06);
  color: var(--af-muted);
}

/* Budget bar */
.budget-bar-container {
  margin-bottom: 1rem;
}

.budget-label {
  display: flex;
  justify-content: space-between;
  font-size: 0.7rem;
  color: var(--af-muted);
  margin-bottom: 0.3rem;
}

.budget-bar {
  height: 6px;
  background: hsl(var(--muted-foreground) / 0.08);
  border-radius: 3px;
  overflow: hidden;
}

.budget-fill {
  height: 100%;
  background: hsl(142 70% 45%);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.budget-fill.warning { background: hsl(38 90% 50%); }
.budget-fill.danger { background: hsl(0 70% 50%); }

/* Pipeline flow */
.pipeline-flow {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  padding: 1rem;
  border: 1px solid var(--af-border);
  border-radius: 8px;
  overflow-x: auto;
  margin-bottom: 1rem;
}

.pipeline-step {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.2rem;
  padding: 0.5rem 0.6rem;
  border-radius: 8px;
  min-width: 72px;
  border: 1px solid transparent;
  transition: all 0.2s;
  position: relative;
}

.pipeline-step.completed {
  border-color: hsl(142 70% 45% / 0.25);
  background: hsl(142 70% 45% / 0.04);
}

.pipeline-step.running {
  border-color: hsl(var(--af-agents) / 0.4);
  background: hsl(var(--af-agents) / 0.08);
}

.pipeline-step.waiting_gate {
  border-color: hsl(38 90% 50% / 0.4);
  background: hsl(38 90% 50% / 0.08);
}

.pipeline-step.pending {
  opacity: 0.5;
}

.step-icon { font-size: 1rem; }
.step-name { font-size: 0.65rem; font-weight: 500; color: var(--af-fg); }
.step-gate { font-size: 0.6rem; position: absolute; top: 2px; right: 2px; }
.step-pulse {
  position: absolute;
  top: 2px; left: 2px;
  width: 6px; height: 6px;
  border-radius: 50%;
  background: hsl(var(--af-agents));
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.4; transform: scale(1.3); }
  100% { opacity: 1; transform: scale(1); }
}

.step-connector {
  color: var(--af-border);
  display: flex;
  align-items: center;
}

/* Gate panel */
.gate-panel {
  padding: 0.75rem 1rem;
  border: 1px solid hsl(38 90% 50% / 0.3);
  border-radius: 8px;
  background: hsl(38 90% 50% / 0.04);
  margin-bottom: 1rem;
}

.gate-header {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.8rem;
  font-weight: 500;
  color: hsl(38 80% 35%);
  margin-bottom: 0.5rem;
}

.gate-actions {
  display: flex;
  gap: 0.4rem;
}

/* History */
.history-panel {
  border: 1px solid var(--af-border);
  border-radius: 8px;
  padding: 0.75rem 1rem;
}

.history-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.3rem 0;
  border-bottom: 1px solid var(--af-border);
  font-size: 0.8rem;
}

.history-row:last-child { border-bottom: none; }
.history-profession { font-weight: 500; color: var(--af-fg); }
.history-time { color: var(--af-muted); font-family: monospace; font-size: 0.75rem; }

/* Config sidebar */
.profession-list, .soul-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.profession-item, .soul-item {
  padding: 0.4rem 0.5rem;
  border-radius: 5px;
  font-size: 0.75rem;
  background: hsl(var(--muted-foreground) / 0.04);
}

.profession-name { font-weight: 500; color: var(--af-fg); }
.profession-phase { font-size: 0.65rem; color: var(--af-muted); text-transform: capitalize; }

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.modal-content {
  background: var(--af-card);
  border: 1px solid var(--af-border);
  border-radius: 10px;
  padding: 1.25rem;
  width: 480px;
  max-width: 90vw;
  max-height: 80vh;
  overflow-y: auto;
}

.modal-content h3 {
  font-size: 0.9rem;
  font-weight: 600;
  margin-bottom: 1rem;
  color: var(--af-fg);
}

.form-group {
  margin-bottom: 0.75rem;
}

.form-group label {
  display: block;
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--af-muted);
  margin-bottom: 0.3rem;
}

.form-group input, .form-group select {
  width: 100%;
  padding: 0.4rem 0.5rem;
  border: 1px solid var(--af-border);
  border-radius: 5px;
  background: var(--af-bg);
  color: var(--af-fg);
  font-size: 0.8rem;
}

.steps-builder {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.step-row {
  display: flex;
  gap: 0.4rem;
  align-items: center;
}

.step-input { flex: 1; }
.step-select { width: 100px; }

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-top: 1rem;
}
</style>
