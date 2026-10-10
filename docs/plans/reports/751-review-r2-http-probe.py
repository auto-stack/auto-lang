"""Verify real HTTP fixture handler closures, without starting a server.

Adds only a temporary cfg(test) diagnostic, invokes the production bytecode
preflight verifier, then restores exact original bytes.
"""
import json
import pathlib
import re
import subprocess

ROOT = pathlib.Path("D:/autostack/.wt/review-751-r2/auto-lang")
SOURCE = ROOT / "crates/auto-lang/src/tests/plan730_http_upload_tests.rs"
LOG = ROOT / ".review751-r2-http.log"
PROBE = r'''
    #[test]
    fn review751_http_entry_contracts() {
        fn check(source: &str, names: &[&str], failures: &mut Vec<String>) {
            // Exercise exactly the entry used by both original server helpers.
            // A bounded channel prevents a successful server startup from
            // leaving the review blocked forever; :0 avoids fixed-port overlap.
            std::env::set_var("AUTO_HTTP_PORT", "0");
            let owned = source.to_string();
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::Builder::new().stack_size(8 * 1024 * 1024)
                .spawn(move || {
                    let result = crate::run(&owned).map(|_| "Ok".to_string())
                        .map_err(|error| format!("{error:?}"));
                    let _ = tx.send(result);
                }).unwrap();
            match rx.recv_timeout(std::time::Duration::from_secs(8)) {
                Ok(result) => eprintln!("REVIEW751_HTTP actual crate::run: {result:?}"),
                Err(error) => eprintln!("REVIEW751_HTTP actual crate::run bounded wait: {error:?}"),
            }
            let (vm, _, _, _) = match crate::create_vm_from_source(source) {
                Ok(value) => value,
                Err(error) => {
                    let message = format!("compile: {error:?}");
                    eprintln!("REVIEW751_HTTP {message}");
                    failures.push(message);
                    return;
                }
            };
            for name in names {
                let (&entry, _) = vm.flash.addr_to_name.iter()
                    .find(|(_, n)| n.as_str() == *name).expect("fixture handler entry");
                let result = crate::stdlib_assembly::reference::verify_linked_native_closure(
                    &vm.flash, entry as usize, &vm.native_interface,
                    &vm.strings.read().unwrap());
                match result {
                    Ok(proofs) => eprintln!("REVIEW751_HTTP {name}: PASS proofs={}", proofs.len()),
                    Err(error) => {
                        eprintln!("REVIEW751_HTTP {name}: {error}");
                        failures.push(format!("{name}: {error}"));
                    }
                }
            }
        }
        let mut failures = Vec::new();
        let (root, staging) = temp_roots("review751-preflight");
        check(&upload_program(&root, &staging),
            &["upload_mp", "upload_raw", "upload_early", "upload_guarded", "download", "plain_int"],
            &mut failures);
        // Exact source from plan326 e2e_a_redirect_302_with_location.
        check(r#"
#[api(method = "GET", path = "/old")]
fn old_handler() int {
    return http.response_redirect("/new", 302)
}

#[api(method = "GET", path = "/new")]
fn new_handler() str {
    return "arrived"
}
"#, &["old_handler", "new_handler"], &mut failures);
        assert!(failures.is_empty(), "HTTP fixture handler preflight failed: {failures:?}");
    }
'''

def main():
    if subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True).strip():
        raise RuntimeError("Requires clean verification worktree")
    original = SOURCE.read_bytes()
    source = original.decode("utf-8")
    match = re.search(r"mod http_e2e \{\r?\n", source)
    if not match:
        raise RuntimeError("HTTP fixture module unavailable")
    newline = "\r\n" if "\r\n" in source else "\n"
    patched = source[:match.end()] + PROBE.replace("\n", newline) + source[match.end():]
    command = ["cargo", "test", "-p", "auto-lang", "--lib", "--features",
               "test-http-e2e", "review751_http_entry_contracts", "--",
               "--test-threads=1", "--nocapture"]
    try:
        SOURCE.write_bytes(patched.encode("utf-8"))
        with LOG.open("w", encoding="utf-8") as log:
            result = subprocess.run(command, cwd=ROOT, stdout=log,
                                    stderr=subprocess.STDOUT, timeout=600)
    finally:
        SOURCE.write_bytes(original)
        assert SOURCE.read_bytes() == original
    highlights = [s for s in LOG.read_text(encoding="utf-8").splitlines()
                  if "REVIEW751_HTTP" in s or "test result:" in s or "panicked at" in s]
    print(json.dumps({"cargo_exit": result.returncode, "restored_exact_bytes": True,
                      "highlights": highlights}, ensure_ascii=False))

if __name__ == "__main__":
    main()
