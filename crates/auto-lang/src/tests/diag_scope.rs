//! PLAN-1066 A-NOWRITE test-side entry point.
//!
//! The implementation lives in `auto_lang_core::diag_scope` (the entry
//! runners in auto-lang-core share the exact same writer); this module
//! re-exports it for the legacy wrappers and carries the behavior test that
//! the corpus group runs daily.

pub(crate) use auto_lang_core::diag_scope::write_scoped;

#[cfg(test)]
mod tests {
    use super::*;

    // One test function on purpose: both halves mutate the same env var, and
    // a bare `cargo test` run shares one process across tests.
    #[test]
    fn diag_write_drops_by_default_and_mirrors_under_scope() {
        // No scope configured: nothing is written anywhere, and the writer
        // still succeeds (the red verdict itself carries the failure).
        std::env::remove_var("AUTO_TEST_DIAG_SCOPE");
        let out = write_scoped("test/vm/xx/yy.wrong.out", b"payload").unwrap();
        assert!(out.is_none(), "default must drop, not write");

        // Scope configured: the payload is mirrored inside it, never into
        // the author tree.
        let scope = tempfile::tempdir().expect("scope dir");
        std::env::set_var("AUTO_TEST_DIAG_SCOPE", scope.path());
        let out = write_scoped("test/vm/01_basics/001_hello/hello.wrong.out", b"abc")
            .expect("scoped write");
        let path = out.expect("scope configured → mirrored target");
        std::env::remove_var("AUTO_TEST_DIAG_SCOPE");
        assert!(path.starts_with(scope.path()), "must stay inside the scope");
        assert_eq!(std::fs::read(&path).unwrap(), b"abc");
    }
}
