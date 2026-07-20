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
}

export type RecordingEvent =
  | { type: 'state'; state: DebugState }
  | { type: 'command'; cmd: DebugCommand }
  | { type: 'breakpoints'; lines: number[] };
