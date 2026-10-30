use ace_av::console_host::{compile, ConsoleCapture};
use ace_av_core::{
    av::{CancelFlag, ExecLimits},
    contract::{ExecOutcome, Value},
};

fn grants() -> Vec<String> {
    vec!["console.stdout".into(), "console.stderr".into()]
}

#[test]
fn av_console_entry_selection_cannot_bypass_main_policy() {
    let error = compile(
        "fn helper() int { return 1 }\nfn main() int { return 2 }\n",
        "d_helper",
    )
    .err()
    .unwrap();
    assert_eq!(error.code, "lower.entry");
}

#[test]
fn av_console_owned_front_door_enforces_console_entry_only() {
    let src = "fn helper() int { print(99); return 11 }\nfn main() int { print(7); return 42 }";
    let c = ace_core::source::compile_host_console_source(src, "entry.at", &[]).unwrap();
    assert_eq!(
        ace_lowering::lower_checked_owned(&c.checked, "d_helper", 1)
            .unwrap_err()
            .code(),
        "lower.entry"
    );
    for src in [src, "print(7)", "fn main() void { print(7); 42 }"] {
        let c = ace_core::source::compile_host_console_source(src, "entry.at", &[]).unwrap();
        let m = ace_lowering::lower_checked_owned(&c.checked, "d_main", 1).unwrap();
        let loaded = ace_av_core::verify_abc_bytes(
            &ace_av_core::encode_abc(&m).unwrap().to_flat(),
            &m.owner,
        )
        .unwrap();
        let mut cap = ConsoleCapture::default();
        let (out, _) = ace_av_core::av::execute_console(
            &loaded,
            &ExecLimits::default(),
            &CancelFlag::new(false),
            &grants(),
            &mut cap,
        )
        .unwrap();
        assert!(matches!(out, ExecOutcome::Success { .. }));
        assert_eq!(cap.stdout, b"7\n");
    }
    // The inherited text/owned API still lets its callers choose a helper.
    let c = ace_core::source::compile_text_source(
        "fn helper() int { return 11 }\nfn main() int { return 42 }",
        "old.at",
        &[],
    )
    .unwrap();
    let m = ace_lowering::lower_checked_owned(&c.checked, "d_helper", 1).unwrap();
    let loaded =
        ace_av_core::verify_abc_bytes(&ace_av_core::encode_abc(&m).unwrap().to_flat(), &m.owner)
            .unwrap();
    assert!(matches!(
        ace_av_core::av::execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false)).0,
        ExecOutcome::Success {
            value: Value::I32(11),
            ..
        }
    ));
}

#[test]
fn av_console_retained_capture_resource_kinds_and_precedence() {
    use ace_av_core::{contract::ResourceKind, host::MAX_RUN_BYTES};
    let p = compile("print(1)", "d_main").unwrap();
    for (bytes, memory, expected) in [
        (3, 2, Some(ResourceKind::SlotBytes)),
        (
            MAX_RUN_BYTES + 1,
            MAX_RUN_BYTES * 2,
            Some(ResourceKind::ConsoleOutput),
        ),
        (MAX_RUN_BYTES + 1, 2, Some(ResourceKind::ConsoleOutput)),
        (3, ExecLimits::default().max_slot_bytes, None),
    ] {
        let mut cap = ConsoleCapture {
            stdout: vec![b'x'; bytes],
            ..Default::default()
        };
        let mut limits = ExecLimits::default();
        limits.max_slot_bytes = memory;
        let (out, trace) = p
            .run(&grants(), &limits, &CancelFlag::new(false), &mut cap)
            .unwrap();
        if let Some(kind) = expected {
            assert!(
                matches!(out, ExecOutcome::Resource { kind: actual, .. } if actual == kind),
                "{out:?}, expected {kind:?}"
            );
            assert_eq!(trace.root_frame_commits, 0);
            assert_eq!(cap.dispatches, 0);
            assert_eq!(cap.stdout, vec![b'x'; bytes]);
        } else {
            assert!(matches!(
                out,
                ExecOutcome::Success {
                    value: Value::Unit,
                    ..
                }
            ));
            assert_eq!(cap.stdout, b"xxx1\n");
            assert_eq!(cap.dispatches, 1);
        }
        assert!(cap.stderr.is_empty());
    }
}

