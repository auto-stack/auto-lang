// PLAN-998 frozen resource wrapper (probe copy of oracle/resource-wrapper-v1.rs)
// — drives the generated business entry under the observe profile, prints the
// typed result line, delivers typed resource failures as controlled exits,
// and supports the same-process fault→clear→retry leg (A2R_RETRY=1).
mod generated {
    include!("generated.rs");
    pub fn entry() -> i64 { main() }
}
use std::panic::{catch_unwind, AssertUnwindSafe};

fn main() {
    // Wrapper-startup allocations (env reads, hook box) must NOT enter the
    // heap ordinal space — read them BEFORE the reset below.
    let cancel_ms = std::env::var("A2R_CANCEL_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok());
    // Typed payloads carry their own diagnostic lines; the default reporter
    // is silenced for ALL panics (a foreign panic is still distinguished by
    // the marker line and exit 101 below).
    std::panic::set_hook(Box::new(|_| {}));
    generated::__a2r_reset();
    let canceller = cancel_ms.map(|ms| {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            generated::__a2r_cancel_set();
        })
    });
    let result = catch_unwind(AssertUnwindSafe(|| generated::entry()));
    match result {
        Ok(v) => {
            println!("auto.res:heap:{}", generated::__a2r_heap_n());
            println!("auto.res:result:{}", v);
        }
        Err(payload) => {
            let msg = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()));
            match msg.as_deref() {
                Some(s) if s.starts_with("auto.res:fail:") => {
                    if std::env::var("A2R_RETRY").map(|v| v == "1").unwrap_or(false) {
                        // Same-process retry: clear the fault knob, reset the
                        // counters, drive the entry again — the failed
                        // attempt's cleanup and this retry are both observed
                        // in one process.
                        println!("auto.res:retry");
                        generated::__a2r_fault_clear();
                        generated::__a2r_reset();
                        match catch_unwind(AssertUnwindSafe(|| generated::entry())) {
                            Ok(v) => {
                                println!("auto.res:heap:{}", generated::__a2r_heap_n());
                                println!("auto.res:result:{}", v)
                            }
                            Err(p2) => {
                                let m2 = p2
                                    .downcast_ref::<String>()
                                    .cloned()
                                    .or_else(|| p2.downcast_ref::<&str>().map(|s| s.to_string()));
                                eprintln!("auto.res:fail:retry-{}", m2.as_deref().unwrap_or("foreign"));
                                std::process::exit(101);
                            }
                        }
                    } else {
                        std::process::exit(71);
                    }
                }
                Some(s) if s.starts_with("auto.trap:") => {
                    std::process::exit(70);
                }
                _ => {
                    eprintln!("auto.res:fail:foreign");
                    std::process::exit(101);
                }
            }
        }
    }
    if let Some(t) = canceller {
        let _ = t.join();
    }
}
