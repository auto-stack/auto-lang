"""Exercise the committed startup helper's panic-reporting branch.

The only injected behavior is a marker panic in the cfg(test) server thread.
The helper's catch/format/channel/readiness code remains unchanged. Restore
the original source bytes even when the negative test fails.
"""
import json
import pathlib
import subprocess

ROOT = pathlib.Path("D:/autostack/.wt/lang-755/auto-lang")
SOURCE = ROOT / "crates/auto-lang/src/tests/plan730_http_upload_tests.rs"
LOG = ROOT / ".review755-panic.log"


def main():
    if subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True).strip():
        raise RuntimeError("Requires a clean plan worktree")
    original = SOURCE.read_bytes()
    text = original.decode("utf-8").replace("\r\n", "\n")
    before = "|| crate::run(&code),"
    after = '''|| {
                        if code == "__review755_panic__" {
                            panic!("REVIEW755_ORIGINAL_PANIC_PAYLOAD");
                        }
                        crate::run(&code)
                    },'''
    if text.count(before) != 1:
        raise RuntimeError("Expected one unmodified server-thread invocation")
    patched = text.replace(before, after, 1) + r'''

#[cfg(feature = "test-http-e2e")]
#[test]
fn review755_panic_payload_is_forwarded() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let outcome = std::panic::catch_unwind(|| {
        http_e2e::start_server("__review755_panic__", port);
    }).expect_err("server-thread panic must fail startup");
    let message = outcome.downcast_ref::<String>().cloned()
        .or_else(|| outcome.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "<non-string parent panic>".to_string());
    eprintln!("REVIEW755_PARENT_FAILURE={message}");
    assert!(message.contains("REVIEW755_ORIGINAL_PANIC_PAYLOAD"),
        "startup helper discarded the original panic payload: {message}");
}
'''
    if b"\r\n" in original:
        patched = patched.replace("\n", "\r\n")
    command = ["cargo", "nextest", "run", "-p", "auto-lang", "--lib",
               "--features", "test-http-e2e", "--jobs", "1", "--no-fail-fast",
               "review755_panic_payload_is_forwarded"]
    try:
        SOURCE.write_bytes(patched.encode("utf-8"))
        with LOG.open("w", encoding="utf-8") as log:
            result = subprocess.run(command, cwd=ROOT, stdout=log,
                                    stderr=subprocess.STDOUT, timeout=180)
    finally:
        SOURCE.write_bytes(original)
        assert SOURCE.read_bytes() == original
    print(json.dumps({"exit": result.returncode, "restored_exact_bytes": True,
        "highlights": [line for line in LOG.read_text(encoding="utf-8").splitlines()
            if "REVIEW755" in line or "Summary" in line or "FAIL [" in line]},
        ensure_ascii=False))


if __name__ == "__main__":
    main()
