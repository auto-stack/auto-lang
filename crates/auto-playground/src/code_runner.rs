use std::path::PathBuf;
use std::process::Command;

pub struct CodeRunResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub time_ms: u64,
}

fn temp_dir() -> PathBuf {
    std::env::temp_dir().join("auto-playground-run")
}

fn ensure_temp_dir() -> std::io::Result<PathBuf> {
    let dir = temp_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn cleanup_files(paths: &[PathBuf]) {
    for p in paths {
        let _ = std::fs::remove_file(p);
    }
}

/// Try to find an executable in PATH
fn find_exe(name: &str) -> Option<PathBuf> {
    let exe_name = if cfg!(windows) {
        format!("{}.exe", name)
    } else {
        name.to_string()
    };

    if let Ok(paths) = std::env::var("PATH") {
        for dir in std::env::split_paths(&paths) {
            let full = dir.join(&exe_name);
            if full.is_file() {
                return Some(full);
            }
        }
    }
    None
}

fn run_cmd(cmd: &mut Command) -> std::io::Result<CodeRunResult> {
    let start = std::time::Instant::now();
    let output = cmd.output()?;
    let time_ms = start.elapsed().as_millis() as u64;

    Ok(CodeRunResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        time_ms,
    })
}

pub fn run_python(code: &str) -> CodeRunResult {
    let dir = match ensure_temp_dir() {
        Ok(d) => d,
        Err(e) => {
            return CodeRunResult {
                stdout: String::new(),
                stderr: format!("Failed to create temp dir: {}", e),
                exit_code: -1,
                time_ms: 0,
            };
        }
    };

    let file = dir.join(format!("run_{}.py", std::process::id()));
    let paths = vec![file.clone()];

    if let Err(e) = std::fs::write(&file, code) {
        return CodeRunResult {
            stdout: String::new(),
            stderr: format!("Failed to write temp file: {}", e),
            exit_code: -1,
            time_ms: 0,
        };
    }

    // Try common Python commands on Windows and Unix
    let python_cmds = ["python", "python3", "py"];

    for cmd_name in &python_cmds {
        if find_exe(cmd_name).is_some() {
            let result = run_cmd(&mut Command::new(cmd_name).arg(&file));
            cleanup_files(&paths);
            match result {
                Ok(r) => return r,
                Err(e) => {
                    return CodeRunResult {
                        stdout: String::new(),
                        stderr: format!("{} failed: {}", cmd_name, e),
                        exit_code: -1,
                        time_ms: 0,
                    };
                }
            }
        }
    }

    cleanup_files(&paths);
    CodeRunResult {
        stdout: String::new(),
        stderr: "Python interpreter not found. Please install Python and ensure 'python' or 'python3' is in PATH.".to_string(),
        exit_code: -1,
        time_ms: 0,
    }
}

pub fn run_rust(code: &str) -> CodeRunResult {
    let dir = match ensure_temp_dir() {
        Ok(d) => d,
        Err(e) => {
            return CodeRunResult {
                stdout: String::new(),
                stderr: format!("Failed to create temp dir: {}", e),
                exit_code: -1,
                time_ms: 0,
            };
        }
    };

    let src = dir.join(format!("run_{}.rs", std::process::id()));
    let exe = dir.join(format!("run_{}{}", std::process::id(), std::env::consts::EXE_SUFFIX));
    let paths = vec![src.clone(), exe.clone()];

    // Strip the a2r stdlib import that requires the auto_lang crate
    let code = code.replace("#[allow(unused_imports)]\nuse auto_lang::a2r_std::*;\n\n", "");
    let code = code.replace("use auto_lang::a2r_std::*;\n\n", "");

    if let Err(e) = std::fs::write(&src, &code) {
        return CodeRunResult {
            stdout: String::new(),
            stderr: format!("Failed to write temp file: {}", e),
            exit_code: -1,
            time_ms: 0,
        };
    }

    if find_exe("rustc").is_none() {
        cleanup_files(&paths);
        return CodeRunResult {
            stdout: String::new(),
            stderr: "Rust compiler not found. Please install Rust and ensure 'rustc' is in PATH.".to_string(),
            exit_code: -1,
            time_ms: 0,
        };
    }

    // Compile
    let compile_start = std::time::Instant::now();
    let compile = Command::new("rustc")
        .arg(&src)
        .arg("-o")
        .arg(&exe)
        .output();

    match compile {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            cleanup_files(&paths);
            return CodeRunResult {
                stdout: String::new(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                exit_code: out.status.code().unwrap_or(-1),
                time_ms: compile_start.elapsed().as_millis() as u64,
            };
        }
        Err(e) => {
            cleanup_files(&paths);
            return CodeRunResult {
                stdout: String::new(),
                stderr: format!("Failed to run rustc: {}", e),
                exit_code: -1,
                time_ms: compile_start.elapsed().as_millis() as u64,
            };
        }
    }

    // Run
    let result = run_cmd(&mut Command::new(&exe));
    cleanup_files(&paths);

    match result {
        Ok(mut r) => {
            r.time_ms += compile_start.elapsed().as_millis() as u64;
            r
        }
        Err(e) => CodeRunResult {
            stdout: String::new(),
            stderr: format!("Failed to run executable: {}", e),
            exit_code: -1,
            time_ms: compile_start.elapsed().as_millis() as u64,
        },
    }
}

