import { ref, watch, computed } from 'vue';
import type { RunResponse, TransResponse, OutputTab, SourceMapEntry } from '../types';
import { runTypeScript } from '../utils/tsRunner';

const API_BASE = '/api';
const STORAGE_KEY = 'auto-playground:state';

interface PersistedState {
  source: string;
  activeTab: OutputTab;
}

const DEFAULT_SOURCE = `// Welcome to Auto Playground!
fn add(a int, b int) int {
    a + b
}

let result = add(3, 4)
print(result)`;

function loadPersistedState(): Partial<PersistedState> {
  const hash = window.location.hash;
  if (hash.startsWith('#share=')) {
    try {
      const json = atob(decodeURIComponent(hash.slice('#share='.length)));
      const parsed = JSON.parse(json) as Partial<PersistedState>;
      if (parsed.source) return parsed;
    } catch { /* ignore corrupt hash */ }
  }
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw);
  } catch { /* ignore */ }
  return {};
}

function persistState(state: PersistedState) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch { /* ignore quota errors */ }
}

export function usePlaygroundFull() {
  const saved = loadPersistedState();

  const source = ref(saved.source ?? DEFAULT_SOURCE);
  const stdout = ref('');
  const stderr = ref('');
  const resultCode = ref('');
  const timeMs = ref(0);
  const isLoading = ref(false);
  const activeTab = ref<OutputTab>(saved.activeTab ?? 'rust');
  const transpiledCode = ref('');
  const transpileTarget = ref('');

  interface TransCacheEntry {
    code: string;
    sourceMap: SourceMapEntry[];
  }
  const transCache = ref<Record<string, TransCacheEntry>>({});
  const sourceMap = ref<SourceMapEntry[]>([]);
  const highlightedSourceLine = ref<number | null>(null);
  const highlightedOutputLines = ref<number[]>([]);
  const shareToast = ref<{ message: string; visible: boolean }>({ message: '', visible: false });

  const sourceToOutputMap = computed(() => {
    const map: Record<number, number[]> = {};
    for (const entry of sourceMap.value) {
      if (!map[entry.source_line]) {
        map[entry.source_line] = [];
      }
      map[entry.source_line].push(entry.output_line);
    }
    return map;
  });

  function highlightSourceLine(line: number) {
    highlightedSourceLine.value = line;
    highlightedOutputLines.value = sourceToOutputMap.value[line] ?? [];
  }

  function clearHighlight() {
    highlightedSourceLine.value = null;
    highlightedOutputLines.value = [];
  }

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
    } catch (e: any) {
      stderr.value = `Network error: ${e.message}`;
    } finally {
      isLoading.value = false;
    }
  }

  async function runAbt() {
    isLoading.value = true;
    stdout.value = '';
    stderr.value = '';
    resultCode.value = '';

    try {
      const abtCode = transCache.value['abt']?.code || '';
      const res = await fetch(`${API_BASE}/run_abt`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ abt: abtCode }),
      });
      const data: RunResponse = await res.json();
      stdout.value = data.stdout || '';
      stderr.value = data.stderr || '';
      timeMs.value = data.time_ms || 0;
      if (data.result !== undefined && data.result !== null && data.result !== '') {
        resultCode.value = data.result;
      }
    } catch (e: any) {
      stderr.value = `Network error: ${e.message}`;
    } finally {
      isLoading.value = false;
    }
  }

  async function runCode(language: string) {
    isLoading.value = true;
    stdout.value = '';
    stderr.value = '';
    resultCode.value = '';
    timeMs.value = 0;

    const code = transCache.value[language]?.code || '';
    if (!code.trim()) {
      stderr.value = `No ${language} code to run. Make sure the transpilation succeeded.`;
      isLoading.value = false;
      return;
    }

    try {
      if (language === 'typescript') {
        // Run TypeScript directly in the browser
        const result = await runTypeScript(code);
        stdout.value = result.stdout;
        stderr.value = result.stderr;
        timeMs.value = 0;
      } else {
        // Run Python, Rust, C through backend
        const res = await fetch(`${API_BASE}/run_code`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ language, code }),
        });
        const data: RunResponse = await res.json();
        stdout.value = data.stdout || '';
        stderr.value = data.stderr || '';
        timeMs.value = data.time_ms || 0;
        if (data.result !== undefined && data.result !== null && data.result !== '') {
          resultCode.value = data.result;
        }
      }
    } catch (e: any) {
      stderr.value = `Network error: ${e.message}`;
    } finally {
      isLoading.value = false;
    }
  }

  async function transpile(target: string) {
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
      sourceMap.value = data.source_map || [];
      transCache.value[target] = { code: transpiledCode.value, sourceMap: data.source_map || [] };
    } catch (e: any) {
      transpiledCode.value = `Error: ${e.message}`;
      transpileTarget.value = target;
    } finally {
      isLoading.value = false;
    }
  }

  function switchTab(target: OutputTab) {
    activeTab.value = target;
    transpileTarget.value = target;
    const cached = transCache.value[target];
    if (cached) {
      transpiledCode.value = cached.code;
      sourceMap.value = cached.sourceMap;
    } else {
      transpiledCode.value = '';
      sourceMap.value = [];
    }
  }

  function loadExample(code: string) {
    source.value = code;
    stdout.value = '';
    stderr.value = '';
    resultCode.value = '';
    sourceMap.value = [];
    highlightedSourceLine.value = null;
    highlightedOutputLines.value = [];
  }

  function getShareUrl(): string {
    const payload = JSON.stringify({
      source: source.value,
      activeTab: activeTab.value,
    });
    const hash = '#share=' + encodeURIComponent(btoa(payload));
    return window.location.origin + window.location.pathname + hash;
  }

  async function share() {
    const url = getShareUrl();
    let ok = false;
    try {
      await navigator.clipboard.writeText(url);
      ok = true;
    } catch {
      const ta = document.createElement('textarea');
      ta.value = url;
      document.body.appendChild(ta);
      ta.select();
      try {
        ok = document.execCommand('copy');
      } catch { /* ignore */ }
      document.body.removeChild(ta);
    }
    shareToast.value = {
      message: ok ? 'Share link copied to clipboard!' : 'Failed to copy link',
      visible: true,
    };
    setTimeout(() => {
      shareToast.value.visible = false;
    }, 2500);
  }

  watch(source, () => {
    // Invalidate cached transpilation results when source changes,
    // but do NOT auto-transpile (explicit Run/Trans/Debug actions only).
    transCache.value = {};
    if (transpileTarget.value) {
      transpiledCode.value = '';
      transpileTarget.value = '';
      sourceMap.value = [];
    }
  });

  watch([source, activeTab], ([s, t]) => {
    persistState({ source: s, activeTab: t });
  }, { deep: true });

  return {
    source, stdout, stderr, resultCode, timeMs, isLoading,
    activeTab, transpiledCode, transpileTarget,
    sourceMap, highlightedSourceLine, highlightedOutputLines,
    shareToast,
    run, runAbt, runCode, transpile, switchTab, loadExample, highlightSourceLine, clearHighlight,
    share,
  };
}
