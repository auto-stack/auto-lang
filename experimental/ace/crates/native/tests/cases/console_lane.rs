//! PLAN-947: the console/1 native lane — real machine-code console
//! dispatch with per-run authorization and the original budgets.

use ace_native::console::{
    console_authorize, console_compile, console_native_run, console_reject_exit,
};

fn caps() -> Vec<String> {
    vec!["console.stdout".into(), "console.stderr".into()]
}

#[test]
fn native_console_exact_utf8_streams_and_exit() {
    let mir = console_compile(
        "fn main() int {\n print(\"你好\\n\")\n eprint(\"é\")\n return 0\n}\n",
        "d_main",
    )
    .expect("console compile");
    console_authorize(&mir, &caps()).expect("authorized");
    let dir = std::env::temp_dir().join(format!(
        "mn-console-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let run = console_native_run(&mir, "d_main", &dir).expect("console run");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(run.exit_code, Some(0), "{:?}", run.exit_code);
    assert_eq!(run.stdout, "你好\n\n".as_bytes());
    assert_eq!(run.stderr, "é\n".as_bytes());
}

#[test]
fn native_console_i32_bool_char_say_lanes() {
    let mir = console_compile(
        "fn main() int {\n print(-2147483648)\n print(-7)\n print(0)\n print(true)\n eprint(false)\n print('界')\n eprint('😀')\n say(\"A中😀\")\n return 7\n}\n",
        "d_main",
    )
    .expect("console compile");
    console_authorize(&mir, &caps()).expect("authorized");
    let dir = std::env::temp_dir().join(format!(
        "mn-console-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let run = console_native_run(&mir, "d_main", &dir).expect("console run");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(run.exit_code, Some(7));
    assert_eq!(
        run.stdout,
        "-2147483648\n-7\n0\ntrue\n界\nA中😀\n".as_bytes()
    );
    assert_eq!(run.stderr, "false\n😀\n".as_bytes());
}

#[test]
fn native_console_unauthorized_run_rejects_before_dispatch() {
    let mir =
        console_compile("fn main() int { print(1); return 0 }", "d_main").expect("console compile");
    // Structural validation passes; the per-run authorization is what
    // rejects — before any emission, link or execution (dispatch count 0).
    assert!(matches!(
        console_authorize(&mir, &[]),
        Err(e) if e.code == "host.unauthorized"
    ));
    console_authorize(&mir, &caps()).expect("authorized");
    // Re-authorize per run: the same module runs cleanly after granting.
    let dir = std::env::temp_dir().join(format!(
        "mn-console-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let run = console_native_run(&mir, "d_main", &dir).expect("console run");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(run.exit_code, Some(0));
    assert_eq!(run.stdout, "1\n".as_bytes());
}

#[test]
fn native_console_per_call_budget_rejects_atomically() {
    // 65535 payload bytes + 1 LF = 65536 (the legal per-call boundary).
    let legal = "A".repeat(65535);
    let mir = console_compile(
        &format!("fn main() int {{ print(\"{legal}\"); return 0 }}"),
        "d_main",
    )
    .expect("console compile");
    console_authorize(&mir, &caps()).expect("authorized");
    let dir = std::env::temp_dir().join(format!(
        "mn-console-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let run = console_native_run(&mir, "d_main", &dir).expect("console run");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(run.exit_code, Some(0));
    assert_eq!(run.stdout.len(), 65536);

    // 65536 payload bytes + LF exceeds MAX_CALL_BYTES: rejected before
    // dispatch, exit = console reject code, no output.
    let over = "A".repeat(65536);
    let mir = console_compile(
        &format!("fn main() int {{ print(\"{over}\"); return 0 }}"),
        "d_main",
    )
    .expect("console compile");
    console_authorize(&mir, &caps()).expect("authorized");
    let dir = std::env::temp_dir().join(format!(
        "mn-console-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let run = console_native_run(&mir, "d_main", &dir).expect("console run");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(run.exit_code, Some(console_reject_exit()));
    assert!(run.stdout.is_empty(), "no partial output");
}

#[test]
fn native_console_cumulative_run_budget_keeps_prior_output() {
    // 17 × 65536 = 1114112 > MAX_RUN_BYTES (1048576): the 17th call
    // rejects; the first 16 calls (1048576 bytes exactly) stay committed.
    let line = "A".repeat(65535);
    let mut src = String::from("fn main() int {\n");
    for _ in 0..17 {
        src.push_str(&format!("print(\"{line}\")\n"));
    }
    src.push_str("return 0\n}\n");
    let mir = console_compile(&src, "d_main").expect("console compile");
    console_authorize(&mir, &caps()).expect("authorized");
    let dir = std::env::temp_dir().join(format!(
        "mn-console-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let run = console_native_run(&mir, "d_main", &dir).expect("console run");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(run.exit_code, Some(console_reject_exit()));
    assert_eq!(
        run.stdout.len(),
        1048576,
        "prior whole calls stay committed"
    );
}

#[test]
fn native_console_helper_functions_and_guest_order() {
    let mir = console_compile(
        "fn helper() int { eprint(8); return 2 }\nfn main() int {\n print(9)\n print(helper())\n return 0\n}\n",
        "d_main",
    )
    .expect("console compile");
    console_authorize(&mir, &caps()).expect("authorized");
    let dir = std::env::temp_dir().join(format!(
        "mn-console-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let run = console_native_run(&mir, "d_main", &dir).expect("console run");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(run.exit_code, Some(0));
    assert_eq!(run.stdout, "9\n2\n".as_bytes());
    assert_eq!(run.stderr, "8\n".as_bytes());
}

#[test]
fn native_console_unit_entry_is_a_named_capability_boundary() {
    // The mn unit ABI is the frozen zero-param i32 entry; a Unit-result
    // entry cannot become a console unit (named rejection, not a fake
    // pass). The AV lane executes the Unit channel (corpus O07/O09).
    let mir = match console_compile(
        "fn main() void {
 print(7)
}
",
        "d_main",
    ) {
        Ok(m) => m,
        Err(d) => {
            assert_eq!(d.code, "source.entry-tail");
            return;
        }
    };
    console_authorize(&mir, &caps()).expect("authorized");
    let dir = std::env::temp_dir().join("mn-console-unit-boundary");
    let _ = std::fs::remove_dir_all(&dir);
    match console_native_run(&mir, "d_main", &dir) {
        Err(ace_native::MnError::Unsupported("result", _)) => {}
        other => panic!("expected the named Unit-result boundary, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn native_console_trap_keeps_prior_output() {
    let mir = console_compile(
        "fn main() int {\n print(\"before\")\n eprint(\"fail\")\n return 1 / 0\n}\n",
        "d_main",
    )
    .expect("console compile");
    console_authorize(&mir, &caps()).expect("authorized");
    let dir = std::env::temp_dir().join(format!(
        "mn-console-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let run = console_native_run(&mir, "d_main", &dir).expect("console run");
    let _ = std::fs::remove_dir_all(&dir);
    // The frozen mn trap exit (70) with prior whole calls preserved.
    assert_eq!(run.exit_code, Some(70));
    assert_eq!(run.stdout, "before\n".as_bytes());
    assert_eq!(run.stderr, "fail\n".as_bytes());
}

#[test]
fn native_ordinary_lane_still_rejects_host_console() {
    // PLAN-863 compat: the ordinary single-function admission keeps
    // rejecting the host-console profile and its host table.
    let program = ace_av::console_host::compile("print(7)\n", "d_main").unwrap();
    let m = program.load().unwrap().module().clone();
    assert!(matches!(
        ace_native::emit::admit_unit(&m),
        Err(ace_native::MnError::Unsupported("profile", _))
    ));
    let units = ace_av::cache::split_units(&m).unwrap();
    for u in &units {
        assert!(matches!(
            ace_native::emit::admit_unit(u),
            Err(ace_native::MnError::Unsupported("profile", _))
        ));
    }
    // And the console door rejects non-console profiles.
    let scalar =
        ace_core::source::compile_scalar_source("fn main() int { return 1 }", "s.at", &[]).unwrap();
    let sm = ace_lowering::lower_checked(&scalar.checked, "d_main", 0).unwrap();
    let sunits = ace_av::cache::split_units(&sm).unwrap();
    assert!(matches!(
        ace_native::emit::emit_console_unit_coff(&sunits[0]),
        Err(ace_native::MnError::Unsupported("profile", _))
    ));
}
