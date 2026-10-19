# PLAN-819 evidence files

These logs were copied from the Plan worktree's ignored Cargo target directory before cleanup. All 34 copies were checked against their source files with SHA-256.

## Merge gate

- `cargo-tb-r5-4threads.log.txt`: accepted final `cargo tb`; 5312 passed, 1547 skipped, 1 slow, exit 0.
- `cargo-tb-r5-standard-target.log.txt`: default-concurrency run stopped when the unchanged `plan707_wait_generator_park_drive_count_static` test timed out; its isolated rerun is `plan707-isolated-repro.log.txt`.
- `cargo-tb-r5-standard-target-retry.log.txt`: default-concurrency run stopped at the unchanged `plan492_m4_tests::m4_pkg_compile_chain::c1_prop_compare_in_init_alive` assertion; its isolated rerun is `plan492-isolated-repro.log.txt`.
- `cargo-tb-r5-final.log.txt`: the initial custom `CARGO_TARGET_DIR` attempt failed because five FFI fixtures could not find their expected release binaries. This is a runner-path configuration failure, not an accepted gate result.
- `cargo-tb-final.log.txt`: revision 4 gate that exposed the ch08/ch09 problems later fixed in Phase 3.

## Targeted listing checks

- `final-book_ch02_05.log.txt` and `final-book_ch03_08.log.txt`: Phase 1 final exact checks.
- `final-book_ch06_*.log.txt`: Phase 2 final exact checks for the ten ch06 listings.
- `phase3-book_ch08_*` and `phase3-book_ch09_02*`: Phase 3 initial mismatch and final exact checks.
- `ch06-baseline.log.txt`, `ch06-after-source-fix.log.txt`, and `ch06-after-commit.log.txt`: Phase 2 failure attribution and repair receipts.