#[test]
fn av_console_exact_source_bytes_and_result() {
    let program = compile("fn main() int {\n print(-2147483648)\n print(-7)\n print(0)\n print(true)\n eprint(false)\n print(\"你好\\0\\n\")\n eprint('界')\n return 7\n}\n", "d_main").unwrap();
    assert_eq!(&program.wire()[..8], b"AVHC\x01\x00\x08\x00");
    let mut cap = ConsoleCapture::default();
    let (out, trace) = program
        .run(
            &grants(),
            &ExecLimits::default(),
            &CancelFlag::new(false),
            &mut cap,
        )
        .unwrap();
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(7),
                ..
            }
        ),
        "{out:?}"
    );
    assert_eq!(
        cap.stdout,
        "-2147483648\n-7\n0\ntrue\n你好\0\n\n".as_bytes()
    );
    assert_eq!(cap.stderr, "false\n界\n".as_bytes());
    assert_eq!(cap.dispatches, 7);
    assert_eq!(trace.frames_remaining, 0);
}

#[test]
fn av_console_provider_cannot_reenter_guest_and_scope_clears() {
    use ace_av_core::host::{ConsoleHost, HostError, Stream};
    let old = ace_core::source::compile_scalar_source("fn main() int { return 99 }", "old.at", &[])
        .unwrap();
    let m = ace_lowering::lower_checked(&old.checked, "d_main", 0).unwrap();
    let loaded =
        ace_av_core::verify_abc_bytes(&ace_av_core::encode_abc(&m).unwrap().to_flat(), &m.owner)
            .unwrap();
    struct ReenterHost<'a> {
        target: &'a ace_av_core::LoadedModule,
        capture: ConsoleCapture,
        attempted: bool,
    }
    impl ConsoleHost for ReenterHost<'_> {
        fn retained_bytes(&self) -> usize {
            self.capture.stdout.len() + self.capture.stderr.len()
        }
        fn write(
            &mut self,
            stream: Stream,
            bytes: &[u8],
            cancel: &CancelFlag,
        ) -> Result<(), HostError> {
            self.attempted = true;
            let (out, trace) =
                ace_av_core::av::execute(self.target, &ExecLimits::default(), cancel);
            assert!(
                matches!(out, ExecOutcome::Infra(ref m) if m.starts_with("host.reentry:")),
                "{out:?}"
            );
            assert_eq!(trace.root_frame_commits, 0);
            self.capture.write(stream, bytes, cancel)
        }
    }
    let p = compile("print(7)\n", "d_main").unwrap();
    let mut provider = ReenterHost {
        target: &loaded,
        capture: ConsoleCapture::default(),
        attempted: false,
    };
    let (out, _) = p
        .run(
            &grants(),
            &ExecLimits::default(),
            &CancelFlag::new(false),
            &mut provider,
        )
        .unwrap();
    assert!(matches!(
        out,
        ExecOutcome::Success {
            value: Value::Unit,
            ..
        }
    ));
    assert!(provider.attempted);
    assert_eq!(provider.capture.stdout, b"7\n");
    let (out, _) =
        ace_av_core::av::execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(
        out,
        ExecOutcome::Success {
            value: Value::I32(99),
            ..
        }
    ));
}

#[test]
fn av_console_owned_borrowed_and_unit_script() {
    for src in [
        "fn main() void {\n let text Str = \"世界\"\n print(text)\n eprint(text)\n}\n",
        "print(\"世界\")\neprint(\"世界\")\n",
    ] {
        let program = compile(src, "d_main").unwrap();
        let mut cap = ConsoleCapture::default();
        let (out, trace) = program
            .run(
                &grants(),
                &ExecLimits::default(),
                &CancelFlag::new(false),
                &mut cap,
            )
            .unwrap();
        assert!(
            matches!(
                out,
                ExecOutcome::Success {
                    value: Value::Unit,
                    ..
                }
            ),
            "{out:?}"
        );
        assert_eq!(cap.stdout, "世界\n".as_bytes());
        assert_eq!(cap.stderr, "世界\n".as_bytes());
        assert_eq!(trace.frames_remaining, 0);
        assert_eq!(
            trace.arena.as_ref().unwrap().live(),
            0,
            "console borrows cannot keep guest owners alive"
        );
    }
}

