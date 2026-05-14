import { ref, computed } from 'vue'

const _mode = ref<'gsd' | 'check'>('gsd')
const STORAGE_KEY = 'autoforge_mode'

// Hydrate from localStorage
const stored = localStorage.getItem(STORAGE_KEY)
if (stored === 'gsd' || stored === 'check') {
  _mode.value = stored
}

export function useForgeMode() {
  const mode = computed({
    get: () => _mode.value,
    set: (val: 'gsd' | 'check') => {
      _mode.value = val
      localStorage.setItem(STORAGE_KEY, val)
    },
  })

  const isGSD = computed(() => _mode.value === 'gsd')
  const isCheck = computed(() => _mode.value === 'check')

  // In GSD mode, only the Goal Gate (or final spec_review gate) pauses
  // In Check mode, every human gate pauses
  function shouldPauseGate(gateType: string): boolean {
    if (_mode.value === 'check') return true
    // GSD mode: only goal-level gates pause
    return gateType === 'human' || gateType === 'goal'
  }

  return {
    mode,
    isGSD,
    isCheck,
    shouldPauseGate,
  }
}
