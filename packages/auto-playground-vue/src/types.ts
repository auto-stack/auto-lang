export interface ProjectFile {
  path: string;
  source: string;
}

export interface RunRequest {
  source: string;
  project_dir?: string;
  files?: ProjectFile[];
}

export interface RunResponse {
  stdout: string;
  stderr: string;
  exit_code: number;
  time_ms: number;
  result?: string;
  bytecode?: BytecodeLine[];
  meta?: BytecodeMeta | null;
}

export interface RunCodeRequest {
  language: string;
  code: string;
}

export interface TransRequest {
  source: string;
  target: string;
  project_dir?: string;
  files?: ProjectFile[];
}

export interface SourceMapEntry {
  source_line: number;
  output_line: number;
  source_file?: string;
}

export interface TransFile {
  path: string;
  code: string;
  source_map?: SourceMapEntry[];
}

export interface TransResponse {
  target: string;
  files: TransFile[];
  source_map: SourceMapEntry[];
}

export interface Example {
  name: string;
  source: string;
  example_type: 'single' | 'project';
  project_dir?: string;
  files?: ProjectFile[];
}

export interface ExamplesResponse {
  examples: Example[];
}

export type OutputTab = 'rust' | 'c' | 'python' | 'typescript' | 'abt' | 'bytecode';

// ── Playground 分层组件契约（Playground 设计 §4；Plan 581）──

/** SnippetRunner / PlaygroundCard 的运行目标（'run'=VM 执行，其余=转译目标）。 */
export type PlaygroundTarget = 'run' | Exclude<OutputTab, 'bytecode'>;

export interface SnippetRunnerProps {
  /** 初始代码（必填）。 */
  code: string;
  /** 后端地址；'' = 同源 /api。 */
  apiBase?: string;
  /** 挂载即运行。 */
  autorun?: boolean;
  /** 首选动作（默认 'run'）。 */
  target?: PlaygroundTarget;
  /** 容器高；'auto' = 按行数。 */
  height?: string;
}

/** PlaygroundCard 工具栏开关（默认全开）。 */
export interface PlaygroundCardToolbar {
  transpile?: boolean;
  share?: boolean;
  debug?: boolean;
  live?: boolean;
}

export interface PlaygroundCardProps extends SnippetRunnerProps {
  /** manifest 笔记 id（Notes Explorer 用）。 */
  noteId?: string;
  /** 期望输出（vm-golden 笔记）；非空时输出区加"期望输出"对照 tab（Plan 582）。 */
  expectedOutput?: string | null;
  /** 项目型笔记文件集（kind=project）；>1 文件时呈文件 tab，entry 锁 main.at（Plan 582）。 */
  files?: NoteFile[] | null;
  /** 项目目录（相对服务端 examples/playground-demo）；运行走 files 形态（Plan 582）。 */
  projectDir?: string | null;
  /** IDE 模式入口：true=可用；false=禁用+提示；null=不渲染（Plan 582）。 */
  ideMode?: boolean | null;
  /** 工具栏项开关（默认全开）。 */
  toolbar?: PlaygroundCardToolbar;
  /** 是否渲染 ExampleSelector（默认 false；旧 AutoPlayground 常驻行为需显式选入）。 */
  exampleSelector?: boolean;
}

// ── Notes manifest（notes.json schema v1，Playground 设计 §5.2）笔记元信息 ──

export type NoteSourceType = 'vm-golden' | 'aavm-corpus' | 'book' | 'demo' | 'parity';
export type NoteKind = 'single' | 'project' | 'fence';

export interface NoteFile {
  path: string;
  content: string;
}

/** manifest 单条笔记（582 Notes Explorer 复用）。 */
export interface NoteMeta {
  id: string;
  title: string;
  sourceType: NoteSourceType;
  sourcePath: string;
  kind: NoteKind;
  standalone: boolean;
  /** kind=project 时为 null，文件见 files。 */
  code: string | null;
  files: NoteFile[] | null;
  /** 仅 vm-golden（.expected.out 内容）。 */
  expectedOutput: string | null;
  /** 期望语义判别（P581-D3）：'stdout'=对照 RunResponse.stdout；'result'=对照终值；null=无期望。 */
  expectedKind?: 'stdout' | 'result' | null;
  description: string | null;
  tags: string[];
}

// Debug types
export interface BytecodeLine {
  offset: number;
  mnemonic: string;
  operands: string;
  line?: number;
}

export interface CallFrameInfo {
  fn_name: string | null;
  line: number;
  return_ip: number;
  bp: number;
  n_args: number;
  n_locals: number;
}

export interface LocalInfo {
  index: number;
  value: number;
}

export interface ArgInfo {
  index: number;
  value: number;
}

export interface RegisterInfo {
  ip: number;
  bp: number;
  sp: number;
}

export interface DebugState {
  status: 'paused' | 'running' | 'finished' | 'error';
  line: number;
  ip: number;
  op: string;
  stack: string[];
  call_stack: CallFrameInfo[];
  locals: LocalInfo[];
  args: ArgInfo[];
  registers: RegisterInfo;
  stdout: string;
  stderr: string;
  result: string | null;
}

export type DebugCommand = 'continue' | 'step' | 'step_over' | 'step_out' | 'stop';

// Replay types
export interface DebugRecording {
  version: number;
  createdAt: string;
  source: string;
  initialBreakpoints: number[];
  bytecode: BytecodeLine[];
  events: RecordingEvent[];
  meta?: BytecodeMeta | null;
}

/// Symbol tables accompanying disassembly, letting operand references
/// (`str[N]`, `nat#N`, jump/call hex targets) resolve to concrete values.
export interface BytecodeMeta {
  strings: string[];
  functions: { offset: number; name: string }[];
  natives: Record<string, string>;
}

export type RecordingEvent =
  | { type: 'state'; state: DebugState }
  | { type: 'command'; cmd: DebugCommand }
  | { type: 'breakpoints'; lines: number[] };
