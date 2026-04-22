import { ref } from 'vue';
import type { RunResponse, TransResponse, OutputTab } from '../types';

const API_BASE = '/api';

export function usePlayground() {
  const source = ref(`// Welcome to Auto Playground!
fn add(a int, b int) int {
    a + b
}

let result = add(3, 4)
print(result)`);

  const stdout = ref('');
  const stderr = ref('');
  const resultCode = ref('');
  const timeMs = ref(0);
  const isLoading = ref(false);
  const activeTab = ref<OutputTab>('console');
  const transpiledCode = ref('');
  const transpileTarget = ref('');

  // Cache transpiled code per target
  const transCache = ref<Record<string, string>>({});

  async function run() {
    isLoading.value = true;
    stdout.value = '';
    stderr.value = '';
    resultCode.value = '';

    try {
      const res = await fetch(`${API_BASE}/run`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ source: source.value }),
      });
      const data: RunResponse = await res.json();
      stdout.value = data.stdout || '';
      stderr.value = data.stderr || '';
      timeMs.value = data.time_ms || 0;
      if (data.result !== undefined && data.result !== null && data.result !== '') {
        resultCode.value = data.result;
      }
      activeTab.value = 'console';
    } catch (e: any) {
      stderr.value = `Network error: ${e.message}`;
      activeTab.value = 'console';
    } finally {
      isLoading.value = false;
    }
  }

  async function transpile(target: string) {
    activeTab.value = target as OutputTab;

    if (transCache.value[target]) {
      transpiledCode.value = transCache.value[target];
      transpileTarget.value = target;
      return;
    }

    isLoading.value = true;
    try {
      const res = await fetch(`${API_BASE}/trans`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ source: source.value, target }),
      });
      const data: TransResponse = await res.json();
      transpiledCode.value = data.code || '';
      transpileTarget.value = target;
      transCache.value[target] = transpiledCode.value;
    } catch (e: any) {
      transpiledCode.value = `Error: ${e.message}`;
      transpileTarget.value = target;
    } finally {
      isLoading.value = false;
    }
  }

  function loadExample(code: string) {
    source.value = code;
    transCache.value = {};
    stdout.value = '';
    stderr.value = '';
    resultCode.value = '';
  }

  return {
    source, stdout, stderr, resultCode, timeMs, isLoading,
    activeTab, transpiledCode, transpileTarget,
    run, transpile, loadExample,
  };
}
