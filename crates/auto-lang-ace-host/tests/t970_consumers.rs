#![cfg(windows)]

use serde_json::Value;

use auto_lang_ace_host::session_bridge::{
    install_session_host, install_session_host_with_store, SessionHostGuard,
};
use auto_lang_core::{AceSessionLimits, AceSessionOp, AceSessionOutcome};

/// One host at a time (the registry is single-owner and the tests run on
/// parallel threads); the guard's stop() runs BEFORE the lock releases
/// (field order is load-bearing).
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct Host {
    _guard: SessionHostGuard,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl Host {
    fn install() -> Host {
        let lock: std::sync::MutexGuard<'static, ()> =
            unsafe { std::mem::transmute(LOCK.lock().unwrap_or_else(|p| p.into_inner())) };
        let guard = install_session_host(AceSessionLimits::default()).expect("install");
        Host {
            _guard: guard,
            _lock: lock,
        }
    }
}

fn ok_of(r: &auto_lang_core::AceSessionReport) {
    match &r.outcome {
        AceSessionOutcome::Ok => {}
        other => panic!("expected Ok, got {other:?}"),
    }
}

/// The daemon protocol over a REAL named pipe: versioned responses,
/// create/eval/edit roundtrips, idempotent replay, and the mixed
/// async/edit discipline.
#[test]
fn t970_daemon_pipe_roundtrip() {
    let _host = Host::install();
    let pipe_name = format!("ace-sess-970-{}", std::process::id());
    std::thread::spawn(move || {
        auto_lang_core::AceDaemon::new().run_pipe(&pipe_name);
    });
    // Give the listener a moment (the client retries anyway).
    std::thread::sleep(std::time::Duration::from_millis(300));

    let mut client =
        auto_lang_core::AceSessionClient::connect(&format!("ace-sess-970-{}", std::process::id()))
            .expect("connect");

    // Versioned responses.
    let pong = client.ping().expect("ping");
    assert_eq!(pong["status"], "ok");
    assert_eq!(pong["version"], 1);

    // Create + eval through the wire.
    let created = client.new_session(None).expect("new-session");
    assert_eq!(created["status"], "ok", "{created}");
    let session = created["data"]["session"]
        .as_str()
        .expect("session id")
        .to_string();
    let eval = client.eval(&session, "2 + 3 * 4").expect("eval");
    assert_eq!(eval["status"], "ok", "{eval}");
    assert_eq!(eval["value"], "14");
    assert_eq!(eval["type"], "int");

    // Same-PID edit through the wire, with the edit base taken FROM THE
    // WIRE (F-970-R-02: inspect carries the source text in `value`).
    let mut wire_req = serde_json::json!({"method": "inspect", "session": session});
    wire_req["id"] = serde_json::json!(900);
    let inspected = client.request(wire_req).expect("wire inspect");
    assert_eq!(inspected["status"], "ok", "{inspected}");
    let source = inspected["value"]
        .as_str()
        .expect("wire source")
        .to_string();
    let at = source.find("return (2 + 3 * 4)").expect("lane body");
    // Replace the `4` inside the lane body.
    let four = at + "return (2 + 3 * ".len();
    let edited = client
        .edit(&session, four as u64, (four + 1) as u64, "5")
        .expect("edit");
    assert_eq!(edited["status"], "ok", "{edited}");
    let after = client.eval(&session, "2 + 3 * 5").expect("eval after edit");
    assert_eq!(after["value"], "17", "{after}");

    // F-970-R-01: an async chain, cancelled, then a SECOND async on the
    // same session must arm (never panic the owner thread).
    let mut async_req = serde_json::json!({
        "method": "eval-async", "session": session,
        "code": "var i = 0\nfor i < 10000 { i = i + 1 }\nreturn i"
    });
    async_req["id"] = serde_json::json!(901);
    let started = client.request(async_req.clone()).expect("first async");
    assert_eq!(started["status"], "ok", "{started}");
    let eval_id = started["data"]["eval_id"].as_u64().expect("eval id");
    let mut cancel_req =
        serde_json::json!({"method": "cancel", "session": session, "eval_id": eval_id});
    cancel_req["id"] = serde_json::json!(902);
    let cancelled = client.request(cancel_req).expect("cancel");
    assert_eq!(cancelled["status"], "cancelled", "{cancelled}");
    async_req["id"] = serde_json::json!(903);
    let second = client.request(async_req).expect("second async arms");
    assert_eq!(second["status"], "ok", "{second}");
    let second_id = second["data"]["eval_id"].as_u64().expect("second id");
    // Drive the second chain to completion over the wire.
    let mut final_value = None;
    for n in 904..1104 {
        let mut poll =
            serde_json::json!({"method": "result", "session": session, "eval_id": second_id});
        poll["id"] = serde_json::json!(n);
        let r = client.request(poll).expect("poll");
        if r["status"] != "ok" || r["data"]["snapshot_len"].is_null() {
            final_value = Some(r["value"].clone());
            break;
        }
    }
    assert_eq!(
        final_value,
        Some(serde_json::json!("10000")),
        "second chain runs to its oracle"
    );
}

/// The MCP tool surface over the real bridge: initialize negotiation,
/// create/evaluate/vars/patch with independent value oracles.
#[test]
fn t970_mcp_tools_over_real_host() {
    use auto_lang_core::mcp::ace_server::AceMcpServer;
    use auto_lang_core::mcp::protocol::{Id, JsonRpcRequest};

    let _host = Host::install();
    let mut server = AceMcpServer::new();

    let resp = server
        .handle(&JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(Id::Number(1)),
            method: "initialize".into(),
            params: Some(serde_json::json!({"protocolVersion": "2025-01-01"})),
        })
        .expect("response");
    assert_eq!(
        resp.result.unwrap()["protocolVersion"],
        auto_lang_core::mcp::ace_server::ACE_MCP_PROTOCOL_VERSION
    );

    let call = |server: &mut AceMcpServer, id: i64, name: &str, args: serde_json::Value| {
        let resp = server
            .handle(&JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: Some(Id::Number(id)),
                method: "tools/call".into(),
                params: Some(serde_json::json!({"name": name, "arguments": args})),
            })
            .expect("response");
        serde_json::to_value(&resp).unwrap()
    };

    let created = call(&mut server, 2, "auto_session_create", serde_json::json!({}));
    let text = created.to_string();
    assert!(text.contains("session"), "{text}");
    // The session id comes from the live registry (the tool text carries
    // it, but the registry is the authoritative id surface).
    let session_id = auto_lang_core::ace_session_list()
        .into_iter()
        .next()
        .expect("one live session");
    assert!(session_id.starts_with("ses_"), "{session_id}");

    // Independent arithmetic oracle through auto_evaluate.
    let ev = call(
        &mut server,
        3,
        "auto_evaluate",
        serde_json::json!({"session": session_id, "code": "6 * 7"}),
    );
    assert!(ev.to_string().contains("42"), "{ev}");

    // Carry discipline through the tool surface.
    let _ = call(
        &mut server,
        4,
        "auto_evaluate",
        serde_json::json!({"session": session_id, "code": "var x = 41"}),
    );
    let vars = call(
        &mut server,
        5,
        "auto_vars",
        serde_json::json!({"session": session_id}),
    );
    let vars_text = vars.to_string();
    assert!(vars_text.contains("x = 41"), "{vars_text}");

    // auto_patch: single-occurrence text replace as a same-PID edit on
    // the LIVE source — item bodies persist there across evaluations.
    let defined = call(
        &mut server,
        6,
        "auto_evaluate",
        serde_json::json!({"session": session_id, "code": "fn helper() int { return 7 }"}),
    );
    assert!(defined.to_string().contains("revision"), "{defined}");
    let patched = call(
        &mut server,
        7,
        "auto_patch",
        serde_json::json!({"session": session_id, "from": "return 7", "to": "return 9"}),
    );
    assert!(patched.to_string().contains("revision"), "{patched}");
    let ev2 = call(
        &mut server,
        8,
        "auto_evaluate",
        serde_json::json!({"session": session_id, "code": "helper() + 1"}),
    );
    assert!(ev2.to_string().contains("10"), "{ev2}");
}