#[test]
fn av_console_output_buffers_share_the_execution_memory_budget() {
    use ace_av_core::contract::ResourceKind;
    let p = compile("print(123456789)\n", "d_main").unwrap();
    let loaded = p.load().unwrap();
    let mut cap = ConsoleCapture::default();
    let limits = ExecLimits::default();
    let (_, baseline) = p
        .run(&grants(), &limits, &CancelFlag::new(false), &mut cap)
        .unwrap();
    assert!(
        baseline.peak_slot_bytes >= 20,
        "staging and capture bytes must be charged"
    );
    let mut limits = limits;
    limits.max_slot_bytes = baseline.peak_slot_bytes as usize - 1;
    let mut cap = ConsoleCapture::default();
    let (out, _) = ace_av_core::av::execute_console(
        &loaded,
        &limits,
        &CancelFlag::new(false),
        &grants(),
        &mut cap,
    )
    .unwrap();
    assert!(
        matches!(
            out,
            ExecOutcome::Resource {
                kind: ResourceKind::SlotBytes,
                ..
            }
        ),
        "{out:?}"
    );
    assert_eq!(cap.dispatches, 0);
    // A reused capture cannot escape the memory line at root-frame admission either.
    cap.stdout.resize(limits.max_slot_bytes, b'x');
    let (out, trace) = ace_av_core::av::execute_console(
        &loaded,
        &limits,
        &CancelFlag::new(false),
        &grants(),
        &mut cap,
    )
    .unwrap();
    assert!(matches!(
        out,
        ExecOutcome::Resource {
            kind: ResourceKind::SlotBytes,
            ..
        }
    ));
    assert_eq!(trace.root_frame_commits, 0);
}

#[test]
fn av_console_unit_returns_never_make_storable_values() {
    use ace_av_core::{
        mir::{MirSlot, MirType, SlotKind},
        mir_verify::verify_mir,
    };
    let p = compile("fn helper() void {}\nfn main() void { print(1) }", "d_main").unwrap();
    let mut m = p.load().unwrap().module().clone();
    let unit = m.intern_type(MirType::Unit);
    m.funcs[0].slots.push(MirSlot {
        name: Some("illegal_unit".into()),
        ty: unit,
        kind: SlotKind::Local,
    });
    assert_eq!(verify_mir(&m).unwrap_err().code(), "mir.unit-placement");
}

#[test]
fn av_console_grants_are_checked_before_guest_and_dispatch() {
    let program = compile(
        "fn main() int {\n print(1)\n eprint(2)\n return 3\n}\n",
        "d_main",
    )
    .unwrap();
    for grants in [
        vec![],
        vec!["console.stdout".into()],
        vec!["console.stderr".into()],
    ] {
        let mut cap = ConsoleCapture::default();
        let error = program
            .run(
                &grants,
                &ExecLimits::default(),
                &CancelFlag::new(false),
                &mut cap,
            )
            .unwrap_err();
        assert_eq!(error.code, "host.unauthorized");
        assert_eq!(cap.dispatches, 0);
        assert!(cap.stdout.is_empty() && cap.stderr.is_empty());
    }
    // The generic entry cannot bypass the explicit host front door.
    let loaded = program.load().unwrap();
    let (out, trace) =
        ace_av_core::av::execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(out, ExecOutcome::Infra(ref message) if message.starts_with("host.unauthorized"))
    );
    assert_eq!(trace.root_frame_commits, 0);
    // Even this already-loaded credential cannot retain a prior run's grant.
    let mut cap = ConsoleCapture::default();
    ace_av_core::av::execute_console(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &grants(),
        &mut cap,
    )
    .unwrap();
    let mut withdrawn = ConsoleCapture::default();
    let error = ace_av_core::av::execute_console(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &[],
        &mut withdrawn,
    )
    .unwrap_err();
    assert_eq!(error.code, "host.unauthorized");
    assert_eq!(withdrawn.dispatches, 0);
    assert!(withdrawn.stdout.is_empty() && withdrawn.stderr.is_empty());
}

