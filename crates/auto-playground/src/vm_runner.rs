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
