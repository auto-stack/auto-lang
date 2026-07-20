import { ref, watch, computed } from 'vue';
import type { RunResponse, TransResponse, OutputTab, SourceMapEntry, TransFile, DebugState, DebugCommand, BytecodeLine } from '../types';
import { runTypeScript } from '../utils/tsRunner';

const DEBOUNCE_MS = 500;

interface PersistedState {
  source: string;
  activeTab: OutputTab;
  liveCompile: boolean;
  projectDir?: string;
}

export interface UsePlaygroundOptions {
  apiBase?: string;
  defaultSource?: string;
  persistKey?: string | false;
  preloadTargets?: boolean;
}

const DEFAULT_SOURCE = `// Welcome to Auto Playground!
fn add(a int, b int) int {
    a + b
}

let result = add(3, 4)
print(result)`;

export function usePlayground(options: UsePlaygroundOptions = {}) {
  const API_BASE = options.apiBase ?? '/api';
  const persistKey = options.persistKey ?? 'auto-playground:state';
  const defaultSource = options.defaultSource ?? DEFAULT_SOURCE;
  const preloadTargets = options.preloadTargets ?? true;

  function loadPersistedState(): Partial<PersistedState> {
    if (typeof window === 'undefined') return {};
    const hash = window.location.hash;
    if (hash.startsWith('#share=')) {
      try {
        const json = atob(decodeURIComponent(hash.slice('#share='.length)));
        const parsed = JSON.parse(json) as Partial<PersistedState>;
        if (parsed.source) return parsed;
      } catch { /* ignore corrupt hash */ }
    }
    if (persistKey === false) return {};
    try {
      const raw = localStorage.getItem(persistKey);
      if (raw) return JSON.parse(raw);
    } catch { /* ignore */ }
    return {};
  }

  function persistState(state: PersistedState) {
    if (typeof window === 'undefined') return;
    if (persistKey === false) return;
    try {
      localStorage.setItem(persistKey, JSON.stringify(state));
    } catch { /* ignore quota errors */ }
  }

  const saved = loadPersistedState();

  const source = ref(saved.source ?? defaultSource);
  const stdout = ref('');
  const stderr = ref('');
  const resultCode = ref('');
  const timeMs = ref(0);
  const bytecode = ref<BytecodeLine[]>([]);
  const isLoading = ref(false);
  const activeTab = ref<OutputTab>(saved.activeTab ?? 'rust');
  const transpileTarget = ref('');
  const liveCompile = ref(saved.liveCompile ?? true);
  const projectDir = ref<string | undefined>(saved.projectDir);

  interface OutputLocation {
    outputFile: string;
    outputLines: number[];
  }

  interface TransCacheEntry {
    files: TransFile[];
    fileSourceMaps: Record<string, SourceMapEntry[]>;
    selectedFile: string;
  }
  const transCache = ref<Record<string, TransCacheEntry>>({});
  const highlightedSourceLine = ref<number | null>(null);
  const highlightedOutputLines = ref<number[]>([]);
  const highlightedOutputFiles = ref<string[]>([]);
  const shareToast = ref<{ message: string; visible: boolean }>({ message: '', visible: false });
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

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

  const activeSourceFile = computed(() => '');

  // Forward map: (sourceFile, sourceLine) -> output files with their mapped output lines
  const forwardSourceMap = computed(() => {
    const target = transpileTarget.value;
    const map = new Map<string, Map<number, OutputLocation[]>>();
    if (!target) return map;
    const cached = transCache.value[target];
    if (!cached) return map;
    for (const file of cached.files) {
      const entries = cached.fileSourceMaps[file.path] ?? [];
      for (const entry of entries) {
        const sourceFile = entry.source_file || activeSourceFile.value;
        if (!map.has(sourceFile)) map.set(sourceFile, new Map());
        const fileMap = map.get(sourceFile)!;
        if (!fileMap.has(entry.source_line)) fileMap.set(entry.source_line, []);
        const locations = fileMap.get(entry.source_line)!;
        const loc = locations.find((l) => l.outputFile === file.path);
        if (loc) {
          if (!loc.outputLines.includes(entry.output_line)) loc.outputLines.push(entry.output_line);
        } else {
          locations.push({ outputFile: file.path, outputLines: [entry.output_line] });
        }
      }
    }
    return map;
  });

  // Reverse map: (outputFile, outputLine) -> originating source file and line
  const reverseSourceMap = computed(() => {
    const target = transpileTarget.value;
    const map = new Map<string, Map<number, { sourceFile: string; sourceLine: number }>>();
    if (!target) return map;
    const cached = transCache.value[target];
    if (!cached) return map;
    for (const file of cached.files) {
      const entries = cached.fileSourceMaps[file.path] ?? [];
      for (const entry of entries) {
        const sourceFile = entry.source_file || activeSourceFile.value;
        if (!map.has(file.path)) map.set(file.path, new Map());
        map.get(file.path)!.set(entry.output_line, { sourceFile, sourceLine: entry.source_line });
      }
    }
    return map;
  });

  function refreshOutputHighlight() {
    if (highlightedSourceLine.value) {
      highlightSourceLine(highlightedSourceLine.value);
    } else {
      highlightedOutputLines.value = [];
      highlightedOutputFiles.value = [];
    }
  }

  function highlightSourceLine(line: number) {
    highlightedSourceLine.value = line;
    const sourceFile = activeSourceFile.value;
    const locations = forwardSourceMap.value.get(sourceFile)?.get(line) ?? [];
    highlightedOutputFiles.value = locations.map((l) => l.outputFile);
    const currentOutputFile = selectedTransFile.value;
    const currentLoc = locations.find((l) => l.outputFile === currentOutputFile);
    highlightedOutputLines.value = currentLoc?.outputLines ?? [];
  }

  function highlightOutputLine(outputFile: string, outputLine: number) {
    const loc = reverseSourceMap.value.get(outputFile)?.get(outputLine);
    if (!loc) {
      clearHighlight();
      return;
    }
    highlightedSourceLine.value = loc.sourceLine;
    const locations = forwardSourceMap.value.get(loc.sourceFile)?.get(loc.sourceLine) ?? [];
    highlightedOutputFiles.value = locations.map((l) => l.outputFile);
    const currentLoc = locations.find((l) => l.outputFile === outputFile);
    highlightedOutputLines.value = currentLoc?.outputLines ?? [];
  }

  function getSourceFileForOutputLine(outputFile: string, outputLine: number): string | undefined {
    return reverseSourceMap.value.get(outputFile)?.get(outputLine)?.sourceFile;
  }

  function clearHighlight() {
    highlightedSourceLine.value = null;
    highlightedOutputLines.value = [];
    highlightedOutputFiles.value = [];
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
    const cached = transCache.value[language];
    const code = cached?.files[0]?.code ?? '';
    if (!code.trim()) {
      stderr.value = `No ${language} code to run. Make sure the transpilation succeeded.`;
      return;
    }

    isLoading.value = true;
    stdout.value = '';
    stderr.value = '';
    resultCode.value = '';

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
      const fileSourceMaps: Record<string, SourceMapEntry[]> = {};
      for (const f of files) {
        fileSourceMaps[f.path] = f.source_map ?? data.source_map ?? [];
      }
      const selected = files[0]?.path ?? '';
      transpileTarget.value = target;
      transCache.value[target] = {
        files,
        fileSourceMaps,
        selectedFile: selected,
      };
      refreshOutputHighlight();
    } catch (e: any) {
      transpileTarget.value = target;
      transCache.value[target] = {
        files: [{ path: 'error.txt', code: `Error: ${e.message}` }],
        fileSourceMaps: { 'error.txt': [] },
        selectedFile: 'error.txt',
      };
      refreshOutputHighlight();
    } finally {
      isLoading.value = false;
    }
  }

  function switchTab(target: OutputTab) {
    activeTab.value = target;
    transpileTarget.value = target;
    const cached = transCache.value[target];
    if (!cached && liveCompile.value) {
      transpile(target);
      return;
    }
    refreshOutputHighlight();
  }

  function selectTransFile(target: string, path: string) {
    const cached = transCache.value[target];
    if (!cached) return;
    cached.selectedFile = path;
    refreshOutputHighlight();
  }

  function loadExample(payload: { source: string; project_dir?: string }) {
    source.value = payload.source;
    projectDir.value = payload.project_dir;
    stdout.value = '';
    stderr.value = '';
    resultCode.value = '';
    bytecode.value = [];
    highlightedSourceLine.value = null;
    highlightedOutputLines.value = [];
    highlightedOutputFiles.value = [];
  }

  function getShareUrl(): string {
    if (typeof window === 'undefined') return '';
    const payload = JSON.stringify({
      source: source.value,
      activeTab: activeTab.value,
      liveCompile: liveCompile.value,
      projectDir: projectDir.value,
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
    transCache.value = {};

    if (liveCompile.value) {
      if (debounceTimer) clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        transpile(activeTab.value);
      }, DEBOUNCE_MS);
    }
  });

  watch([source, activeTab, liveCompile, projectDir], ([s, t, l, d]) => {
    persistState({ source: s, activeTab: t, liveCompile: l, projectDir: d });
  }, { deep: true });

  // Initial transpile on load: fetch all targets in parallel so every tab has cached content
  if (preloadTargets && typeof window !== 'undefined') {
    setTimeout(() => {
      transpileAll();
    }, 100);
  }

  async function transpileAll() {
    const targets: OutputTab[] = ['rust', 'c', 'python', 'typescript', 'abt'];
    isLoading.value = true;
    try {
      const results = await Promise.all(
        targets.map(async (target) => {
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
            const fileSourceMaps: Record<string, SourceMapEntry[]> = {};
            for (const f of files) {
              fileSourceMaps[f.path] = f.source_map ?? data.source_map ?? [];
            }
            return {
              target,
              files,
              fileSourceMaps,
              selectedFile: files[0]?.path ?? '',
            };
          } catch (e: any) {
            return {
              target,
              files: [{ path: 'error.txt', code: `Error: ${e.message}` }],
              fileSourceMaps: { 'error.txt': [] },
              selectedFile: 'error.txt',
            };
          }
        })
      );
      for (const r of results) {
        transCache.value[r.target] = {
          files: r.files,
          fileSourceMaps: r.fileSourceMaps,
          selectedFile: r.selectedFile,
        };
      }
      const current = activeTab.value;
      const cached = transCache.value[current];
      if (cached) {
        transpileTarget.value = current;
        refreshOutputHighlight();
      }
    } finally {
      isLoading.value = false;
    }
  }

  // ── Debug state ──
  const debugSessionId = ref<string | null>(null);
  const debugState = ref<DebugState | null>(null);
  const debugBytecode = ref<BytecodeLine[]>([]);
  const breakpoints = ref<number[]>([]);
  const isDebugging = ref(false);

  async function debugStart() {
    isLoading.value = true;
    stderr.value = '';
    try {
      const res = await fetch(`${API_BASE}/agent-debug/start`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ source: source.value }),
      });
      const data = await res.json();
      debugSessionId.value = data.session_id;
      debugBytecode.value = (data.bytecode || []).map((b: any) => ({
        offset: b.offset ?? b.idx ?? 0,
        mnemonic: b.mnemonic ?? b.op ?? '',
        operands: b.operands ?? b.args ?? '',
        line: b.line,
      }));
      isDebugging.value = true;
      debugState.value = null;
      if (breakpoints.value.length > 0) {
        await debugSetBreakpoints(breakpoints.value);
      }
    } catch (e: any) {
      stderr.value = `Debug start error: ${e.message}`;
    } finally {
      isLoading.value = false;
    }
  }

  async function debugSetBreakpoints(lines: number[]) {
    breakpoints.value = lines;
    if (!debugSessionId.value) return;
    try {
      await fetch(`${API_BASE}/agent-debug/${debugSessionId.value}/breakpoints`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ lines }),
      });
    } catch { /* ignore */ }
  }

  async function debugCommand(cmd: DebugCommand) {
    if (!debugSessionId.value) return;
    isLoading.value = true;
    try {
      const res = await fetch(`${API_BASE}/agent-debug/${debugSessionId.value}/command`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ cmd }),
      });
      const data: DebugState = await res.json();
      debugState.value = data;
      if (data.stdout) stdout.value = data.stdout;
      if (data.stderr) stderr.value = data.stderr;
      if (data.result) resultCode.value = data.result;
      if (data.status === 'finished' || data.status === 'error') {
        isDebugging.value = false;
      }
    } catch (e: any) {
      stderr.value = `Debug command error: ${e.message}`;
    } finally {
      isLoading.value = false;
    }
  }

  async function debugStop() {
    if (!debugSessionId.value) return;
    try {
      await fetch(`${API_BASE}/agent-debug/${debugSessionId.value}`, {
        method: 'DELETE',
      });
    } catch { /* ignore */ }
    debugSessionId.value = null;
    debugState.value = null;
    debugBytecode.value = [];
    isDebugging.value = false;
  }

  return {
    source, stdout, stderr, resultCode, timeMs, runBytecode: bytecode, isLoading,
    activeTab, transpiledCode, transpileTarget, liveCompile, projectDir,
    transFiles, selectedTransFile,
    highlightedSourceLine, highlightedOutputLines, highlightedOutputFiles,
    shareToast,
    // Debug state
    debugSessionId, debugState, bytecode: debugBytecode, breakpoints, isDebugging,
    // Methods
    run, runCode, transpile, switchTab, selectTransFile, loadExample, highlightSourceLine, highlightOutputLine, getSourceFileForOutputLine, clearHighlight,
    share,
    debugStart, debugSetBreakpoints, debugCommand, debugStop,
  };
}