pub fn run_c(code: &str) -> CodeRunResult {
    let dir = match ensure_temp_dir() {
        Ok(d) => d,
        Err(e) => {
            return CodeRunResult {
                stdout: String::new(),
                stderr: format!("Failed to create temp dir: {}", e),
                exit_code: -1,
                time_ms: 0,
            };
        }
    };

    let src = dir.join(format!("run_{}.c", std::process::id()));
    let exe = dir.join(format!("run_{}{}", std::process::id(), std::env::consts::EXE_SUFFIX));
    let paths = vec![src.clone(), exe.clone()];

    if let Err(e) = std::fs::write(&src, code) {
        return CodeRunResult {
            stdout: String::new(),
            stderr: format!("Failed to write temp file: {}", e),
            exit_code: -1,
            time_ms: 0,
        };
    }

    // Try gcc first, then clang
    let compilers = [("gcc", "gcc"), ("clang", "clang")];
    let mut compiler_found = None;
    for (name, _) in &compilers {
        if find_exe(name).is_some() {
            compiler_found = Some(*name);
            break;
        }
    }

    let compiler = match compiler_found {
        Some(c) => c,
        None => {
            cleanup_files(&paths);
            return CodeRunResult {
                stdout: String::new(),
                stderr: "C compiler not found. Please install gcc or clang and ensure it is in PATH.".to_string(),
                exit_code: -1,
                time_ms: 0,
            };
        }
    };

    // Compile
    let compile_start = std::time::Instant::now();
    let compile = Command::new(compiler)
        .arg(&src)
        .arg("-o")
        .arg(&exe)
        .output();

    match compile {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            cleanup_files(&paths);
            return CodeRunResult {
                stdout: String::new(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                exit_code: out.status.code().unwrap_or(-1),
                time_ms: compile_start.elapsed().as_millis() as u64,
            };
        }
        Err(e) => {
            cleanup_files(&paths);
            return CodeRunResult {
                stdout: String::new(),
                stderr: format!("Failed to run {}: {}", compiler, e),
                exit_code: -1,
                time_ms: compile_start.elapsed().as_millis() as u64,
            };
        }
    }

    // Run
    let result = run_cmd(&mut Command::new(&exe));
    cleanup_files(&paths);

    match result {
        Ok(mut r) => {
            r.time_ms += compile_start.elapsed().as_millis() as u64;
            r
        }
        Err(e) => CodeRunResult {
            stdout: String::new(),
            stderr: format!("Failed to run executable: {}", e),
            exit_code: -1,
            time_ms: compile_start.elapsed().as_millis() as u64,
        },
    }
}

pub fn run_typescript(code: &str) -> CodeRunResult {
    let dir = match ensure_temp_dir() {
        Ok(d) => d,
        Err(e) => {
            return CodeRunResult {
                stdout: String::new(),
                stderr: format!("Failed to create temp dir: {}", e),
                exit_code: -1,
                time_ms: 0,
            };
        }
    };

    let ts_file = dir.join(format!("run_{}.ts", std::process::id()));
    let js_file = dir.join(format!("run_{}.js", std::process::id()));
    let paths = vec![ts_file.clone(), js_file.clone()];

    if let Err(e) = std::fs::write(&ts_file, code) {
        return CodeRunResult {
            stdout: String::new(),
            stderr: format!("Failed to write temp file: {}", e),
            exit_code: -1,
            time_ms: 0,
        };
    }

    let start = std::time::Instant::now();

    // Try tsx / ts-node first (direct TS execution)
    for cmd_name in ["tsx", "ts-node"] {
        if find_exe(cmd_name).is_some() {
            let result = run_cmd(&mut Command::new(cmd_name).arg(&ts_file));
            cleanup_files(&paths);
            match result {
                Ok(mut r) => {
                    r.time_ms = start.elapsed().as_millis() as u64;
                    return r;
                }
                Err(_) => continue,
            }
        }
    }

    // Fallback: tsc + node
    let node = find_exe("node");
    let tsc = find_exe("tsc");

    if node.is_none() {
        cleanup_files(&paths);
        return CodeRunResult {
            stdout: String::new(),
            stderr: "Node.js not found. Please install Node.js and ensure 'node' is in PATH, or install 'tsx' for direct TypeScript execution.".to_string(),
            exit_code: -1,
            time_ms: start.elapsed().as_millis() as u64,
        };
    }

    if let Some(tsc_path) = tsc {
        let compile = Command::new(tsc_path)
            .arg("--target")
            .arg("ES2020")
            .arg("--module")
            .arg("commonjs")
            .arg("--outDir")
            .arg(&dir)
            .arg(&ts_file)
            .output();

        match compile {
            Ok(out) if out.status.success() => {}
            Ok(out) => {
                cleanup_files(&paths);
                return CodeRunResult {
                    stdout: String::new(),
                    stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                    exit_code: out.status.code().unwrap_or(-1),
                    time_ms: start.elapsed().as_millis() as u64,
                };
            }
            Err(e) => {
                cleanup_files(&paths);
                return CodeRunResult {
                    stdout: String::new(),
                    stderr: format!("Failed to run tsc: {}", e),
                    exit_code: -1,
                    time_ms: start.elapsed().as_millis() as u64,
                };
            }
        }

        let result = run_cmd(&mut Command::new(node.unwrap()).arg(&js_file));
        cleanup_files(&paths);
        match result {
            Ok(mut r) => {
                r.time_ms = start.elapsed().as_millis() as u64;
                r
            }
            Err(e) => CodeRunResult {
                stdout: String::new(),
                stderr: format!("Failed to run node: {}", e),
                exit_code: -1,
                time_ms: start.elapsed().as_millis() as u64,
            },
        }
    } else {
        // No tsc - try running raw JS-like TS with node (may fail on type annotations)
        cleanup_files(&paths);
        CodeRunResult {
            stdout: String::new(),
            stderr: "TypeScript compiler (tsc) not found. Please install TypeScript globally ('npm install -g typescript') or install 'tsx' ('npm install -g tsx').".to_string(),
            exit_code: -1,
            time_ms: start.elapsed().as_millis() as u64,
        }
    }
}