#[test]
fn av_console_one_call_boundary_and_atomic_failure() {
    let line = "x".repeat(65_535);
    for (text, expected_ok) in [(line.clone(), true), (format!("{line}x"), false)] {
        let program = compile(
            &format!("fn main() int {{\n print(\"{text}\")\n return 0\n}}"),
            "d_main",
        )
        .unwrap();
        let mut cap = ConsoleCapture::default();
        let (out, _) = program
            .run(
                &grants(),
                &ExecLimits::default(),
                &CancelFlag::new(false),
                &mut cap,
            )
            .unwrap();
        if expected_ok {
            assert!(matches!(out, ExecOutcome::Success { .. }), "{out:?}");
            assert_eq!(cap.stdout.len(), 65_536);
            assert_eq!(cap.stdout.last(), Some(&b'\n'));
            assert_eq!(cap.dispatches, 1);
        } else {
            assert!(
                matches!(
                    out,
                    ExecOutcome::Resource {
                        kind: ace_av_core::contract::ResourceKind::ConsoleOutput,
                        ..
                    }
                ),
                "{out:?}"
            );
            assert!(cap.stdout.is_empty());
            assert_eq!(cap.dispatches, 0);
        }
    }
}

#[test]
fn av_console_total_cross_stream_boundary_and_prior_output_survives() {
    let line = "x".repeat(65_535);
    let lines = (0..16)
        .map(|i| {
            format!(
                "{}(\"{line}\")\n",
                if i % 2 == 0 { "print" } else { "eprint" }
            )
        })
        .collect::<String>();
    for extra in ["", "print(\"\")\n"] {
        let program = compile(
            &format!("fn main() int {{\n{lines}{extra}return 0\n}}"),
            "d_main",
        )
        .unwrap();
        let mut cap = ConsoleCapture::default();
        let mut limits = ExecLimits::default();
        limits.wall = std::time::Duration::from_secs(20);
        limits.max_steps = 10_000_000;
        let (out, _) = program
            .run(&grants(), &limits, &CancelFlag::new(false), &mut cap)
            .unwrap();
        if extra.is_empty() {
            assert!(matches!(out, ExecOutcome::Success { .. }), "{out:?}");
        } else {
            assert!(
                matches!(
                    out,
                    ExecOutcome::Resource {
                        kind: ace_av_core::contract::ResourceKind::ConsoleOutput,
                        ..
                    }
                ),
                "{out:?}"
            );
        }
        assert_eq!(cap.stdout.len(), 524_288);
        assert_eq!(cap.stderr.len(), 524_288);
        assert_eq!(cap.dispatches, 16);
    }
}

#[test]
fn av_console_cancellation_inside_host_and_io_failure_are_distinct() {
    use ace_av_core::host::{ConsoleHost, HostError, Stream};
    struct FailSecond {
        first: ConsoleCapture,
        cancel: bool,
    }
    impl ConsoleHost for FailSecond {
        fn retained_bytes(&self) -> usize {
            self.first.stdout.len() + self.first.stderr.len()
        }
        fn write(
            &mut self,
            stream: Stream,
            bytes: &[u8],
            flag: &CancelFlag,
        ) -> Result<(), HostError> {
            if self.first.dispatches == 0 {
                return self.first.write(stream, bytes, flag);
            }
            if self.cancel {
                flag.store(true, std::sync::atomic::Ordering::Relaxed);
                if ace_av_core::av::CancelCheck::poll(flag) {
                    return Err(HostError::Cancelled);
                }
                panic!("cancel must be visible before commit");
            }
            Err(HostError::Io("injected write failure".into()))
        }
    }
    let program = compile(
        "fn main() void {\n let text Str = \"1\"\n print(text)\n eprint(2)\n}",
        "d_main",
    )
    .unwrap();
    for cancel in [true, false] {
        let mut host = FailSecond {
            first: ConsoleCapture::default(),
            cancel,
        };
        let (out, trace) = program
            .run(
                &grants(),
                &ExecLimits::default(),
                &CancelFlag::new(false),
                &mut host,
            )
            .unwrap();
        if cancel {
            assert_eq!(out, ExecOutcome::Cancelled);
        } else {
            assert!(
                matches!(out, ExecOutcome::Infra(ref m) if m.starts_with("host.io:")),
                "{out:?}"
            );
        }
        assert_eq!(host.first.stdout, b"1\n");
        assert!(host.first.stderr.is_empty());
        assert_eq!(trace.frames_remaining, 0);
        assert_eq!(
            trace.arena.as_ref().unwrap().live(),
            0,
            "host failure must release guest owners"
        );
    }
}