/// The T-08 persistence face: records flush on shutdown, restore by
/// op-log replay (revision continuity, variables re-established),
/// legacy `.at` seeding, and the named bad-schema skip with the file
/// untouched.
#[test]
fn t970_persistence_records() {
    let _lock: std::sync::MutexGuard<'static, ()> =
        unsafe { std::mem::transmute(LOCK.lock().unwrap_or_else(|p| p.into_inner())) };
    let dir = std::env::temp_dir().join(format!("ace-sess-970-persist-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Phase 1: a live session with state, then a clean shutdown flush.
    {
        let _host =
            install_session_host_with_store(AceSessionLimits::default(), dir.clone()).unwrap();
        let created = auto_lang_core::ace_session_create(None, None);
        ok_of(&created);
        let s = created.session.clone().unwrap();
        let _ = auto_lang_core::ace_session_op(
            &s,
            AceSessionOp::Eval {
                input: "var x = 41",
                wall: None,
                cancel: None,
            },
        );
        let _ = auto_lang_core::ace_session_op(
            &s,
            AceSessionOp::Eval {
                input: "x = x + 1",
                wall: None,
                cancel: None,
            },
        );
        // Record the id for the restore assertion.
        std::fs::write(dir.join(".test-session-id"), &s).unwrap();
    } // guard drop → flush

    let sid = std::fs::read_to_string(dir.join(".test-session-id")).unwrap();
    let record_path = dir.join(format!("{sid}.state.json"));
    let record_text = std::fs::read_to_string(&record_path).expect("record flushed");
    assert!(record_text.contains("ac.session.state/1"), "{record_text}");

    // A legacy `.at` snapshot and a bad-schema record.
    std::fs::write(
        dir.join("legacy-session.at"),
        "fn main() int { return 2 + 3 * 4 }",
    )
    .unwrap();
    let bad = dir.join("future.state.json");
    std::fs::write(
        &bad,
        r#"{"schema":"ac.session.state/999","id":"ses_future","ops":[],"revision":0,"saved_at":1}"#,
    )
    .unwrap();
    let bad_before = std::fs::read_to_string(&bad).unwrap();

    // Phase 2: reinstall with the store — restore by op-log replay.
    {
        let _host =
            install_session_host_with_store(AceSessionLimits::default(), dir.clone()).unwrap();
        // The persisted session is back with its variables (re-verified).
        let vars = auto_lang_core::ace_session_op(&sid, AceSessionOp::Vars);
        ok_of(&vars);
        assert!(
            vars.vars
                .iter()
                .any(|v| v.name == "x" && v.value == auto_lang_core::AceSessionValue::Int(42)),
            "carried variable re-established: {:?}",
            vars.vars
        );
        let ev = auto_lang_core::ace_session_op(
            &sid,
            AceSessionOp::Eval {
                input: "x + 1",
                wall: None,
                cancel: None,
            },
        );
        ok_of(&ev);
        assert_eq!(ev.value, auto_lang_core::AceSessionValue::Int(43));

        // The legacy source replay session evaluates its frozen oracle.
        let legacy = auto_lang_core::ace_session_op(
            "legacy-session",
            AceSessionOp::Eval {
                input: "2 + 3 * 4",
                wall: None,
                cancel: None,
            },
        );
        ok_of(&legacy);
        assert_eq!(legacy.value, auto_lang_core::AceSessionValue::Int(14));

        // The bad-schema record was skipped and left untouched.
        let bad_after = std::fs::read_to_string(&bad).unwrap();
        assert_eq!(bad_before, bad_after, "unknown-schema record untouched");
        let gone = auto_lang_core::ace_session_op("ses_future", AceSessionOp::Inspect);
        match gone.outcome {
            AceSessionOutcome::Denied { code, .. } if code == "session/unknown" => {}
            other => panic!("bad-schema session must be skipped, got {other:?}"),
        }

        // F-970-R-03: a NEW session created after restore gets a FRESH
        // id (never the restored one) and never replaces it.
        let fresh = auto_lang_core::ace_session_create(None, None);
        ok_of(&fresh);
        let fresh_id = fresh.session.clone().unwrap();
        assert_ne!(fresh_id, sid, "restored id must not be regenerated");
        let still_there = auto_lang_core::ace_session_op(&sid, AceSessionOp::Vars);
        ok_of(&still_there);
        assert!(still_there.vars.iter().any(|v| v.name == "x"));

        // F-970-R-07: the FIRST edit after restore is a fast-path edit —
        // it advances the revision by exactly one and reports no rebuild
        // (a whole-source rebuild would reshape and set rebuilt).
        let before = auto_lang_core::ace_session_op(&sid, AceSessionOp::Inspect);
        ok_of(&before);
        let rev_before = before.revision;
        let raw = before.snapshot.clone().unwrap();
        // Edit the LANE BODY (never the last-region anchor): the prelude
        // literal of the carried variable.
        let at = raw
            .find("var x = 42")
            .expect("prelude literal in the lane body");
        let fast = auto_lang_core::ace_session_op(
            &sid,
            AceSessionOp::Edit {
                start: (at + 8) as u64,
                end: (at + 10) as u64,
                insert: b"43".as_slice(),
            },
        );
        ok_of(&fast);
        assert!(!fast.rebuilt, "post-restore first edit is a fast-path edit");
        assert_eq!(fast.revision, rev_before + 1, "exactly one revision step");
        // The session remains fully functional after the fast-path edit
        // (the carry table drives later evals; the edited literal was the
        // lane body's prelude, rewritten by the next evaluation).
        let ev = auto_lang_core::ace_session_op(
            &sid,
            AceSessionOp::Eval {
                input: "x + 1",
                wall: None,
                cancel: None,
            },
        );
        ok_of(&ev);
    }

    // Close removes the record (a clean end).
    {
        let _host =
            install_session_host_with_store(AceSessionLimits::default(), dir.clone()).unwrap();
        let closed = auto_lang_core::ace_session_op(&sid, AceSessionOp::Close);
        ok_of(&closed);
        assert!(!record_path.exists(), "close removes the record");
        let _ = Value::Null;
    }
    let _ = std::fs::remove_dir_all(&dir);
}
