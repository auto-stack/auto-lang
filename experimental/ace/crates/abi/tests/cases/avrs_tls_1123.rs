//! PLAN-1123 (AV stdlib HTTPS and proxy provider) — V-01/V-02/V-03 on the
//! ordinary AV chain against REAL loopback peers. The app is the ordinary
//! single-module source (`use.rs avrs`) carrying the delivered
//! HTTP-over-transport glue (`fixtures/tls1123/https_app.at`): the request
//! bytes are built in Auto and the response is parsed in Auto, the `tls`
//! provider only moves them. The provider closure is PREPARED by the
//! explicit `prepare-avrs-tls` step (never compiled here — the test fails
//! with the lane's named `prepared.*` error when it is missing or stale).
//!
//! Independent oracles: `status * 100000 + body_len * 10 + body_match` is
//! hand-computed from the peer's fixed bodies (GET: 200 / "hello over tls"
//! = 14 bytes / match = 20000141; POST echo: 201 / "ping-42" = 7 bytes /
//! match = 20100071); the peers record the request
//! bytes they really received, so "no request byte reached the peer" is an
//! observation, not an inference from the app's value.
#![cfg(windows)]

#[path = "../fixtures/tls1123/peers.rs"]
mod peers;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use ace_abi::rust_provider::{
    compose_delivered_avrs, describe_rust_provider, f0_authority, open_prepared_avrs_session,
    prepared_avrs_closure, verify_provider_closure, F0Session,
};
use ace_av_core::contract::{ExecOutcome, Value};

