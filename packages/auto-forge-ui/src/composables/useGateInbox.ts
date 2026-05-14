import { ref, computed } from 'vue'

export interface PendingGate {
  gateId: string
  runId: string
  profession: string
  title: string
  sectionId?: string
  since: number
  status: 'pending' | 'reviewing' | 'approved' | 'rejected' | 'snoozed'
}

// ─── Singleton state ────────────────────────────────────────────────────────
const _gates = ref<PendingGate[]>([])
const _currentSecretary = ref<PendingGate | null>(null)

function promoteNext() {
  const next = _gates.value.find((g) => g.status === 'pending')
  _currentSecretary.value = next ?? null
}

export function useGateInbox() {
  const gates = _gates
  const currentSecretary = _currentSecretary
  const badgeCount = computed(() => _gates.value.filter((g) => g.status === 'pending').length)
  const hasPending = computed(() => badgeCount.value > 0)

  function registerGate(gate: Omit<PendingGate, 'status'>) {
    const existing = _gates.value.find((g) => g.gateId === gate.gateId)
    if (existing) return
    const newGate: PendingGate = { ...gate, status: 'pending' }
    _gates.value.push(newGate)
    if (!_currentSecretary.value) {
      _currentSecretary.value = newGate
    }
  }

  function resolveGate(gateId: string, decision: 'approved' | 'rejected') {
    const gate = _gates.value.find((g) => g.gateId === gateId)
    if (gate) {
      gate.status = decision
    }
    if (_currentSecretary.value?.gateId === gateId) {
      promoteNext()
    }
  }

  function dismissSecretary() {
    // Dismiss current secretary view without resolving — next pending gate shows
    promoteNext()
  }

  function snoozeGate(gateId: string) {
    const gate = _gates.value.find((g) => g.gateId === gateId)
    if (gate) {
      gate.status = 'snoozed'
      if (_currentSecretary.value?.gateId === gateId) {
        promoteNext()
      }
    }
  }

  function wakeSnoozed() {
    for (const gate of _gates.value) {
      if (gate.status === 'snoozed') {
        gate.status = 'pending'
      }
    }
    if (!_currentSecretary.value) {
      promoteNext()
    }
  }

  function clearResolved() {
    _gates.value = _gates.value.filter((g) => g.status !== 'approved' && g.status !== 'rejected')
    if (_currentSecretary.value && !['approved', 'rejected'].includes(_currentSecretary.value.status)) {
      return
    }
    promoteNext()
  }

  return {
    gates,
    currentSecretary,
    badgeCount,
    hasPending,
    registerGate,
    resolveGate,
    dismissSecretary,
    snoozeGate,
    wakeSnoozed,
    clearResolved,
  }
}
