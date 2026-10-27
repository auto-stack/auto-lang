# 本轮复现入口

工作目录固定 D:/autostack/.wt/lang-828/auto-lang，冻结 a030427fe；全部定向，不运行根 cargo 别名档。

1. `python D:/autostack/auto-lang/docs/plans/attachments/828/review-t10-a030427fe/freeze.py`：首次 baseline，再次 final-state。
2. `cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828 --no-run --message-format=json` → long-build.jsonl；运行其中唯一 function_incremental_828 executable，参数 `512 --test-threads=1 --nocapture` → 512.log/512-result.json（含exe hash）。
3. `cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test cache_tier --test mir_tier --test abc_semantics_tier --test binding_tier --test vertical --test native_admission_tier --test native_trace_parity --test case_binding --test resources_tier --lib -p auto-av-probe -p auto-av-core -- --test-threads=1` → compat.log/compat.exit。
4. `cargo test --locked --manifest-path experimental/ac-core/Cargo.toml --test hir_verify --test scalar_hir --test source_frontend --test source_hir --test scalar_source --test text_source --test record_source --test text_record_source -- --test-threads=1` → ac-core.log/ac-core.exit。
5. `cargo test --locked --manifest-path experimental/ac-syntax/Cargo.toml -p ac-syntax-core-adapter --test scalar_projection_ -- --test-threads=1` → scalar-projection.log/.exit。
6. `powershell -NoProfile -File D:/autostack/auto-lang/docs/plans/attachments/828/review-t10-a030427fe/run-controls.ps1 -Source close-replay.rs -Filter close_ -AllLibraries` → close.log/close.exit；原源include保持，Cargo rlib hash在linked-libraries.json。
7. rustfmt --check --edition 2021 --config skip_children=true，参数为fmt-files.json全列表 → rustfmt.log/exit；git diff --check f79b1b835 HEAD -- experimental crates scripts → source-diff-check.log/exit。整份含原始raw的diff-check另存，不改写raw。

同冻结重复检查无需仅为复现再次跑重负载。后续格式-only新提交按 format-closure.json 的 rustfmt精确输出核等价，核依赖/配置/输入不变；源语义变化才要求相应重测。正式战役本轮复用相邻R5同freeze官方一次5/5及raw，未新跑、未覆盖原目录。
