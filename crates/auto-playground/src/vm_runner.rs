use std::time::Instant;

pub struct RunResult {
    pub stdout: String,
    pub result: String,
    pub time_ms: u64,
}

pub fn run_source(source: &str) -> RunResult {
    let start = Instant::now();

    let (result, stdout) = match auto_lang::run_with_capture(source) {
        Ok((res, out)) => (res, out),
        Err(e) => (
            String::new(),
            format!("Error: {}", e),
        ),
    };

    let time_ms = start.elapsed().as_millis() as u64;

    RunResult {
        stdout,
        result,
        time_ms,
    }
}

pub fn run_abt(abt: &str) -> RunResult {
    let start = Instant::now();

    let runtime = auto_lang::get_global_runtime();
    let stdout = match runtime.block_on(auto_lang::run_abt(abt)) {
        Ok(out) => out,
        Err(e) => format!("Error: {}", e),
    };

    let time_ms = start.elapsed().as_millis() as u64;

    RunResult {
        stdout,
        result: String::new(),
        time_ms,
    }
}
