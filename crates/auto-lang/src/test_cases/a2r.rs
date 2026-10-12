//! a2r adapter (`a2r/emit`) — lands in Plan 768 T-05 (real emit artifacts,
//! manifest pre-verification, big-stack execution, proposal-only bless).

use std::path::Path;

use auto_test_case::contract::CheckSpec;
use auto_test_case::ir::ResolvedCase;

use super::avm::Verdict;

pub fn run_check(
    _case: &ResolvedCase,
    _check: &CheckSpec,
    _project_dir: &Path,
    _artifact_dir: &Path,
) -> Result<Verdict, String> {
    Err("a2r/emit adapter is not wired yet (Plan 768 T-05)".to_string())
}
