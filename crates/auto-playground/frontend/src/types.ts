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

export type OutputTab = 'rust' | 'c' | 'python' | 'typescript';
