import { ref, watch, computed } from 'vue';
import type { RunResponse, TransResponse, OutputTab, SourceMapEntry, TransFile, ProjectFile } from '../types';
import { runTypeScript } from '../utils/tsRunner';

const API_BASE = '/api';
const STORAGE_KEY = 'auto-playground:state';

interface PersistedState {
  source: string;
  activeTab: OutputTab;
  projectDir?: string;
  projectFiles?: ProjectFile[];
  activeFile?: string;
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
  const projectDir = ref<string | undefined>(saved.projectDir);
  const projectFiles = ref<ProjectFile[]>(saved.projectFiles ?? []);
  const activeFile = ref<string>(saved.activeFile ?? '');

  // Write the current editor buffer back into the projectFiles entry it
  // belongs to, so run/transpile always see the latest edits.
  function syncActiveBuffer() {
    if (!activeFile.value) return;
    const file = projectFiles.value.find((f) => f.path === activeFile.value);
    if (file) file.source = source.value;
  }

  function selectFile(path: string) {
    if (path === activeFile.value) return;
    syncActiveBuffer();
    activeFile.value = path;
    const file = projectFiles.value.find((f) => f.path === path);
    if (file) source.value = file.source;
  }

  // Request body for run/transpile: for project examples the entry source is
  // main.at's (possibly edited) content plus the full edited file set.
  function projectRequestBody(body: Record<string, unknown>) {
    if (projectDir.value) {
      syncActiveBuffer();
      body.project_dir = projectDir.value;
      body.files = projectFiles.value;
      const main = projectFiles.value.find((f) => f.path === 'main.at');
      if (main) body.source = main.source;
    }
    return body;
  }

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

  const activeSourceFile = computed(() => activeFile.value || '');

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

  const mappedSourceFiles = computed(() => {
    const target = transpileTarget.value;
    const set = new Set<string>();
    if (!target) return set;
    const cached = transCache.value[target];
    if (!cached) return set;
    for (const file of cached.files) {
      for (const entry of cached.fileSourceMaps[file.path] ?? []) {
        set.add(entry.source_file || activeSourceFile.value);
      }
    }
    return set;
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
    if (loc.sourceFile && projectFiles.value.length > 0 && activeFile.value !== loc.sourceFile) {
      selectFile(loc.sourceFile);
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
      const body: Record<string, unknown> = projectRequestBody({ source: source.value });
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
      const body: Record<string, unknown> = projectRequestBody({ source: source.value, target });
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
    refreshOutputHighlight();
  }

  function selectTransFile(target: string, path: string) {
    const cached = transCache.value[target];
    if (!cached) return;
    cached.selectedFile = path;
    refreshOutputHighlight();
  }

  function loadExample(payload: { source: string; project_dir?: string; files?: ProjectFile[] }) {
    source.value = payload.source;
    projectDir.value = payload.project_dir;
    projectFiles.value = payload.files ?? [];
    activeFile.value = payload.files?.length ? 'main.at' : '';
    stdout.value = '';
    stderr.value = '';
    resultCode.value = '';
    bytecode.value = [];
    highlightedSourceLine.value = null;
    highlightedOutputLines.value = [];
    highlightedOutputFiles.value = [];
  }

  function getShareUrl(): string {
    const payload = JSON.stringify({
      source: source.value,
      activeTab: activeTab.value,
      projectDir: projectDir.value,
      projectFiles: projectFiles.value.length ? projectFiles.value : undefined,
      activeFile: activeFile.value || undefined,
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
      highlightedOutputLines.value = [];
      highlightedOutputFiles.value = [];
    }
  });

  watch([source, activeTab, projectDir, projectFiles, activeFile], ([s, t, d, f, a]) => {
    persistState({ source: s, activeTab: t, projectDir: d, projectFiles: f, activeFile: a });
  }, { deep: true });

  return {
    source, stdout, stderr, resultCode, timeMs, bytecode, isLoading,
    activeTab, transpiledCode, transpileTarget, projectDir,
    projectFiles, activeFile,
    transFiles, selectedTransFile,
    highlightedSourceLine, highlightedOutputLines, highlightedOutputFiles, mappedSourceFiles,
    shareToast,
    run, runCode, transpile, switchTab, selectTransFile, selectFile, loadExample, highlightSourceLine, highlightOutputLine, getSourceFileForOutputLine, clearHighlight,
    share,
  };
}
