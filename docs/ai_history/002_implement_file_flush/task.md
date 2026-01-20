# Implement File.flush()

- [x] Read `auto-stdlib-dev` skill instructions <!-- id: 0 -->
- [x] Analyze `stdlib/auto/io.at` and related files <!-- id: 1 -->
- [x] Create implementation plan <!-- id: 2 -->
- [x] Interface: Add `flush()` to `stdlib/auto/io.at` <!-- id: 3 -->
- [x] C Impl: Add `flush()` to `stdlib/auto/io.c.at` <!-- id: 4 -->
- [x] VM Interface: Add `#[vm] flush()` to `stdlib/auto/io.vm.at` <!-- id: 5 -->
- [x] VM Backend: Implement `flush` in `crates/auto-lang/src/vm/io.rs` <!-- id: 6 -->
- [x] VM Registration: Register in `crates/auto-lang/src/vm.rs` <!-- id: 7 -->
- [x] Verification: Add `test_116_std_file_flush` to `src/trans/c.rs` <!-- id: 10 -->
- [x] Verification: Add/Update A2C transpilation test <!-- id: 9 -->
- [x] Verification: Improved VM `test_std_file_flush` to run actual code <!-- id: 11 -->
- [x] Cleanup: Remove misplaced `test_c_trans_std_file_flush` from `lib.rs` <!-- id: 12 -->
