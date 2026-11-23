//! `ace.m2.p1123.tls-project` — T-03 (V-03): the AV/Rust stdlib secure
//! transport through the DELIVERED ordinary-project lane, consuming the
//! PREPARED closure of the secure-transport selection (`str` + `tls`: a
//! project that needs only the secure transport selects two layers instead
//! of the whole unit; the project lane never compiles it):
//! build → run → SDK pack → RELOCATE → run-from-SDK after the development
//! cache is deleted. The release closure's deployment dependencies are
//! recorded from the provider DLL's real PE imports.
mod common;

#[path = "../../abi/tests/fixtures/tls1123/peers.rs"]
mod peers;

use std::path::PathBuf;
use std::time::Instant;

use ace_abi::rust_provider::prepared_avrs_tls_closure;
use ace_m2_build::driver::{M2Build, ProjectInputs, RustProjectInputs};
use ace_m2_build::identity::TargetKind;

const GRANT: &str = "run:avrs-https";

/// The ordinary project: one module over the composed provider's public
/// names, its Rust provider declared with an optional prepared closure.
fn project(
    source: String,
    lock: Option<String>,
    prepared: Option<PathBuf>,
    app: &str,
) -> ProjectInputs {
    let mut inputs = ProjectInputs::new("avrs-https", "main", "main.at", app);
    inputs.caps = [
        "ac.source.language.av/v1",
        "foreign.scalar",
        "foreign.resource",
    ]
    .map(str::to_owned)
    .to_vec();
    inputs.rust_provider = Some(RustProjectInputs {
        source,
        cargo_lock: lock,
        prepared,
    });
    inputs
}

/// The app: direct HTTPS and CONNECT-tunneled HTTPS to the loopback peer,
/// then a request whose peer certificate the native root store rejects.
fn app(ca: &str, tls_port: u16, via: &str) -> String {
    format!(
        "use.rs avrs\n{}\nfn main() int {{\n    let direct = tls_send(tls_spec(\"127.0.0.1\", {tls_port}, \"file:{ca}\"), tls_get_req(\"127.0.0.1\", {tls_port}, \"/ok\"), \"hello over tls\")\n    let tunneled = tls_send(tls_spec_via(\"127.0.0.1\", {tls_port}, \"file:{ca}\", \"{via}\"), tls_get_req(\"127.0.0.1\", {tls_port}, \"/ok\"), \"hello over tls\")\n    let rejected = tls_send(tls_spec(\"127.0.0.1\", {tls_port}, \"system\"), tls_get_req(\"127.0.0.1\", {tls_port}, \"/ok\"), \"\")\n    return direct + tunneled + rejected\n}}",
        peers::HTTPS_APP
    )
}

/// 20000141 (direct) + 20000141 (tunneled) + (-9 tls.untrusted).
const EXPECTED: i32 = 200 * 100_000 + 14 * 10 + 1 + 200 * 100_000 + 14 * 10 + 1 - 9;

