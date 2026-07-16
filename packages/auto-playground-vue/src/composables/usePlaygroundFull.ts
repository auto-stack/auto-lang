import { ref, watch, computed } from 'vue';
import type { RunResponse, TransResponse, OutputTab, SourceMapEntry, TransFile } from '../types';
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
  const bytecode = ref<any[]>([]);
  const isLoading = ref(false);
  const activeTab = ref<OutputTab>(saved.activeTab ?? 'rust');
  const transpileTarget = ref('');
  const projectDir = ref<string | undefined>(undefined);

  interface TransCacheEntry {
    files: TransFile[];
    sourceMap: SourceMapEntry[];
    selectedFile: string;
  }
  const transCache = ref<Record<string, TransCacheEntry>>({});
  const sourceMap = ref<SourceMapEntry[]>([]);
  const highlightedSourceLine = ref<number | null>(null);
  const highlightedOutputLines = ref<number[]>([]);
  const shareToast = ref<{ message: string; visible: boolean }>({ message: '', visible: false });

  const transpiledCode = computed(() => {
    const target = transpileTarget.value;
    if (!target) return '';
    const cached = transCache.value[target];
    if (!cached) return '';
    const file = cached.files.find((f) => f.path === cached.selectedFile);
    return file?.code ?? cached.files[0]?.code ?? '';
  });

  const transFiles = computed(() => {
    const target = transpileTarget.value;
    if (!target) return [];
    return transCache.value[target]?.files ?? [];
  });

  const selectedTransFile = computed(() => {
    const target = transpileTarget.value;
    if (!target) return '';
    return transCache.value[target]?.selectedFile ?? '';
  });

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
    bytecode.value = [];

    try {
      const body: Record<string, unknown> = { source: source.value };
      if (projectDir.value) {
        body.project_dir = projectDir.value;
      }
      const res = await fetch(`${API_BASE}/run`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });
      const data: RunResponse = await res.json();
      stdout.value = data.stdout || '';
      stderr.value = data.stderr || '';
      timeMs.value = data.time_ms || 0;
      bytecode.value = data.bytecode || [];
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

    const cached = transCache.value[language];
    const code = cached?.files[0]?.code ?? '';
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
      const body: Record<string, unknown> = { source: source.value, target };
      if (projectDir.value) {
        body.project_dir = projectDir.value;
      }
      const res = await fetch(`${API_BASE}/trans`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });
      const data: TransResponse = await res.json();
      const files = data.files ?? [];
      const selected = files[0]?.path ?? '';
      transpileTarget.value = target;
      sourceMap.value = data.source_map || [];
      transCache.value[target] = {
        files,
        sourceMap: data.source_map || [],
        selectedFile: selected,
      };
    } catch (e: any) {
      transpileTarget.value = target;
      transCache.value[target] = {
        files: [{ path: 'error.txt', code: `Error: ${e.message}` }],
        sourceMap: [],
        selectedFile: 'error.txt',
      };
    } finally {
      isLoading.value = false;
    }
  }

  function switchTab(target: OutputTab) {
    activeTab.value = target;
    transpileTarget.value = target;
    const cached = transCache.value[target];
    if (cached) {
      sourceMap.value = cached.sourceMap;
    } else {
      sourceMap.value = [];
    }
  }

  function selectTransFile(target: string, path: string) {
    const cached = transCache.value[target];
    if (!cached) return;
    cached.selectedFile = path;
    sourceMap.value = cached.sourceMap;
  }

  function loadExample(payload: { source: string; project_dir?: string }) {
    source.value = payload.source;
    projectDir.value = payload.project_dir;
    stdout.value = '';
    stderr.value = '';
    resultCode.value = '';
    bytecode.value = [];
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
      transpileTarget.value = '';
      sourceMap.value = [];
    }
  });

  watch([source, activeTab], ([s, t]) => {
    persistState({ source: s, activeTab: t });
  }, { deep: true });

  return {
    source, stdout, stderr, resultCode, timeMs, bytecode, isLoading,
    activeTab, transpiledCode, transpileTarget, projectDir,
    transFiles, selectedTransFile,
    sourceMap, highlightedSourceLine, highlightedOutputLines,
    shareToast,
    run, runCode, transpile, switchTab, selectTransFile, loadExample, highlightSourceLine, clearHighlight,
    share,
  };
}
