export interface RunRequest {
  source: string;
}

export interface RunResponse {
  stdout: string;
  stderr: string;
  exit_code: number;
  time_ms: number;
  result?: string;
}

export interface TransRequest {
  source: string;
  target: string;
}

export interface SourceMapEntry {
  source_line: number;
  output_line: number;
}

export interface TransResponse {
  code: string;
  target: string;
  source_map: SourceMapEntry[];
}

export interface Example {
  name: string;
  source: string;
}

export interface ExamplesResponse {
  examples: Example[];
}

export type OutputTab = 'rust' | 'c' | 'python' | 'typescript' | 'bytecode';

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
}

export interface LocalInfo {
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
  registers: RegisterInfo;
  stdout: string;
  stderr: string;
  result: string | null;
}

export type DebugCommand = 'continue' | 'step' | 'step_over' | 'step_out' | 'stop';