#[test]
fn av_console_fuel_depth_memory_and_pre_cancel_guard_dispatch() {
    use ace_av_core::contract::ResourceKind;
    let program = compile("print(123456789)\n", "d_main").unwrap();
    let mut limits = ExecLimits::default();
    limits.max_frames = 1;
    let mut cap = ConsoleCapture::default();
    let (out, _) = program
        .run(&grants(), &limits, &CancelFlag::new(false), &mut cap)
        .unwrap();
    assert!(matches!(
        out,
        ExecOutcome::Resource {
            kind: ResourceKind::Frames,
            ..
        }
    ));
    assert_eq!(cap.dispatches, 0);
    limits = ExecLimits::default();
    limits.max_steps = 4;
    let (out, _) = program
        .run(&grants(), &limits, &CancelFlag::new(false), &mut cap)
        .unwrap();
    assert!(matches!(
        out,
        ExecOutcome::Resource {
            kind: ResourceKind::InstrSteps,
            ..
        }
    ));
    assert_eq!(cap.dispatches, 0);
    limits = ExecLimits::default();
    limits.max_slot_bytes = 12;
    let (out, _) = program
        .run(&grants(), &limits, &CancelFlag::new(false), &mut cap)
        .unwrap();
    assert!(matches!(
        out,
        ExecOutcome::Resource {
            kind: ResourceKind::SlotBytes,
            ..
        }
    ));
    assert_eq!(cap.dispatches, 0);
    let (out, _) = program
        .run(
            &grants(),
            &ExecLimits::default(),
            &CancelFlag::new(true),
            &mut cap,
        )
        .unwrap();
    assert_eq!(out, ExecOutcome::Cancelled);
    assert_eq!(cap.dispatches, 0);
}

