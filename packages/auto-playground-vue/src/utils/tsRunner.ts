interface TSRunnerResult {
  stdout: string;
  stderr: string;
}

let tsLoaded = false;
let tsLoadingPromise: Promise<void> | null = null;

function loadTypeScript(): Promise<void> {
  if (tsLoaded) return Promise.resolve();
  if (tsLoadingPromise) return tsLoadingPromise;

  tsLoadingPromise = new Promise((resolve, reject) => {
    if ((window as any).ts) {
      tsLoaded = true;
      resolve();
      return;
    }
    const script = document.createElement('script');
    script.src = 'https://cdn.jsdelivr.net/npm/typescript@5.7.3/lib/typescript.js';
    script.onload = () => {
      tsLoaded = true;
      resolve();
    };
    script.onerror = () => reject(new Error('Failed to load TypeScript compiler'));
    document.head.appendChild(script);
  });

  return tsLoadingPromise;
}

export async function runTypeScript(code: string): Promise<TSRunnerResult> {
  try {
    await loadTypeScript();
  } catch (e) {
    return {
      stdout: '',
      stderr: `Failed to load TypeScript compiler: ${e}`,
    };
  }

  const ts = (window as any).ts;
  if (!ts) {
    return { stdout: '', stderr: 'TypeScript compiler not available' };
  }

  // Transpile TypeScript to JavaScript
  let jsCode: string;
  try {
    const result = ts.transpileModule(code, {
      compilerOptions: {
        module: ts.ModuleKind.ES2015,
        target: ts.ScriptTarget.ES2015,
        removeComments: true,
      },
    });
    jsCode = result.outputText;
  } catch (e) {
    return {
      stdout: '',
      stderr: `TypeScript compilation error: ${e}`,
    };
  }

  // Run transpiled JS in an iframe to capture output
  const iframe = document.createElement('iframe');
  iframe.style.display = 'none';
  document.body.appendChild(iframe);

  const outputs: string[] = [];
  const errors: string[] = [];

  try {
    const win = iframe.contentWindow!;
    const doc = iframe.contentDocument!;

    // Override console methods to capture output
    const winAny = win as any;
    winAny.console.log = (...args: any[]) => {
      outputs.push(args.map((a) => String(a)).join(' '));
    };
    winAny.console.error = (...args: any[]) => {
      errors.push(args.map((a) => String(a)).join(' '));
    };
    winAny.console.warn = (...args: any[]) => {
      errors.push(args.map((a) => String(a)).join(' '));
    };
    winAny.console.info = (...args: any[]) => {
      outputs.push(args.map((a) => String(a)).join(' '));
    };

    // Inject and execute the transpiled code
    const script = doc.createElement('script');
    script.textContent = jsCode;
    doc.body.appendChild(script);
  } catch (e) {
    errors.push(String(e));
  } finally {
    // Small delay to allow async code to complete, then cleanup
    setTimeout(() => {
      if (iframe.parentNode) {
        document.body.removeChild(iframe);
      }
    }, 100);
  }

  return {
    stdout: outputs.join('\n'),
    stderr: errors.join('\n'),
  };
}