/// Transient scratch under the OS temp dir, removed on drop.
struct Scope(PathBuf);
impl std::ops::Deref for Scope {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn scope(tag: &str) -> Scope {
    let root = std::env::temp_dir().join(format!("p1123-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    Scope(root)
}

/// A fixture written to disk under its trust-source name (forward slashes:
/// the path is embedded in an Auto string literal).
fn fixture_file(dir: &Path, name: &str, bytes: &[u8]) -> String {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path.to_string_lossy().replace('\\', "/")
}

/// The app: the ordinary single-module source importing the composed
/// provider, prefixed by the delivered HTTP-over-transport glue.
fn app(main: &str) -> String {
    format!("use.rs avrs\n{}\n{main}", peers::HTTPS_APP)
}

fn caps(foreign: bool) -> Vec<String> {
    let mut caps = vec!["ac.source.language.av/v1".to_string()];
    if foreign {
        caps.push(ace_foreign_contract::manual::SOURCE_CAP.to_string());
        caps.push(ace_foreign_contract::resource::RESOURCE_CAP.to_string());
    }
    caps
}

/// The case's wall time, reported for the Plan's timing table.
fn timed<T>(case: &str, body: impl FnOnce() -> T) -> T {
    let started = Instant::now();
    let value = body();
    eprintln!("[p1123-timing] {case} {} ms", started.elapsed().as_millis());
    value
}

/// Open the PREPARED closure of the delivered stdlib unit. Never builds.
fn session(generation: u64) -> F0Session {
    let started = Instant::now();
    let session = open_prepared_avrs_session(generation)
        .unwrap_or_else(|error| panic!("the daily lane never builds: {error}"));
    eprintln!(
        "[p1123-timing] prepared-session-open {} ms",
        started.elapsed().as_millis()
    );
    session
}

/// Compile the app on the ordinary AV chain and execute it through the
/// session (the PLAN-1114 lane's spelling).
fn run(source: &str, session: &mut F0Session) -> ExecOutcome {
    let program = ace_av::foreign_source::compile_foreign_program(
        source,
        "main.at",
        "d_main",
        &caps(true),
        &session.admitted,
    )
    .expect("the app compiles against the admitted provider");
    let decoded = ace_av_core::abc::decode_abc(&program.abc).unwrap();
    let loaded = ace_av_core::abc_verify::verify_abc_foreign(
        decoded,
        &program.owner,
        session.session.table_shared(),
    )
    .expect("the program admits against the session");
    session
        .session
        .execute(
            &loaded,
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(false),
        )
        .expect("authorized run")
        .0
}

/// The same run under a host-chosen execution wall (the default is 2 s).
fn run_with_wall(source: &str, session: &mut F0Session, wall: Duration) -> ExecOutcome {
    let program = ace_av::foreign_source::compile_foreign_program(
        source,
        "main.at",
        "d_main",
        &caps(true),
        &session.admitted,
    )
    .expect("the app compiles against the admitted provider");
    let decoded = ace_av_core::abc::decode_abc(&program.abc).unwrap();
    let loaded = ace_av_core::abc_verify::verify_abc_foreign(
        decoded,
        &program.owner,
        session.session.table_shared(),
    )
    .expect("the program admits against the session");
    let limits = ace_av_core::av::ExecLimits {
        wall,
        ..ace_av_core::av::ExecLimits::default()
    };
    session
        .session
        .execute(&loaded, &limits, &ace_av_core::av::CancelFlag::new(false))
        .expect("authorized run")
        .0
}

fn exit_code(out: ExecOutcome) -> i32 {
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("expected Success(I32), got {other:?}"),
    }
}

fn expect_handle_rejection(out: ExecOutcome, what: &str) {
    match &out {
        ExecOutcome::Infra(message) if message.contains("foreign.handle") => {}
        other => {
            panic!("{what}: expected the owned model's `foreign.handle` rejection, got {other:?}")
        }
    }
}

/// A one-request app: `tls_send(spec, GET /ok, "")`.
fn send_main(spec: &str, port: u16, want: &str) -> String {
    let request = format!("tls_get_req(\"127.0.0.1\", {port}, \"/ok\")");
    format!("fn main() int {{\n    return tls_send({spec}, {request}, \"{want}\")\n}}")
}

/// GET /ok: status 200, the 14 bytes of "hello over tls", every byte equal.
const OK: i32 = 200 * 100_000 + 14 * 10 + 1;

#[test]
fn v01_direct_https_peer_observes_request_status_and_bytes() {
    timed("v01_direct_https", || {
        let scope = scope("v01");
        let ca_pem = fixture_file(&scope, "ca-test.pem", peers::CA_TEST_PEM);
        let ca_der = fixture_file(&scope, "ca-test.der", peers::CA_TEST_DER);
        let server = peers::tls_server(peers::LEAF_OK_SERVER, peers::CloseStyle::Notify);
        let port = server.addr.port();
        let mut session = session(1);

        // by DNS name: SNI plus the SAN hostname decision, a PEM trust source
        let dns = format!(
            "fn main() int {{\n    let spec_a = tls_spec(\"localhost\", {port}, \"file:{ca_pem}\")\n    let a = tls_send(spec_a, tls_get_req(\"localhost\", {port}, \"/ok\"), \"hello over tls\")\n    return a\n}}"
        );
        assert_eq!(
            exit_code(run(&app(&dns), &mut session)),
            OK,
            "direct HTTPS GET by name"
        );
        // by IP literal: the IP SAN; the method, the path and the body bytes
        // really traverse; a DER trust source
        let ip = format!(
            "fn main() int {{\n    let spec_b = tls_spec(\"127.0.0.1\", {port}, \"file:{ca_der}\")\n    let b = tls_send(spec_b, tls_post_req(\"127.0.0.1\", {port}, \"/echo\", \"ping-42\"), \"ping-42\")\n    let spec_c = tls_spec(\"127.0.0.1\", {port}, \"file:{ca_der}\")\n    let c = tls_send(spec_c, tls_get_req(\"127.0.0.1\", {port}, \"/ok\"), \"hello over tls\")\n    return b + c\n}}"
        );
        assert_eq!(
            exit_code(run(&app(&ip), &mut session)),
            201 * 100_000 + 7 * 10 + 1 + OK,
            "POST echo and GET by IP"
        );

        // the supported single-hop redirect over the secure transport:
        // /start answers 302 + Location, the caller re-requests /next
        let redirect = format!(
            "fn main() int {{\n    let spec_d = tls_spec(\"127.0.0.1\", {port}, \"file:{ca_pem}\")\n    let hop = tls_send(spec_d, tls_get_req(\"127.0.0.1\", {port}, \"/start\"), \"\")\n    let spec_e = tls_spec(\"127.0.0.1\", {port}, \"file:{ca_pem}\")\n    let next = tls_send(spec_e, tls_get_req(\"127.0.0.1\", {port}, \"/next\"), \"landed\")\n    return hop + next\n}}"
        );
        // 302 / 0 bytes / match + 200 / "landed" = 6 bytes / match
        assert_eq!(
            exit_code(run(&app(&redirect), &mut session)),
            302 * 100_000 + 1 + 200 * 100_000 + 6 * 10 + 1,
            "redirect hop and landing over HTTPS"
        );

        // the peer's own record: what really arrived, in order
        let received = server.received();
        assert_eq!(received.len(), 5, "one request per exchange");
        let get = String::from_utf8_lossy(&received[0]).to_string();
        assert!(
            get.starts_with(&format!("GET /ok HTTP/1.1\r\nHost: localhost:{port}\r\n")),
            "{get:?}"
        );
        let post = String::from_utf8_lossy(&received[1]).to_string();
        assert!(
            post.starts_with("POST /echo HTTP/1.1\r\n")
                && post.contains("\r\nContent-Length: 7\r\n")
                && post.ends_with("\r\n\r\nping-42"),
            "{post:?}"
        );
        assert!(
            String::from_utf8_lossy(&received[2])
                .starts_with(&format!("GET /ok HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n")),
            "{:?}",
            String::from_utf8_lossy(&received[2])
        );
        assert!(
            String::from_utf8_lossy(&received[3]).starts_with("GET /start HTTP/1.1\r\n")
                && String::from_utf8_lossy(&received[4]).starts_with("GET /next HTTP/1.1\r\n"),
            "the redirect hop and its landing both reached the peer"
        );
        // every exchange really dispatched: open + result + bytes + close
        assert!(
            session.session.table_shared().callee_count() > 5 * 4,
            "every exchange really dispatched through the provider"
        );
    });
}

#[test]
fn v01_verification_failures_are_named_and_send_nothing() {
    timed("v01_verification_negatives", || {
        let scope = scope("v01-neg");
        let ca = fixture_file(&scope, "ca-test.pem", peers::CA_TEST_PEM);
        let named = peers::tls_server(peers::LEAF_NAME_SERVER, peers::CloseStyle::Notify);
        let untrusted = peers::tls_server(peers::LEAF_CA_SERVER, peers::CloseStyle::Notify);
        let trusted = peers::tls_server(peers::LEAF_OK_SERVER, peers::CloseStyle::Notify);
        let mut session = session(1);
        let file = format!("file:{ca}");
        let mut send = |port: u16, trust: &str, want: &str| {
            let main = send_main(
                &format!("tls_spec(\"127.0.0.1\", {port}, \"{trust}\")"),
                port,
                want,
            );
            exit_code(run(&app(&main), &mut session))
        };
        // the leaf is valid for other.invalid only: -10 tls.hostname
        assert_eq!(send(named.addr.port(), &file, ""), -10);
        // the leaf chains to an issuer outside the trust source: -9 tls.untrusted
        assert_eq!(send(untrusted.addr.port(), &file, ""), -9);
        // the native root store does not trust the private test CA: -9
        assert_eq!(send(trusted.addr.port(), "system", ""), -9);
        // no trust source is a malformed spec: -1 (there is no implicit trust)
        assert_eq!(send(trusted.addr.port(), "", ""), -1);
        // an unreadable trust file: -8
        assert_eq!(
            send(trusted.addr.port(), &format!("file:{ca}.missing"), ""),
            -8
        );
        // port 0 is not a port: the spec is malformed before anything is sent
        assert_eq!(send(0, &file, ""), -1);
        assert!(
            named.received().is_empty()
                && untrusted.received().is_empty()
                && trusted.received().is_empty(),
            "a verification failure sends no request byte"
        );
        // nothing carried over: the trusted request passes right after the failures
        assert_eq!(send(trusted.addr.port(), &file, "hello over tls"), OK);
        assert_eq!(trusted.received().len(), 1);
    });
}

#[test]
fn v02_proxy_connect_plain_forward_and_verdicts() {
    timed("v02_proxy", || {
        let scope = scope("v02");
        let ca = fixture_file(&scope, "ca-test.pem", peers::CA_TEST_PEM);
        let server = peers::tls_server(peers::LEAF_OK_SERVER, peers::CloseStyle::Notify);
        let origin = peers::plain_origin();
        let forwarding = peers::proxy(peers::ProxyBehavior::Forward);
        let via = format!("127.0.0.1:{}", forwarding.addr.port());
        let tls_port = server.addr.port();
        let mut session = session(1);
        let tunnel = || {
            send_main(
                &format!("tls_spec_via(\"127.0.0.1\", {tls_port}, \"file:{ca}\", \"{via}\")"),
                tls_port,
                "hello over tls",
            )
        };
        assert_eq!(
            exit_code(run(&app(&tunnel()), &mut session)),
            OK,
            "HTTPS through CONNECT"
        );
        // the released exchange does not hold the proxy: the next request runs
        assert_eq!(
            exit_code(run(&app(&tunnel()), &mut session)),
            OK,
            "HTTPS through CONNECT again"
        );
        let heads = forwarding.received();
        assert_eq!(heads.len(), 2);
        let connect = String::from_utf8_lossy(&heads[0]).to_string();
        assert_eq!(
            connect,
            format!("CONNECT 127.0.0.1:{tls_port} HTTP/1.1\r\nHost: 127.0.0.1:{tls_port}\r\n\r\n")
        );
        assert_eq!(
            server.received().len(),
            2,
            "the TLS peer received both real requests"
        );

        // plain HTTP forwarded by the proxy: the absolute-form bytes verbatim
        let plain_port = origin.addr.port();
        let plain = send_main(
            &format!("tls_spec_http(\"127.0.0.1\", {plain_port}, \"{via}\")"),
            plain_port,
            "hello over tls",
        )
        .replace(
            &format!("tls_get_req(\"127.0.0.1\", {plain_port}, \"/ok\")"),
            &format!("tls_proxy_get_req(\"127.0.0.1\", {plain_port}, \"/ok\")"),
        );
        assert_eq!(
            exit_code(run(&app(&plain), &mut session)),
            OK,
            "plain HTTP through the proxy"
        );
        let heads = forwarding.received();
        let absolute = String::from_utf8_lossy(&heads[2]).to_string();
        assert!(
            absolute.starts_with(&format!(
                "GET http://127.0.0.1:{plain_port}/ok HTTP/1.1\r\n"
            )),
            "{absolute:?}"
        );
        let forwarded = String::from_utf8_lossy(&origin.received()[0]).to_string();
        assert!(
            forwarded.starts_with("GET /ok HTTP/1.1\r\n"),
            "{forwarded:?}"
        );

        // verdicts: rejected / closed / stalled are named apart
        let rejecting = peers::proxy(peers::ProxyBehavior::Reject(403));
        let via = format!("127.0.0.1:{}", rejecting.addr.port());
        let rejected = send_main(
            &format!("tls_spec_via(\"127.0.0.1\", {tls_port}, \"file:{ca}\", \"{via}\")"),
            tls_port,
            "",
        );
        assert_eq!(
            exit_code(run(&app(&rejected), &mut session)),
            -5,
            "proxy.rejected"
        );
        let closing = peers::proxy(peers::ProxyBehavior::Close);
        let via = format!("127.0.0.1:{}", closing.addr.port());
        let closed = send_main(
            &format!("tls_spec_via(\"127.0.0.1\", {tls_port}, \"file:{ca}\", \"{via}\")"),
            tls_port,
            "",
        );
        assert_eq!(
            exit_code(run(&app(&closed), &mut session)),
            -6,
            "proxy.closed"
        );
        let stalling = peers::proxy(peers::ProxyBehavior::Stall);
        let via = format!("127.0.0.1:{}", stalling.addr.port());
        let stalled = send_main(
            &format!("tls_spec_via(\"127.0.0.1\", {tls_port}, \"file:{ca}\", \"{via}\") + \"timeout 400\\n\""),
            tls_port,
            "",
        );
        let started = Instant::now();
        assert_eq!(
            exit_code(run(&app(&stalled), &mut session)),
            -4,
            "net.timeout"
        );
        let elapsed = started.elapsed();
        assert!(
            elapsed >= Duration::from_millis(380) && elapsed < Duration::from_secs(3),
            "the stalled CONNECT times out at its deadline: {elapsed:?}"
        );
        // a proxy reply that carries no HTTP status is named apart
        let garbling = peers::proxy(peers::ProxyBehavior::Garbage);
        let via = format!("127.0.0.1:{}", garbling.addr.port());
        let garbled = send_main(
            &format!("tls_spec_via(\"127.0.0.1\", {tls_port}, \"file:{ca}\", \"{via}\")"),
            tls_port,
            "",
        );
        assert_eq!(
            exit_code(run(&app(&garbled), &mut session)),
            -7,
            "proxy.bad"
        );
        assert_eq!(
            server.received().len(),
            2,
            "no failed proxy reaches the peer"
        );
        // after the refused, closed and stalled proxies a healthy request still runs
        assert_eq!(exit_code(run(&app(&tunnel()), &mut session)), OK);
    });
}

#[test]
fn refused_connect_is_timeout_then_connect_by_the_deadline() {
    timed("refused_connect", || {
        let scope = scope("refused");
        let ca = fixture_file(&scope, "ca-test.pem", peers::CA_TEST_PEM);
        let mut session = session(1);
        // a refused port under the default deadline: the Windows stack retries
        // a refused SYN for ~2 s, longer than the deadline, so the attempt is
        // the named timeout — never a hang
        let dead = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let dead_port = dead.local_addr().unwrap().port();
        drop(dead);
        let refused = send_main(
            &format!("tls_spec(\"127.0.0.1\", {dead_port}, \"file:{ca}\")"),
            dead_port,
            "",
        );
        let started = Instant::now();
        assert_eq!(
            exit_code(run(&app(&refused), &mut session)),
            -4,
            "net.timeout under the default deadline"
        );
        assert!(
            started.elapsed() < Duration::from_millis(1900),
            "{:?}",
            started.elapsed()
        );
        // with a longer timeout the caller must raise the execution wall: the
        // stack then gives up by itself and the refusal is the named connect failure
        let refused = send_main(
            &format!("tls_spec(\"127.0.0.1\", {dead_port}, \"file:{ca}\") + \"timeout 4000\\n\""),
            dead_port,
            "",
        );
        assert_eq!(
            exit_code(run_with_wall(
                &app(&refused),
                &mut session,
                Duration::from_secs(6)
            )),
            -3,
            "net.connect once the stack gives up"
        );
        // the same long request under the DEFAULT wall is the executor's
        // wall-budget outcome, not a provider value (the documented contract)
        let long = run(&app(&refused), &mut session);
        assert!(
            matches!(
                &long,
                ExecOutcome::Resource {
                    kind: ace_av_core::contract::ResourceKind::ExecutionWall,
                    ..
                }
            ),
            "the default wall preempts a longer request: {long:?}"
        );
    });
}

#[test]
fn timeout_refused_and_truncated_close_are_named() {
    timed("timeout_refused_truncated", || {
        let scope = scope("timeouts");
        let ca = fixture_file(&scope, "ca-test.pem", peers::CA_TEST_PEM);
        let mut session = session(1);

        // a peer that never answers the handshake: the deadline names it
        let silent = peers::silent();
        let stalled = send_main(
            &format!(
                "tls_spec(\"127.0.0.1\", {}, \"file:{ca}\") + \"timeout 300\\n\"",
                silent.addr.port()
            ),
            silent.addr.port(),
            "",
        );
        let started = Instant::now();
        assert_eq!(
            exit_code(run(&app(&stalled), &mut session)),
            -4,
            "net.timeout"
        );
        let elapsed = started.elapsed();
        assert!(
            elapsed >= Duration::from_millis(280) && elapsed < Duration::from_secs(3),
            "{elapsed:?}"
        );

        // the peer closes without close_notify: the outcome is named (-15)
        // and the bytes are kept — this caller observes the complete body
        let truncating = peers::tls_server(peers::LEAF_OK_SERVER, peers::CloseStyle::Silent);
        let port = truncating.addr.port();
        let truncated = send_main(
            &format!("tls_spec(\"127.0.0.1\", {port}, \"file:{ca}\")"),
            port,
            "hello over tls",
        );
        assert_eq!(
            exit_code(run(&app(&truncated), &mut session)),
            OK,
            "bytes kept across the truncated close"
        );
        assert_eq!(truncating.received().len(), 1);

        // the peer resets the connection mid-response: the read failure is
        // named (-14) and nothing of the partial response is kept
        let resetting = peers::tls_server(peers::LEAF_OK_SERVER, peers::CloseStyle::Reset);
        let port = resetting.addr.port();
        let reset = send_main(
            &format!("tls_spec(\"127.0.0.1\", {port}, \"file:{ca}\")"),
            port,
            "",
        );
        assert_eq!(exit_code(run(&app(&reset), &mut session)), -14, "net.read");
        assert!(
            resetting.received().is_empty(),
            "the resetting peer never read the request"
        );
        // the plain neighbor in the SAME provider unit is untouched
        let plain = "use.rs avrs\nfn main() int { return net_loopback_echo(\"A中😀\0\") }";
        assert_eq!(
            exit_code(run(plain, &mut session)),
            9,
            "plain loopback still answers"
        );
    });
}

#[test]
fn handle_lifecycle_closed_and_stale_are_rejected_before_the_callee() {
    timed("handle_lifecycle", || {
        let scope = scope("handles");
        let ca = fixture_file(&scope, "ca-test.pem", peers::CA_TEST_PEM);
        let server = peers::tls_server(peers::LEAF_OK_SERVER, peers::CloseStyle::Notify);
        let port = server.addr.port();
        let exchange = format!("tls_exchange(tls_spec(\"127.0.0.1\", {port}, \"file:{ca}\") + \"\\n\" + tls_get_req(\"127.0.0.1\", {port}, \"/ok\"))");
        let mut first = session(1);
        // the handle escapes as the app's value (an opaque i32 to the guest)
        let handle = exit_code(run(
            &app(&format!("fn main() int {{\n    return {exchange}\n}}")),
            &mut first,
        ));
        // closed: the same session rejects the handle, the callee never runs
        let closed = format!("fn main() int {{\n    let x = {exchange}\n    let _c = tls_close(x)\n    return tls_result(x)\n}}");
        expect_handle_rejection(run(&app(&closed), &mut first), "a closed exchange");
        // double close
        let twice = format!("fn main() int {{\n    let x = {exchange}\n    let _a = tls_close(x)\n    return tls_close(x)\n}}");
        expect_handle_rejection(run(&app(&twice), &mut first), "a double close");
        // a new generation rejects the OLD generation's handle before execution
        let mut second = session(2);
        expect_handle_rejection(
            run(
                &app(&format!(
                    "fn main() int {{\n    return tls_result({handle})\n}}"
                )),
                &mut second,
            ),
            "a stale-generation handle",
        );
        // the old session still answers its own legal handle afterwards
        assert_eq!(
            exit_code(run(
                &app(&format!(
                    "fn main() int {{\n    return tls_result({handle})\n}}"
                )),
                &mut first
            )),
            0
        );
    });
}

#[test]
fn v03_revoked_grants_reject_before_any_dispatch() {
    timed("v03_revoked_grants", || {
        let scope = scope("grants");
        let ca = fixture_file(&scope, "ca-test.pem", peers::CA_TEST_PEM);
        let server = peers::tls_server(peers::LEAF_OK_SERVER, peers::CloseStyle::Notify);
        let session = session(1);
        let source = app(&send_main(
            &format!(
                "tls_spec(\"127.0.0.1\", {}, \"file:{ca}\")",
                server.addr.port()
            ),
            server.addr.port(),
            "",
        ));
        let text = match ace_av::foreign_source::compile_foreign_program(
            &source,
            "main.at",
            "d_main",
            &caps(false),
            &session.admitted,
        ) {
            Err(error) => format!("{error:?}"),
            Ok(_) => panic!("a program without the foreign grants must reject"),
        };
        assert!(
            text.contains("foreign"),
            "the named capability rejection: {text}"
        );
        assert_eq!(
            session.session.table_shared().callee_count(),
            0,
            "zero dispatch"
        );
        assert!(
            server.received().is_empty(),
            "no request byte reaches the peer"
        );
    });
}

#[test]
fn v03_provider_missing_and_prepared_closure_is_source_bound() {
    timed("v03_provider_and_closure", || {
        let (closure, source) = prepared_avrs_closure().expect("the prepared stdlib closure");
        // a provider WITHOUT the `tls` layer: the call is the named
        // undefined-symbol rejection at compile (nothing is built anywhere)
        let without_tls = compose_delivered_avrs(&["str"]).expect("the str layer composes");
        let described = describe_rust_provider(
            &without_tls,
            "rustc-probe".to_string(),
            PathBuf::from("unused.dll"),
            PathBuf::from("unused.lib"),
        )
        .expect("describes without building");
        let admitted = described
            .manual
            .clone()
            .admit(&f0_authority(&described, 1))
            .expect("admits");
        let text = match ace_av::foreign_source::compile_foreign_program(
            &app("fn main() int { return tls_exchange(\"host a\\nport 1\\ntrust system\\n\\nGET / HTTP/1.1\\r\\n\\r\\n\") }"),
            "main.at",
            "d_main",
            &caps(true),
            &admitted,
        )
        {
            Err(error) => error.to_string(),
            Ok(_) => panic!("a provider without the tls layer must reject the call"),
        };
        // the first missing op the binder meets is named, and it is one of
        // the delivered `tls` contract's own operations
        let contract = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../stdlib/ace/avrs/tls.at"
        ))
        .unwrap();
        let named = text
            .split("no function named `")
            .nth(1)
            .and_then(|rest| rest.split('`').next())
            .unwrap_or("");
        assert!(
            text.contains("source.undefined"),
            "the rejection names the missing op: {text}"
        );
        assert!(
            contract.contains(&format!("pub spec fn {named}(")),
            "the rejected op `{named}` is a delivered tls operation: {text}"
        );

        // a missing closure is the named preparation failure and stays missing
        let scope = scope("closure");
        let missing = scope.join("nowhere");
        let error = verify_provider_closure(&missing, &source).expect_err("no closure there");
        assert!(error.starts_with("prepared.missing"), "{error}");
        assert!(
            !missing.exists(),
            "verification never creates or builds anything"
        );

        // the closure is bound to its source: an edited source is stale
        let error =
            verify_provider_closure(&closure.root, &format!("{source}\n// an edited provider\n"))
                .expect_err("an edited source must not reuse the closure");
        assert!(
            error.starts_with("prepared.stale") && error.contains("provider source"),
            "{error}"
        );

        // a tampered artifact is corrupt (the record is real, the DLL is not)
        let tampered = scope.join("tampered");
        std::fs::create_dir_all(tampered.join("cargo-unit/target/debug")).unwrap();
        std::fs::copy(
            closure.root.join("closure.json"),
            tampered.join("closure.json"),
        )
        .unwrap();
        std::fs::create_dir_all(tampered.join("cargo-unit")).unwrap();
        std::fs::copy(
            closure.root.join("cargo-unit/Cargo.lock"),
            tampered.join("cargo-unit/Cargo.lock"),
        )
        .unwrap();
        std::fs::write(
            tampered.join("cargo-unit/target/debug/provider_unit.dll"),
            b"not the provider",
        )
        .unwrap();
        let error =
            verify_provider_closure(&tampered, &source).expect_err("a tampered DLL must reject");
        assert!(error.starts_with("prepared.corrupt"), "{error}");
        assert!(
            !tampered.join("cargo-unit/Cargo.toml").exists(),
            "no build was attempted"
        );

        // the real closure verifies and names exactly what it bound
        let verified =
            verify_provider_closure(&closure.root, &source).expect("the real closure verifies");
        assert_eq!(verified.provider_digest, closure.provider_digest);
        assert_eq!(
            verified
                .lock
                .as_deref()
                .map(|lock| lock.contains("name = \"rustls\"")),
            Some(true)
        );
    });
}