#[test]
fn av_console_fresh_process_restore_and_grant_withdrawal() {
    use std::process::Command;
    let dir = std::env::temp_dir().join(format!(
        "av-console-863-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&dir).unwrap();
    let store = dir.join("store");
    let reference_path = dir.join("console-ref.json");
    let program = compile("print(\"跨进程\")\neprint(7)\n42\n", "d_main").unwrap();
    program.persist(&store, "plan863-test").unwrap();
    std::fs::write(
        &reference_path,
        serde_json::to_vec(&program.reference).unwrap(),
    )
    .unwrap();
    drop(program);
    for (mode, expected) in [("GRANTED", "ok"), ("WITHDRAWN", "host.unauthorized")] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_av_console_863_restore"))
            .arg(&store)
            .arg(&reference_path)
            .arg(mode)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let started = std::time::Instant::now();
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if started.elapsed() > std::time::Duration::from_secs(10) {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("console restore child exceeded 10s");
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(response["code"], expected);
        if mode == "GRANTED" {
            assert_eq!(response["stdout_b64"], "6Leo6L+b56iLCg==");
            assert_eq!(response["stderr_b64"], "Nwo=");
            assert_eq!(response["dispatches"], 2);
            assert!(response["outcome"].as_str().unwrap().contains("I32(42)"));
        } else {
            assert_eq!(response["stdout_b64"], "");
            assert_eq!(response["stderr_b64"], "");
            assert_eq!(response["dispatches"], 0);
        }
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn av_console_wire_declaration_mutations_reject_before_dispatch() {
    use ace_av_core::{
        abc_verify::{verify_abc_bytes, AbcAdmissionError},
        mir::{Effect, MirImport, MirType},
    };
    let p = compile("fn main() int {\n print(7)\n return 0\n}", "d_main").unwrap();
    let baseline = p.load().unwrap().module().clone();
    let attempts = [
        "unknown",
        "signature",
        "capability",
        "category",
        "duplicate",
        "guest-table",
        "guest-in-host",
        "required-cap",
        "budget",
        "reference",
        "operand",
    ];
    for mutation in attempts {
        let mut m = baseline.clone();
        let expected = match mutation {
            "unknown" => {
                m.host_imports[0].symbol = "auto.console.unknown.i32.v1".into();
                "host.unknown-symbol"
            }
            "signature" => {
                m.host_imports[0].symbol = "auto.console.print.bool.v1".into();
                "host.signature"
            }
            "capability" => {
                m.host_imports[0].capability = "console.stderr".into();
                "host.signature"
            }
            "category" => {
                m.host_imports[0].category = "guest/1".into();
                "host.signature"
            }
            "duplicate" => {
                m.host_imports.push(m.host_imports[0].clone());
                "host.duplicate"
            }
            "guest-table" => {
                m.imports.push(MirImport {
                    symbol: m.host_imports[0].symbol.clone(),
                    sig: m.host_imports[0].sig,
                    effect: Effect::Observable,
                    env: None,
                });
                "host.category"
            }
            "guest-in-host" => {
                m.host_imports[0].symbol = "fn:d_main".into();
                "host.unknown-symbol"
            }
            "required-cap" => {
                m.capabilities.clear();
                "host.unauthorized"
            }
            "budget" => {
                m.host_imports[0].max_call_bytes += 1;
                "host.signature"
            }
            "reference" => {
                for i in &mut m.funcs[0].blocks[0].instrs {
                    if let ace_av_core::mir::MirInstr::HostCall { host, .. } = i {
                        *host = 42;
                    }
                }
                "host.reference"
            }
            "operand" => {
                let bool_ty = m.intern_type(MirType::Bool);
                // Keep literal and slot consistent so the host type check owns the failure.
                m.consts[0] = ace_av_core::mir::MirConst::Bool(true);
                m.funcs[0].slots[0].ty = bool_ty;
                "host.signature"
            }
            _ => unreachable!(),
        };
        let bytes = ace_av_core::encode_abc(&m).unwrap().to_flat();
        let mut cap = ConsoleCapture::default();
        let result = verify_abc_bytes(&bytes, &m.owner);
        let code = match result {
            Err(AbcAdmissionError::Verify(e)) => e.code(),
            Err(e) => panic!("wrong admission category for {mutation}: {e:?}"),
            Ok(loaded) => {
                let run = ace_av_core::av::execute_console(
                    &loaded,
                    &ExecLimits::default(),
                    &CancelFlag::new(false),
                    &grants(),
                    &mut cap,
                );
                panic!(
                    "mutation {mutation} admitted and ran: {run:?}, dispatches {}",
                    cap.dispatches
                );
            }
        };
        assert_eq!(code, expected, "mutation {mutation}");
        assert_eq!(cap.dispatches, 0);
        assert!(cap.stdout.is_empty() && cap.stderr.is_empty());
    }
}

#[test]
fn av_console_unreachable_host_declarations_still_need_authority() {
    let program = compile(
        "fn unused() int {\n eprint(7)\n return 0\n}\nfn main() int { return 0 }",
        "d_main",
    )
    .unwrap();
    let mut cap = ConsoleCapture::default();
    let error = program
        .run(
            &[],
            &ExecLimits::default(),
            &CancelFlag::new(false),
            &mut cap,
        )
        .unwrap_err();
    assert_eq!(error.code, "host.unauthorized");
    assert_eq!(cap.dispatches, 0);
    let (out, _) = program
        .run(
            &grants(),
            &ExecLimits::default(),
            &CancelFlag::new(false),
            &mut cap,
        )
        .unwrap();
    assert!(matches!(
        out,
        ExecOutcome::Success {
            value: Value::I32(0),
            ..
        }
    ));
    assert_eq!(cap.dispatches, 0);
}

#[test]
fn av_console_public_decoded_identity_is_not_a_cross_profile_bypass() {
    use ace_av_core::{
        abc::{decode_abc, DecodedAbc},
        abc_verify::{verify_abc, AbcAdmissionError},
    };
    let p = compile("print(7)\n", "d_main").unwrap();
    let mut decoded = decode_abc(p.wire()).unwrap();
    let owner = p.load().unwrap().module().owner.clone();
    decoded.profile = ace_av_core::contract::ABC_PROFILE_ENTRY_UNIT;
    assert!(matches!(
        verify_abc(decoded, &owner),
        Err(AbcAdmissionError::Identity(_))
    ));
    let old =
        ace_core::source::compile_entry_unit_source("fn main() int { return 0 }", "old.at", &[])
            .unwrap();
    let m = ace_lowering::lower_checked(&old.checked, "d_main", 0).unwrap();
    let d = DecodedAbc {
        version: 1,
        profile: ace_av_core::contract::ABC_PROFILE_HOST_CONSOLE,
        module: m,
    };
    assert!(matches!(
        verify_abc(d, &owner),
        Err(AbcAdmissionError::Identity(_))
    ));
}

#[test]
fn av_console_artifact_identity_does_not_serialize_runtime_authority() {
    let source = "print(\"ok\")\n";
    let p = compile(source, "d_main").unwrap();
    let identical = compile(source, "d_main").unwrap();
    assert_eq!(p.wire(), identical.wire());
    let before = p.wire().to_vec();
    let mut cap = ConsoleCapture::default();
    p.run(
        &grants(),
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut cap,
    )
    .unwrap();
    assert_eq!(p.wire(), before);
    let mut wrong = p.reference.clone();
    wrong.requirements_fp = "wrong-required-capabilities".into();
    let root = std::env::temp_dir().join(format!(
        "av-console-863-identity-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    p.persist(&root, "plan863-identity").unwrap();
    assert!(ace_av::console_host::restore(&root, &wrong).is_err());
    std::fs::remove_dir_all(&root).unwrap();
    let mut invalid = p;
    invalid.reference.owner = "x".repeat(65_536);
    assert_eq!(invalid.load().unwrap_err().code, "host.identity");
}

#[test]
fn av_console_portable_section_is_required_and_legacy_families_reject_it() {
    let p = compile("print(7)\n", "d_main").unwrap();
    let original = p.wire().to_vec();
    for mutation in ["missing-section", "legacy-magic", "legacy-profile"] {
        let mut bytes = original.clone();
        match mutation {
            "missing-section" => {
                let at = bytes.windows(4).position(|x| x == b"HOST").unwrap();
                bytes[at..at + 4].copy_from_slice(b"NOPE");
            }
            "legacy-magic" => bytes[..4].copy_from_slice(b"AVAU"),
            "legacy-profile" => bytes[6..8].copy_from_slice(&7u16.to_le_bytes()),
            _ => unreachable!(),
        }
        let len = bytes.len() - 8;
        let hash = ace_av_core::contract::fnv1a64(&bytes[..len]);
        bytes[len..].copy_from_slice(&hash.to_le_bytes());
        assert!(
            ace_av_core::decode_abc(&bytes).is_err(),
            "{mutation} must reject structurally"
        );
    }
}

#[test]
fn av_console_guest_order_calls_text_projection_and_prior_output_on_trap() {
    let p = compile("fn helper() int {\n eprint(8)\n return 9\n}\nfn main() int {\n print(helper())\n print(\"a世界z\".slice(1, 3))\n return 1 / 0\n}", "d_main").unwrap();
    let mut cap = ConsoleCapture::default();
    let (out, _) = p
        .run(
            &grants(),
            &ExecLimits::default(),
            &CancelFlag::new(false),
            &mut cap,
        )
        .unwrap();
    assert!(matches!(out, ExecOutcome::Trap { .. }), "{out:?}");
    assert_eq!(cap.stdout, "9\n世界\n".as_bytes());
    assert_eq!(cap.stderr, b"8\n");
    assert_eq!(cap.dispatches, 3);
}
