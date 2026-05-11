<template>
  <div class="order-view">
    <div class="order-header">
      <h2>The Order · 法阵</h2>
      <div class="order-stats">
        <span class="stat">Active: 1</span>
        <span class="stat">Completed: 47</span>
        <span class="stat">Failed: 2</span>
      </div>
    </div>
    <div class="order-body">
      <div class="pipeline-flow">
        <div
          v-for="(role, idx) in pipeline"
          :key="role.name"
          class="pipeline-node"
          :class="role.status"
        >
          <div class="node-icon">{{ role.icon }}</div>
          <div class="node-name">{{ role.name }}</div>
          <div class="node-meta">{{ role.tokens }}k tk</div>
          <div class="node-meta">{{ role.time }}s</div>
          <div v-if="idx < pipeline.length - 1" class="node-arrow">→</div>
        </div>
      </div>
      <div class="runs-list">
        <div class="run-card active">
          <div class="run-header">
            <span class="run-id">Run #42</span>
            <span class="run-badge active">Active</span>
            <span class="run-budget">150k / 200k tokens</span>
          </div>
          <div class="run-title">JWT Auth Implementation</div>
          <div class="run-actions">
            <button class="run-btn">Pause</button>
            <button class="run-btn">Rollback</button>
            <button class="run-btn">Checkpoint</button>
          </div>
        </div>
      </div>
      <div class="cost-panel">
        <div class="cost-title">Cost Analytics</div>
        <div class="cost-row">
          <span>This session</span>
          <span class="cost-value">$1.24</span>
        </div>
        <div class="cost-row saved">
          <span>Saved vs parallel</span>
          <span class="cost-value saved">$8.30</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const pipeline = [
  { name: 'Planner', icon: '📋', status: 'completed', tokens: 5, time: 3.2 },
  { name: 'Architect', icon: '📐', status: 'completed', tokens: 15, time: 8.1 },
  { name: 'Coder', icon: '💻', status: 'active', tokens: 45, time: 14.5 },
  { name: 'Tester', icon: '🧪', status: 'pending', tokens: 0, time: 0 },
  { name: 'Reviewer', icon: '👁', status: 'pending', tokens: 0, time: 0 },
]
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
  padding: 0.75rem 1rem;
  background: #181825;
  border-bottom: 1px solid #313244;
  flex-shrink: 0;
}

.order-header h2 {
  font-size: 1rem;
  font-weight: 600;
  color: #89dceb;
}

.order-stats {
  display: flex;
  gap: 1rem;
  font-size: 0.8rem;
}

.stat {
  color: #a6adc8;
}

.order-body {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.pipeline-flow {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 1rem;
  background: #181825;
  border: 1px solid #313244;
  border-radius: 8px;
  overflow-x: auto;
}

.pipeline-node {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
  padding: 0.75rem 1rem;
  border-radius: 8px;
  min-width: 90px;
  border: 2px solid transparent;
}

.pipeline-node.completed {
  border-color: #a6e3a144;
  background: #a6e3a111;
}

.pipeline-node.active {
  border-color: #89dceb44;
  background: #89dceb11;
  animation: pulse 2s infinite;
}

.pipeline-node.pending {
  border-color: #313244;
  background: #1e1e2e;
  opacity: 0.6;
}

@keyframes pulse {
  0%, 100% { box-shadow: 0 0 0 0 #89dceb22; }
  50% { box-shadow: 0 0 0 8px #89dceb00; }
}

.node-icon {
  font-size: 1.25rem;
}

.node-name {
  font-size: 0.75rem;
  font-weight: 600;
  color: #cdd6f4;
}

.node-meta {
  font-size: 0.65rem;
  color: #6c7086;
}

.node-arrow {
  font-size: 1.25rem;
  color: #45475a;
  margin: 0 0.25rem;
}

.runs-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.run-card {
  background: #181825;
  border: 1px solid #313244;
  border-radius: 8px;
  padding: 1rem;
}

.run-card.active {
  border-color: #89dceb44;
}

.run-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.5rem;
}

.run-id {
  font-weight: 700;
  font-size: 0.9rem;
}

.run-badge {
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
}

.run-badge.active {
  background: #89dceb22;
  color: #89dceb;
}

.run-budget {
  font-size: 0.75rem;
  color: #6c7086;
  margin-left: auto;
}

.run-title {
  font-size: 0.85rem;
  color: #a6adc8;
  margin-bottom: 0.75rem;
}

.run-actions {
  display: flex;
  gap: 0.5rem;
}

.run-btn {
  background: #313244;
  color: #cdd6f4;
  border: 1px solid #45475a;
  border-radius: 4px;
  padding: 0.3rem 0.6rem;
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.15s;
}

.run-btn:hover {
  background: #45475a;
}

.cost-panel {
  background: #181825;
  border: 1px solid #313244;
  border-radius: 8px;
  padding: 1rem;
}

.cost-title {
  font-size: 0.85rem;
  font-weight: 600;
  margin-bottom: 0.75rem;
  color: #cdd6f4;
}

.cost-row {
  display: flex;
  justify-content: space-between;
  font-size: 0.85rem;
  color: #a6adc8;
  padding: 0.35rem 0;
}

.cost-row.saved {
  border-top: 1px solid #313244;
  margin-top: 0.35rem;
  padding-top: 0.7rem;
}

.cost-value {
  font-weight: 600;
  color: #cdd6f4;
}

.cost-value.saved {
  color: #a6e3a1;
}
</style>
