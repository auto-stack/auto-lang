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

export interface TransResponse {
  code: string;
  target: string;
}

export interface Example {
  name: string;
  source: string;
}

export interface ExamplesResponse {
  examples: Example[];
}

export type OutputTab = 'console' | 'rust' | 'c' | 'python' | 'javascript' | 'typescript';