#[test]
fn p1123_tls_project_builds_runs_and_sdk_relocates_from_the_prepared_closure() {
    let started = Instant::now();
    let (closure, provider) = prepared_avrs_tls_closure().expect("the daily lane never builds");
    let lock = closure
        .lock
        .clone()
        .expect("the secure transport declares dependencies");
    let root = common::scratch("p1123-tls-project");
    let ca_path = root.join("ca-test.pem");
    std::fs::write(&ca_path, peers::CA_TEST_PEM).unwrap();
    let ca = ca_path.to_string_lossy().replace('\\', "/");
    let server = peers::tls_server(peers::LEAF_OK_SERVER, peers::CloseStyle::Notify);
    let forwarding = peers::proxy(peers::ProxyBehavior::Forward);
    let via = format!("127.0.0.1:{}", forwarding.addr.port());
    let inputs = project(
        provider,
        Some(lock),
        Some(closure.root.clone()),
        &app(&ca, server.addr.port(), &via),
    );

    // Development build + run on the adopted closure.
    let dev = root.join("dev");
    let mut driver = M2Build::new(&dev);
    let built = driver
        .build(TargetKind::Av, &inputs)
        .expect("the project builds");
    assert_eq!(
        built.run.as_ref().and_then(|run| run.exit_code),
        Some(EXPECTED),
        "the secure transport answers on the development run"
    );
    assert!(
        !dev.join("scratch").join("rust-providers").exists(),
        "the project lane adopted the prepared closure: no provider was compiled"
    );

    // SDK pack: the release closure declares what the provider DLL imports.
    let sdk = root.join("sdk");
    let recipe = driver
        .build_sdk(&inputs, &[TargetKind::Av], &sdk, vec![], GRANT)
        .expect("pack SDK");
    let declared: Vec<&str> = recipe
        .run_dependencies
        .iter()
        .filter(|dependency| dependency.kind == "system-lib")
        .map(|dependency| dependency.name.as_str())
        .collect();
    eprintln!("[p1123-timing] sdk system-lib run dependencies: {declared:?}");
    // the provider DLL's real PE imports (read from the prepared closure's
    // unit, lower-cased as the packager records them): what a relocated SDK
    // host must provide
    const IMPORTS: [&str; 12] = [
        "advapi32.dll",
        "api-ms-win-core-synch-l1-2-0.dll",
        "api-ms-win-crt-heap-l1-1-0.dll",
        "api-ms-win-crt-runtime-l1-1-0.dll",
        "api-ms-win-crt-string-l1-1-0.dll",
        "bcrypt.dll",
        "bcryptprimitives.dll",
        "crypt32.dll",
        "kernel32.dll",
        "ntdll.dll",
        "vcruntime140.dll",
        "ws2_32.dll",
    ];
    assert_eq!(
        declared, IMPORTS,
        "the SDK declares exactly the provider's imports"
    );

    // Relocate → delete the development cache → run from the moved SDK.
    drop(driver);
    std::fs::rename(&sdk, root.join("moved")).expect("relocate SDK");
    std::fs::remove_dir_all(&dev).expect("remove development cache");
    let run = M2Build::new(&root.join("consumer"))
        .run_sdk(&root.join("moved"), TargetKind::Av, GRANT)
        .expect("run from the relocated SDK");
    assert_eq!(
        run.exit_code,
        Some(EXPECTED),
        "the transport survives distribution"
    );
    eprintln!(
        "[p1123-timing] sdk project total {} ms",
        started.elapsed().as_millis()
    );

    // the peers' own records: two runs, two requests each
    assert_eq!(
        server.received().len(),
        4,
        "direct + tunneled, development and SDK run"
    );
    assert_eq!(forwarding.received().len(), 2, "one CONNECT per run");
    std::fs::remove_dir_all(&root).expect("remove test scratch");
}

#[test]
fn p1123_prepared_closure_must_match_the_project_exactly() {
    let started = Instant::now();
    let (closure, provider) = prepared_avrs_tls_closure().expect("the daily lane never builds");
    let lock = closure
        .lock
        .clone()
        .expect("the secure transport declares dependencies");
    let app = "fn main() int { return 0 }";
    let root = common::scratch("p1123-prepared-rejects");
    let reject = |tag: &str, inputs: ProjectInputs| {
        let scratch = root.join(tag);
        let outcome = M2Build::new(&scratch).build(TargetKind::Av, &inputs);
        let error = match outcome {
            Err(error) => error,
            Ok(_) => panic!("{tag}: the project must reject the prepared closure"),
        };
        assert_eq!(error.code, "m2build.rust-prepared", "{tag}: {error:?}");
        assert!(
            !scratch.join("scratch").join("rust-providers").exists(),
            "{tag}: a rejected closure is never compiled in its place"
        );
        error.detail
    };

    // a project declaring another resolution than the closure was built with
    let detail = reject(
        "another-resolution",
        project(
            provider.clone(),
            Some(format!("{lock}\n# another resolution\n")),
            Some(closure.root.clone()),
            app,
        ),
    );
    assert!(detail.contains("resolution"), "{detail}");

    // a closure directory that does not exist
    let detail = reject(
        "missing",
        project(
            provider.clone(),
            Some(lock.clone()),
            Some(root.join("nowhere")),
            app,
        ),
    );
    assert!(detail.starts_with("prepared.missing"), "{detail}");

    // a closure prepared from another provider source
    let detail = reject(
        "another-source",
        project(
            format!("{provider}\n// an edited provider\n"),
            Some(lock),
            Some(closure.root.clone()),
            app,
        ),
    );
    assert!(detail.starts_with("prepared.stale"), "{detail}");
    eprintln!(
        "[p1123-timing] prepared rejections {} ms",
        started.elapsed().as_millis()
    );
    std::fs::remove_dir_all(&root).expect("remove test scratch");
}
